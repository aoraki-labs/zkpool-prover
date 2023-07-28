use json_rpc_types::{Error, Id};

use crate::codec::ResponseParams;

pub enum StratumMessage {
  
    Subscribe(Id, String, String, u64, u64, u64),

    Authorize(Id, String, String),

    Notify(Id,String,u64,String,u64),

    Heartbeat(Id,String,String),

    Submit(Id, String,String, String,u8,u32),

    Response(Id, Option<ResponseParams>, Option<Error<()>>),
}

impl StratumMessage {
    pub fn name(&self) -> &'static str {
        match self {
            StratumMessage::Subscribe(..) => "zkpool.subscribe",
            StratumMessage::Authorize(..) => "zkpool.authorize",
            StratumMessage::Notify(..) => "zkpool.notify",
            StratumMessage::Submit(..) => "zkpool.submit",
            StratumMessage::Response(..) => "zkpool.response",
            StratumMessage::Heartbeat(..) => "zkpool.heartbeat",
        }
    }
}