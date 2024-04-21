use std::{net::SocketAddr, sync::{Arc, RwLock}, thread::sleep};
use consensus::ConsensusServer;
use snafu::prelude::*;

use raft::raft_service_server::RaftServiceServer;

mod raft;
mod consensus;
mod election_timer;


#[derive(Debug, Snafu)]
pub enum Error {
    
    #[snafu(display("Unable to start Raft consensus server: {source}"))]
    UnableToStartConsensus { source: tonic::transport::Error },

    #[snafu(display("unable to handle election loop"))]
    ErrorToHandleElectionLoop,
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn start_consensus(addr: SocketAddr, spice_file: String, peers: Vec<SocketAddr>) -> Result<()>{
    let module: Arc<RwLock<consensus::ConsensusModule>> = Arc::new(RwLock::new(consensus::ConsensusModule::new(spice_file, peers)));

    let election_module = module.clone();
    let _handle_election_timeout_handle = std::thread::spawn(move || handle_election_timeout(election_module));

    tonic::transport::Server::builder()
        .add_service(RaftServiceServer::new(ConsensusServer{
            module: module.clone()
        }))
        .serve(addr)
        .await
        .context(UnableToStartConsensusSnafu)
}

async fn handle_election_timeout(module: Arc<RwLock<consensus::ConsensusModule>>) -> Result<()> {
    loop {
        let sleep_dur = module.read()
            .map_err(|_| Error::ErrorToHandleElectionLoop)?
            .local_state.election_timer.until_next_election();
        
        // Trigger re-election.
        if sleep_dur.is_zero() {
            tracing::info!("Reelection triggered...");
            module.write()
                .map_err(|_| Error::ErrorToHandleElectionLoop)?
                .run_reelection().map_err(|_| Error::ErrorToHandleElectionLoop)?;
                
        } else {
            tracing::info!("Waiting {:#?} until next election", sleep_dur);
            sleep(sleep_dur);
        }

    }
}