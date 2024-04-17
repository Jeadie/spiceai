use crate::raft::{raft_service_server::RaftService, AppendEntryRequest, AppendEntryResponse, ClientRequestMessage, ClientRequestResponse, HeartbeatMessage, HeartbeatResponse, LogEntry, RequestVoteRequest, RequestVoteResponse};
use tonic::{async_trait, Request, Response, Status};

pub struct LeaderState {
    next_index: Vec<u64>,
    match_index: Vec<u64>
}

pub struct LocalState {
    commit_index: u64,
    last_applied: u64,
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
}   

impl ConsensusModule {
    pub fn new(spicepod_file: String) -> Self {
        ConsensusModule {
            position: Position::Follower,
            state: PersistState {
                current_term: 0,
                voted_for: None,
                log: vec![]
            },
            local_state: LocalState {
                commit_index: 0,
                last_applied: 0
            },
            spicepod_file
        }
    }
}

#[async_trait]
impl RaftService for ConsensusModule {
    async fn append_entry(&self, request: Request<AppendEntryRequest>) -> Result<Response<AppendEntryResponse>, Status> {
        println!("Received append entry request: {:?}", request);
        todo!()
    }

    async fn request_vote(&self, request: tonic::Request<RequestVoteRequest>) -> Result<Response<RequestVoteResponse>, Status> {
        println!("Received request vote request: {:?}", request);
        todo!()
    }

    async fn client_request(&self, request: tonic::Request<ClientRequestMessage>) -> Result<Response<ClientRequestResponse>, Status> {
        println!("Received client request: {:?}", request);
        todo!(  )
    }

    async fn heartbeat(&self, request: tonic::Request<HeartbeatMessage>) -> Result<Response<HeartbeatResponse>, Status> {
        println!("Received heartbeat: {:#?}", request);
        todo!()
    }
}
