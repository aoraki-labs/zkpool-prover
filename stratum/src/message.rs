use json_rpc_types::{Error, Id};

use crate::codec::ResponseParams;

// CHANGE(zkpool): use custom StratumMessage protocol
pub enum StratumMessage {
  
    Subscribe(Id, String, String, u64, u64, u64),

    Authorize(Id, String, String),

    Notify(Id,String,String,String,u64),

    Heartbeat(Id,String,String),

    Submit(Id, String,String, String,u8,u32,u8),

    Response(Id, Option<ResponseParams>, Option<Error<()>>),
}

// CHANGE(zkpool): use custom StratumMessage name
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
