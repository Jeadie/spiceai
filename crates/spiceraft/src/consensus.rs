use std::{cmp::min, net::SocketAddr, sync::{Arc, RwLock}};

use crate::{election_timer::ElectionTimer, raft::{raft_service_server::RaftService, AppendEntryRequest, AppendEntryResponse, ClientRequestMessage, ClientRequestResponse, LogEntry, RequestVoteRequest, RequestVoteResponse}};

use snafu::prelude::*;
use tonic::{async_trait, Request, Response, Status};

pub struct LeaderState {
    next_index: Vec<u64>,
    match_index: Vec<u64>
}

pub struct LocalState {
    commit_index: u64,
    last_applied: u64,
    pub election_timer: ElectionTimer
}

// State that should be persisted before responding to RPC requests.
// TODO: read/write this to disk.
pub struct PersistState {
    pub current_term: u64,
    pub voted_for: Option<u64>,
    pub log: Vec<LogEntry>
}

pub enum Position {
    Leader,
    Follower,
    Candidate(LeaderState)
}

pub struct ConsensusModule {
    pub spicepod_file: String,

    pub position: Position,
    pub state: PersistState,
    pub local_state: LocalState,

    pub peers: Vec<SocketAddr>,
}   

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Unable to write to consensus state"))]
    UnableToUpdateNode,

    #[snafu(display("Failed to run candidate reelection"))]
    UnableToRunCandidateReelection,

}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct ConsensusServer {
    pub module: Arc<RwLock<ConsensusModule>>
}

impl ConsensusServer {
    pub fn update_candidate(&self, candidate_id: u64) -> Result<()> {
        self.module.write()
            .map_err(|_| Error::UnableToUpdateNode{})?
            .state.voted_for = Some(candidate_id);

        Ok(())
    }
}

impl ConsensusModule {
    pub fn new(spicepod_file: String, peers: Vec<SocketAddr>) -> Self {
        ConsensusModule {
            position: Position::Follower,
            state: PersistState {
                current_term: 0,
                voted_for: None,
                log: vec![]
            },
            local_state: LocalState {
                commit_index: 0,
                last_applied: 0,
                election_timer: ElectionTimer::default(),
            },
            spicepod_file,
            peers
        }
    }

    /// Returns true of (last_term, last_index) is at least as up-to-date as the log.
    pub fn log_is_up_to_date(&self, last_term: u64, last_index: u64) -> bool {
        let last_log_term = self.state.log.last().map(|entry| entry.term).unwrap_or(0);
        let last_log_index = self.state.log.iter().rev().filter(|x| x.term == last_log_term).count() as u64;
        
        last_log_term < last_term || (last_log_term == last_term && last_log_index <= last_index)
    }

    pub fn is_correct_candidate(&self,  candidate_id: u64) -> bool {
        self.state.voted_for.is_none() || self.state.voted_for == Some(candidate_id)
    }

    pub fn run_reelection(&self) -> Result<()> {
        return Ok(())
    }
}

#[async_trait]
impl RaftService for ConsensusServer {
    async fn append_entry(self: Arc<Self>, request: Request<AppendEntryRequest>) -> Result<Response<AppendEntryResponse>, Status> {
        let AppendEntryRequest{term, leader_ids, prev_log_index, prev_log_term, entries, leader_commit} = request.get_ref();
        
        match self.module.read() {
            Err(_) => {
                return Err(Status::internal("Unable to read module state"));
            }
            Ok(module) => {
                // Reply false if term < currentTerm (§5.1)
                if module.state.current_term > *term {
                    return Ok(Response::new(AppendEntryResponse {
                        term: module.state.current_term,
                        success: false
                    }));
                }

                //  Reply false if log doesn’t contain an entry at prevLogIndex whose term matches prevLogTerm (§5.3)
                if (module.state.log.len() as u64) < *prev_log_index || module.state.log[*prev_log_index as usize].term != *prev_log_term {
                    // TODO: Also need to delete

                    return Ok(Response::new(AppendEntryResponse {
                        term: module.state.current_term,
                        success: false
                    }));
                }
            },
        };

        match self.module.write() {
            Err(_) => {
                Err(Status::internal("Unable to write log entries"))
            }
            Ok(mut write_module) => {
                for i in 0..entries.len() {
                    let idx = (*prev_log_index as usize) + 1 + i;
                    write_module.state.log[idx] = LogEntry{
                        term: *term,
                        command: entries[i].command.clone()
                    }
                }
                write_module.state.current_term = *term;
        
                // If leaderCommit > commitIndex, set commitIndex = min(leaderCommit, index of last new entry)
                write_module.local_state.commit_index = min(
                    min(write_module.local_state.commit_index, *leader_commit),
                    (write_module.state.log.len()-1) as u64
                );
        
                Ok(Response::new(AppendEntryResponse {
                    term: write_module.state.current_term,
                    success: false
                }))
            }
        }
    }

    async fn request_vote(self: Arc<Self>, request: tonic::Request<RequestVoteRequest>) -> Result<Response<RequestVoteResponse>, Status> {
        let RequestVoteRequest{candidate_id, term, last_log_index, last_log_term} = *request.get_ref();

        let Ok(module) =  self.module.read() else {
            return Err(Status::internal("Unable to read module state"));
        };

        // Invalid/out-of-date term, reject the vote.
        if term < module.state.current_term {
            return Ok(Response::new(RequestVoteResponse {
                term: module.state.current_term,
                vote_granted: false
            }));
        };


        if module.is_correct_candidate(candidate_id) && module.log_is_up_to_date(last_log_term, last_log_index) {
            return match self.update_candidate(candidate_id) {
                Ok(_) => {Ok(Response::new(RequestVoteResponse {
                    term: module.state.current_term,
                    vote_granted: true
                }))},
                Err(_) => return Err(Status::internal("Unable to update candidate"))
            };
        }

        Ok(Response::new(RequestVoteResponse {
            term: module.state.current_term,
            vote_granted: false
        }))

    }

    async fn client_request(self: Arc<Self>,  request: tonic::Request<ClientRequestMessage>) -> Result<Response<ClientRequestResponse>, Status> {
        println!("Received client request: {:?}", request);
        todo!(  )
    }
}
