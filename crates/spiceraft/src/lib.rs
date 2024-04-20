use std::{net::SocketAddr, sync::{Arc, RwLock}};
use consensus::ConsensusServer;
use snafu::prelude::*;

use raft::raft_service_server::RaftServiceServer;

mod raft;
mod consensus;


#[derive(Debug, Snafu)]
pub enum Error {
    
    #[snafu(display("Unable to start Raft consensus server: {source}"))]
    UnableToStartConsensus { source: tonic::transport::Error },
}

type Result<T, E = Error> = std::result::Result<T, E>;

pub async fn start_consensus(addr: SocketAddr, spice_file: String, peers: Vec<SocketAddr>) -> Result<()>{
    let module = consensus::ConsensusModule::new(spice_file, peers);
    tonic::transport::Server::builder()
        .add_service(RaftServiceServer::new(ConsensusServer{ module: Arc::new(RwLock::new(module)) }))
        .serve(addr)
        .await
        .context(UnableToStartConsensusSnafu)
}