use std::{sync::Arc, time::Duration};
use std::sync::atomic::{AtomicBool};

use taiko_stratum::{
    codec::StratumCodec,
    message::StratumMessage,
};
use taiko_stratum::codec::ResponseParams;
use futures_util::sink::SinkExt;
use json_rpc_types::Id;
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
use tracing::{error, info, warn, debug};
use crate::prover::ProverEvent;
use crate::prover::LATEST_TASK_CONTENT;
use crate::prover::TASK_HANDLER;

pub struct Client {
    pub name: String ,
    pub server: String,
    pub uuid:String,
    pub sender: Arc<Sender<StratumMessage>>,
    pub busy: Arc<AtomicBool>,
    pub receiver: Arc<Mutex<Receiver<StratumMessage>>>,
}

impl Client {
    pub fn init(name: String, device_id:String,server: String) -> Arc<Self> {
        let (sender, receiver) = mpsc::channel(4096);
        Arc::new(Self {
            name,
            server,
            uuid:device_id,
            sender: Arc::new(sender),
            busy:  Arc::new(AtomicBool::new(false)),
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

pub async fn start(prover_sender: Arc<Sender<ProverEvent>>, client: Arc<Client>) {

    task::spawn(async move {
        let receiver = client.receiver();
        let mut id = 1;
        loop {
            info!("Connecting to server...");

            match timeout(Duration::from_secs(10), TcpStream::connect(&client.server)).await {
                Ok(socket) => match socket {
                    Ok(socket) => {
                        info!("Connected to {}", client.server);
                        let mut framed = Framed::new(socket, StratumCodec::default());

                        //step1:send Subscribe msg
                        let handshake = StratumMessage::Subscribe(
                            Id::Num(id),
                            "test".to_string(),
                            "test".to_string(),
                            2, //just for test
                            4,
                            6,
                        );
                        id += 1;
                        if let Err(e) = framed.send(handshake).await {
                            error!("Error sending handshake: {}", e);
                        } else {
                            info!("Send handshake msg over");
                        }

                        match framed.next().await {
                            None => {
                                error!("Unexpected end of stream");
                                sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                            Some(Ok(message)) => match message {
                                StratumMessage::Response(_, _, result) => {
                                    info!("Handshake successful,result is {:?}",result);
                                }
                                _ => {
                                    error!("Unexpected message: {:?}", message.name());
                                }
                            },
                            Some(Err(e)) => {
                                error!("Error receiving handshake: {}", e);
                                sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                        }

                        //step2:send Authorize msg
                        let worker_access_key = client.name.clone();
                        let uuid = client.uuid.clone();
                        let authorization =
                            StratumMessage::Authorize(Id::Num(id), worker_access_key, uuid); //access_key + uuid
                        id += 1;
                        if let Err(e) = framed.send(authorization).await {
                            error!("Error sending authorization: {}", e);
                        } else {
                            info!("Sent authorization msg over");
                        }
                        match framed.next().await {
                            None => {
                                error!("Unexpected end of stream");
                                sleep(Duration::from_secs(2)).await;
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
                                sleep(Duration::from_secs(2)).await;
                                continue;
                            }
                        }

                        // step3:send Heartbeat msg
                        info!("send heartbeat to server when startup");
    
                 
                        let heartbeat = StratumMessage::Heartbeat(Id::Num(id),String::from(""),String::from("")); //initial heartbeat
                        if let Err(e) = framed.send(heartbeat).await {
                                error!("Error sending heartbeat in startup: {}", e);
                            } else {
                                info!("Sent heartbeat msg over");
                        }
    

                        let receiver = &mut *receiver.lock().await;
                        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(3));
                        loop {
                            tokio::select! {
                                //process the msg send by prover
                                Some(message) = receiver.recv() => { 
                                    // let message = message.clone();
                                    let name = message.name();
                                    if let Err(e) = framed.send(message).await {
                                        error!("Error sending {}: {:?}", name, e);
                                        cancel_task().await;
                                    }
                                }

                                _ = heartbeat_interval.tick() => {
                                    let task = LATEST_TASK_CONTENT.lock().await;
                                    let task_current = (*task).clone();
                                    let heart_msg: Vec<&str> =task_current.split("#").collect();
                                    if heart_msg.len()==2 {
                                        let heartbeat = StratumMessage::Heartbeat(Id::Num(id),heart_msg[0].to_string(),heart_msg[1].to_string());  
                                        if let Err(e) = framed.send(heartbeat).await {
                                            error!("Error sending heartbeat in loop: {}", e);
                                            cancel_task().await;
                                        } else {
                                            info!("Loop Sent {} heartbeat msg over task :{}",heart_msg[0].to_string(),heart_msg[1].to_string());
                                        }
                                    }else {
                                        let heartbeat = StratumMessage::Heartbeat(Id::Num(id),String::from(""),String::from("")); //initial heartbeat
                                        if let Err(e) = framed.send(heartbeat).await {
                                                error!("Error sending heartbeat in startup: {}", e);
                                        } else {
                                                info!("Sent heartbeat msg over no any task");
                                        }
                                    }
                                }
                             
                                //process the msg from server
                                result = framed.next() => match result {
                                    Some(Ok(message)) => {
                                        debug!("Received {:?} from server", message.name());
                                        match message {
                                            StratumMessage::Notify(id, project_name,task_id,task_content,_) => { 
                                                info!("zkpool : receive {} task of {}",project_name.clone(),task_id);
                                                let resp = StratumMessage::Response(id,Some(ResponseParams::Bool(true)),Some(json_rpc_types::Error::from_code(json_rpc_types::ErrorCode::ServerError(1)))); 
                                                if let Err(e) = framed.send(resp).await {
                                                    error!("Error send  notify Response: {}", e);
                                                    cancel_task().await;
                                                } else {
                                                    debug!("Send notify Response Msg Over");
                                                }

                                                //parse parameter
                                                if let Err(e) = prover_sender.send(ProverEvent::NewWork(project_name.clone(),task_id,task_content)).await {
                                                    error!("Error sending work to prover: {}", e);
                                                    cancel_task().await;
                                                } else {
                                                    debug!("Sent work to prover");
                                                }

                                            }
                                            _ => {
                                                debug!("ignore msg!!!");
                                            }
                                        }
                                    }
                                    Some(Err(e)) => {  //case will not run
                                        warn!("Failed to read the message: {:?}", e);
                                        sleep(Duration::from_secs(1)).await;
                                    }
                                    None => {
                                        error!("Disconnected from server");
                                         //Clear the block task cache
                                        let block_current = LATEST_TASK_CONTENT.clone();
                                        let mut block_id_now = block_current.lock().await;
                                        *block_id_now = String::from("");
                                        cancel_task().await;
                                        sleep(Duration::from_secs(1)).await;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to connect to operator: {}", e);
                        cancel_task().await;
                        sleep(Duration::from_secs(2)).await;
                    }
                },
                Err(_) => {
                    error!("Failed to connect to operator: Timed out");
                    cancel_task().await;
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }
    });
}


pub async fn cancel_task(){
    let task_temp = TASK_HANDLER.clone();
    let mut queue = task_temp.lock().await;
    while queue.len() > 0 {
        info!("clear the old task handle");
        for i in queue.iter() {
            i.abort();
            //drop(i.to_owned())
        }
        queue.clear(); 
    }
}

