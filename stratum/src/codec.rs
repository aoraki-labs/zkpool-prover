use std::io;

use bytes::BytesMut;
use downcast_rs::{impl_downcast, DowncastSync};
use erased_serde::Serialize as ErasedSerialize;
use json_rpc_types::{Id, Request, Response, Version};
use serde::{ser::SerializeSeq, Deserialize, Serialize};
use serde_json::Value;
use tokio_util::codec::{AnyDelimiterCodec, Decoder, Encoder};
use tracing::debug;

use crate::message::StratumMessage;

pub struct StratumCodec {
    codec: AnyDelimiterCodec,
}

impl Default for StratumCodec {
    fn default() -> Self {
        Self {
            // Notify is ~400 bytes and submit is ~1750 bytes. 4096 should be enough for all messages
            // TODO: verify again
            codec: AnyDelimiterCodec::new_with_max_length(vec![b'\n'], vec![b'\n'], 4096),
        }
    }
}

// CHANGE(zkpool): use custom StratumMessage type
#[derive(Serialize, Deserialize)]
struct NotifyParams(String,String,String,u64);

#[derive(Serialize, Deserialize)]
struct SubmitParams(String,String, String,u8,u32,u8);

#[derive(Serialize, Deserialize)]
struct HeartBeatParams(String, String);

#[derive(Serialize, Deserialize)]
struct SubscribeParams(String, String,u64,u64,u64);

pub trait BoxedType: ErasedSerialize + Send + DowncastSync {}
erased_serde::serialize_trait_object!(BoxedType);
impl_downcast!(sync BoxedType);

impl BoxedType for String {}
impl BoxedType for Option<u64> {}
impl BoxedType for Option<String> {}

pub enum ResponseParams {
    Bool(bool),
    Array(Vec<Box<dyn BoxedType>>),
    Null,
}

impl Serialize for ResponseParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ResponseParams::Bool(b) => serializer.serialize_bool(*b),
            ResponseParams::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            ResponseParams::Null => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for ResponseParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Bool(b) => Ok(ResponseParams::Bool(b)),
            Value::Array(a) => {
                let mut vec: Vec<Box<dyn BoxedType>> = Vec::new();
                a.iter().for_each(|v| match v {
                    Value::Null => vec.push(Box::new(None::<String>)),
                    Value::String(s) => vec.push(Box::new(s.clone())),
                    Value::Number(n) => vec.push(Box::new(n.as_u64())),
                    _ => {}
                });
                Ok(ResponseParams::Array(vec))
            }
            Value::Null => Ok(ResponseParams::Null),
            _ => Err(serde::de::Error::custom("invalid response params")),
        }
    }
}

// CHANGE(zkpool): use custom StratumMessage protocol and name 
impl Encoder<StratumMessage> for StratumCodec {
    type Error = io::Error;

    fn encode(&mut self, item: StratumMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let bytes = match item {
            StratumMessage::Subscribe(id, user_agent, protocol_version, machine_cpu_num,machine_gpu_num,machine_mem) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "zkpool.subscribe",
                    params: Some(SubscribeParams(user_agent, protocol_version,machine_cpu_num,machine_gpu_num,machine_mem)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Authorize(id, worker_name, worker_password) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "zkpool.authorize",
                    params: Some(vec![worker_name, worker_password]),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            // StratumMessage::Notify(_,block_id, address, propose_tx,clean) => {
                StratumMessage::Notify(_,id_name,task_id,task_content,degree) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "zkpool.notify",
                    params: Some(NotifyParams(id_name,task_id,task_content,degree)),
                    id: None,
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Heartbeat(id, project_name,block) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "zkpool.heartbeat",
                    params: Some(HeartBeatParams(project_name,block)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Submit(id, project_name,block, proof,degree,time,status) => {
                let request = Request {
                    jsonrpc: Version::V2,
                    method: "zkpool.submit",
                    params: Some(SubmitParams(project_name,block, proof,degree,time,status)),
                    id: Some(id),
                };
                serde_json::to_vec(&request).unwrap_or_default()
            }
            StratumMessage::Response(id, result, _) =>  {
                let response = Response::<Option<ResponseParams>, ()>::result(Version::V2, result, Some(id));
                serde_json::to_vec(&response).unwrap_or_default()
            },
        };

        let string =
            std::str::from_utf8(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        self.codec
            .encode(string, dst)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(())
    }
}

fn unwrap_str_value(value: &Value) -> Result<String, io::Error> {
    match value {
        Value::String(s) => Ok(s.clone()),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Param is not str")),
    }
}

fn unwrap_u64_value(value: &Value) -> Result<u64, io::Error> {
    match value {
        Value::Number(n) => Ok(n
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Param is not u64"))?),
        _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Param is not u64")),
    }
}

// CHANGE(zkpool): use custom StratumMessage protocol and name 
impl Decoder for StratumCodec {
    type Error = io::Error;
    type Item = StratumMessage;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let string = self
            .codec
            .decode(src)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if string.is_none() {
            return Ok(None);
        }
        let bytes = string.unwrap();
        let mut json = serde_json::from_slice::<serde_json::Value>(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        if !json.is_object() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not an object"));
        }
        let mut object = json.as_object().unwrap().clone();
        object.remove("error");
        json = object.clone().into();
        if !json.is_object() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Not an object"));
        }
        debug!(" help debug :New json with no error: {}", json.to_string());
        
        let result = if object.contains_key("method") {
            let request = serde_json::from_value::<Request<Vec<Value>>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = request.id;
            let method = request.method.as_str();
            let params = match request.params {
                Some(params) => params,
                None => return Err(io::Error::new(io::ErrorKind::InvalidData, "No params")),
            };
            match method {
                "zkpool.subscribe" => {
                    if params.len() != 5 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let user_agent = unwrap_str_value(&params[0])?;
                    let protocol_version = unwrap_str_value(&params[1])?;
                    let machine_cpu_num = params[2].as_u64().unwrap();
                    let machine_gpu_num = params[3].as_u64().unwrap();
                    let machine_mem = params[4].as_u64().unwrap();
                    StratumMessage::Subscribe(
                        id.unwrap_or(Id::Num(0)),
                        user_agent,
                        protocol_version,
                        machine_cpu_num,
                        machine_gpu_num,
                        machine_mem,
                    )
                }
                "zkpool.authorize" => {
                    if params.len() != 3 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let worker_name = unwrap_str_value(&params[0])?;
                    let worker_password = unwrap_str_value(&params[1])?;
                    StratumMessage::Authorize(id.unwrap_or(Id::Num(0)), worker_name, worker_password)
                }
                "zkpool.heartbeat" => {
                    if params.len() != 1 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let project_name = unwrap_str_value(&params[0])?;
                    let block = unwrap_u64_value(&params[1])?;
                    StratumMessage::Heartbeat(id.unwrap_or(Id::Num(0)), project_name,block.to_string())
                }

                "zkpool.notify" => {
                    if params.len() != 4 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let project_name = unwrap_str_value(&params[0])?;
                    // let task_id = unwrap_u64_value(&params[1])?;
                    let task_id =  unwrap_str_value(&params[1])?;
                    let task_content = unwrap_str_value(&params[2])?;
                    // let degree = unwrap_u64_value(&params[3])?;
                    let degree =  (unwrap_str_value(&params[3])?).parse::<u64>().unwrap();
                    StratumMessage::Notify(id.unwrap(),project_name,task_id,task_content,degree)
                }
                "zkpool.submit" => {
                    if params.len() != 6 { 
                        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid params"));
                    }
                    let project_name = unwrap_str_value(&params[0])?;
                    let block = unwrap_str_value(&params[1])?;
                    let proof = unwrap_str_value(&params[2])?;
                    let degree = unwrap_u64_value(&params[3])? as u8;
                    let time = unwrap_u64_value(&params[4])?;
                    let status = unwrap_u64_value(&params[5])? as u8;
                    StratumMessage::Submit(id.unwrap_or(Id::Num(0)), project_name,block, proof,degree,time as u32,status)
                }
                _ => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown method"));
                }
            }
        } else {
            let response = serde_json::from_value::<Response<ResponseParams, ()>>(json)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let id = response.id;
            match response.payload {
                Ok(payload) => StratumMessage::Response(id.unwrap_or(Id::Num(0)), Some(payload), None),
                Err(error) => StratumMessage::Response(id.unwrap_or(Id::Num(0)), None, Some(error)),
            }
        };
        Ok(Some(result))
    }
}
