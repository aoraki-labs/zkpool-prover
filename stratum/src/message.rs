use json_rpc_types::{Error, Id};

use crate::codec::ResponseParams;

pub enum StratumMessage {
    /// This first version doesn't support vhosts.
    /// (id, user_agent, protocol_version, session_id)
    Subscribe(Id, String, String, Option<String>),

    /// (id, worker_name, worker_password)
    Authorize(Id, String, String),

    /// New job from the mining pool.
    /// See protocol specification for details about the fields.
    /// (job_id, epoch number, difficulty, epoch_challenge, address, clean_jobs)
    Notify(String, u64, u64, String, Option<String>, bool),

    /// Submit shares to the pool.
    /// See protocol specification for details about the fields.
    /// (id, job_id, provesolution)
    Submit(Id, String, String),

    /// (id, result, error)
    Response(Id, Option<ResponseParams>, Option<Error<()>>),
}

impl StratumMessage {
    pub fn name(&self) -> &'static str {
        match self {
            StratumMessage::Subscribe(..) => "mining.subscribe",
            StratumMessage::Authorize(..) => "mining.authorize",
            StratumMessage::Notify(..) => "mining.notify",
            StratumMessage::Submit(..) => "mining.submit",
            StratumMessage::Response(..) => "mining.response",
        }
    }
}
