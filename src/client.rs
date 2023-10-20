use std::{sync::Arc, time::Duration};

use aleo_stratum::{
    codec::{ResponseParams, StratumCodec},
    message::StratumMessage,
};
use futures_util::sink::SinkExt;
use json_rpc_types::Id;
use snarkvm::{prelude::Testnet3,  prelude::FromBytes};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc,
        mpsc::{Receiver, Sender},
        Mutex,
    },
    task,
    time::{sleep, timeout},
};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;
use tracing::{debug, error, info};

use snarkvm::prelude::coinbase::EpochChallenge;

use crate::prover::ProverEvent;

pub struct Client {
    pub name: String ,
    pub server: String,
    pub uuid:String,
    pub sender: Arc<Sender<StratumMessage>>,
    pub receiver: Arc<Mutex<Receiver<StratumMessage>>>,
}

impl Client {
    pub fn init(name: String, device_id:String,server: String) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(1024);
        Arc::new(Self {
            name,
            server,
            uuid:device_id,
            sender: Arc::new(sender),
            receiver: Arc::new(Mutex::new(receiver)),
        })
    }

    pub fn sender(&self) -> Arc<Sender<StratumMessage>> {
        self.sender.clone()
    }

    pub fn receiver(&self) -> Arc<Mutex<Receiver<StratumMessage>>> {
        self.receiver.clone()
    }
}

pub fn start(prover_sender: Arc<Sender<ProverEvent>>, client: Arc<Client>) {
    task::spawn(async move {
        let receiver = client.receiver();
        let mut id = 1;
        let mut server_prefix: Option<String> = None;
        server_prefix = server_prefix.clone();
        loop {
            info!("Connecting to server...");
            server_prefix = server_prefix.clone();
            match timeout(Duration::from_secs(10), TcpStream::connect(&client.server)).await {
                Ok(socket) => match socket {
                    Ok(socket) => {
                        info!("Connected to {}", client.server);
                        let mut framed = Framed::new(socket, StratumCodec::default());
                        let _pool_address: Option<String> = None;

                        let handshake = StratumMessage::Subscribe(
                            Id::Num(id),
                            format!("ZKPoolProver/{}", env!("CARGO_PKG_VERSION")),
                            "AleoStratum/2.0.0".to_string(),
                            None,
                        );
                        id += 1;
                        if let Err(e) = framed.send(handshake).await {
                            error!("Error sending handshake: {}", e);
                        } else {
                            info!("Sent handshake over");
                        }
                        match framed.next().await {
                            None => {
                                error!("Unexpected end of stream");
                                sleep(Duration::from_secs(5)).await;
                                continue;
                            }
                            Some(Ok(message)) => match message {
                                StratumMessage::Response(_, params, _) => {
                                    match params {
                                        Some(ResponseParams::Array(array)) => {
                                            if let Some(prefix) = array.get(1) {
                                                if let Some(prefix) = prefix.downcast_ref::<String>() {
                                                    server_prefix = Some(prefix.to_string().clone());
                                                    if let Some(prefix_length) = array.get(2) {
                                                        if let Some(prefix_length) = prefix_length.downcast_ref::<Option<u64>>() {
                                                            info!("prefix is {}, length is {}", prefix, prefix_length.unwrap());
                                                        }
                                                    }
                                                } else {
                                                    error!("Invalid type for prefix");
                                                    sleep(Duration::from_secs(5)).await;
                                                    continue;
                                                }
                                            } else {
                                                error!("Invalid handshake response");
                                                sleep(Duration::from_secs(5)).await;
                                                continue;
                                            }
                                        }
                                        None => {
                                            error!("No handshake response");
                                            sleep(Duration::from_secs(5)).await;
                                            continue;
                                        }
                                        _ => {
                                            error!("Invalid handshake response");
                                            sleep(Duration::from_secs(5)).await;
                                            continue;
                                        }
                                    }
                                    info!("Handshake successful");
                                }
                                _ => {
                                    error!("Unexpected message: {:?}", message.name());
                                }
                            },
                            Some(Err(e)) => {
                                error!("Error receiving handshake: {}", e);
                                sleep(Duration::from_secs(5)).await;
                                continue;
                            }
                        }
                        let worker_access_key = client.name.clone();
                        let uuid = client.uuid.clone();
                        let authorization =
                            StratumMessage::Authorize(Id::Num(id), worker_access_key, uuid);
                        id += 1;
                        if let Err(e) = framed.send(authorization).await {
                            error!("Error sending authorization: {}", e);
                        } else {
                            debug!("Sent authorization");
                        }
                        match framed.next().await {
                            None => {
                                error!("Unexpected end of stream");
                                sleep(Duration::from_secs(5)).await;
                                continue;
                            }
                            Some(Ok(message)) => match message {
                                StratumMessage::Response(_, _, _) => {
                                    info!("Authorization successful");
                                }
                                _ => {
                                    error!("Unexpected message: {:?}", message.name());
                                }
                            },
                            Some(Err(e)) => {
                                error!("Error receiving authorization: {}", e);
                                sleep(Duration::from_secs(5)).await;
                                continue;
                            }
                        }
                        let receiver = &mut *receiver.lock().await;
                        loop {
                            tokio::select! {
                                Some(message) = receiver.recv() => {
                                    // let message = message.clone();
                                    let name = message.name();
                                    info!("Sending {} to server", name);
                                    if let Err(e) = framed.send(message).await {
                                        error!("Error sending {}: {:?}", name, e);
                                    }
                                }
                                result = framed.next() => match result {
                                    Some(Ok(message)) => {
                                        debug!("Received {} from server", message.name());
                                        match message {
                                            StratumMessage::Response(_, result, _error) => {
                                                match result {
                                                    Some(params) => {
                                                        match params {
                                                            ResponseParams::Bool(_result) => {
                                                                debug!("receive zkpool Response msg");
                                                            }
                                                            _ => {
                                                                debug!("Unexpected response params");
                                                            }
                                                        }
                                                    }
                                                    None => {
                                                        debug!("receive None msg");
                                                    }
                                                }
                                            }
                                            StratumMessage::Notify(job_id, epoch_number, difficulty, epoch_challenge, address, _) => {
                                                let job_id_bytes = job_id.as_bytes();
                                                if job_id_bytes.len() != 10 {
                                                    error!("Unexpected job_id length: {}", job_id_bytes.len());
                                                    continue;
                                                }
                                                let my_server_prefix = server_prefix.clone();
                                                let epoch_challenge_hex_byte = match hex::decode(epoch_challenge){
                                                    Ok(r) => r,
                                                    Err(_) => {
                                                        info!("invalid epoch_challenge_hex_byte");
                                                        continue
                                                    },
                                                };
                                                let my_epoch_challenge = match EpochChallenge::<Testnet3>::from_bytes_le(&epoch_challenge_hex_byte[..]){
                                                    Ok(r) => r,
                                                    Err(_) => {
                                                        info!("can not decode epoch_challenge data:{:?}",epoch_challenge_hex_byte);
                                                        continue
                                                    },
                                                };
                                                if let Err(e) = prover_sender.send(ProverEvent::NewWork(difficulty, epoch_number, my_epoch_challenge, address.unwrap(), job_id,my_server_prefix.unwrap())).await {
                                                    error!("Error sending work to prover: {}", e);
                                                } else {
                                                    info!("Sent work to prover");
                                                }
                                            }
                                            _ => {
                                                info!("Unhandled message: {}", message.name());
                                            }
                                        }
                                    }
                                    Some(Err(e)) => {
                                        info!("Failed to read the message: {:?}", e);
                                    }
                                    None => {
                                        error!("Disconnected from server");
                                        sleep(Duration::from_secs(5)).await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to operator: {}", e);
                        sleep(Duration::from_secs(5)).await;
                    }
                },
                Err(_) => {
                    error!("Failed to connect to operator: Timed out");
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    });
}
