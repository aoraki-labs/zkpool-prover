use std::sync::{
    atomic::{AtomicU64,Ordering},
    Arc,
};

use tracing::{error, info, warn, debug};
use taiko_stratum::message::StratumMessage;
use json_rpc_types::Id;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use tokio::{
sync::mpsc,
task::{self, JoinHandle},
};
// use tracing::error;

use crate::Client;

use std::collections::HashMap;


use std::time::Instant;


use smartcore_ml::{generate_proof,generate_segment_proof};

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProverConfig {
name_list: Vec<String>,
rpc_url_list: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectInfo {
pub name: String,
pub rpc_url: String,
}


lazy_static! {
pub static ref LATEST_TASK_CONTENT: Arc<Mutex<String>> = {
    Arc::new(Mutex::new(String::from("")))
};
pub static ref TASK_HANDLER: Arc<Mutex<Vec<JoinHandle<()>>>> = {
    Arc::new(Mutex::new(Vec::<JoinHandle<()>>::new()))
};
pub static ref PROJECT_LIST: Arc<Mutex<HashMap<String, ProjectInfo>>> = {
    Arc::new(Mutex::new(HashMap::default()))
};
}

pub struct Prover {
sender: Arc<mpsc::Sender<ProverEvent>>,
client: Arc<Client>,
current_block: Arc<AtomicU64>,
}

#[allow(clippy::large_enum_variant)]
pub enum ProverEvent {
NewWork(String,String, String),
}

impl Prover {
pub async fn init(
    client: Arc<Client>,
) -> Result<Arc<Self>,String> {

    let (sender, mut receiver) = mpsc::channel(4096);

    let prover = Arc::new(Self {
        sender: Arc::new(sender),
        client,
        current_block: Default::default(),
    });

    let p = prover.clone();
    let _ = task::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            match msg {
                   ProverEvent::NewWork(project,task_id,task_content) => {    
                        //clear the older task handle,to be optimize
                        let task_temp = TASK_HANDLER.clone();
                        let queue = task_temp.lock().await;
                        if queue.len()>0 {
                            let _ = task::spawn(async move {
                                let task_temp = TASK_HANDLER.clone();
                               let mut queue = task_temp.lock().await;
                               if queue.len()>0 {
                                   let task_handle =&(*queue)[0];
                                   task_handle.abort();
                                   //drop(task_handle);
                                   queue.remove(0);
                               }
                               debug!("clear the old task over");
                           });
                        }

                        //Cache the newest block number
                        let cached_task=format!("{}*{}",project,task_id);
                        let current_task = LATEST_TASK_CONTENT.clone();
                        let mut current_task_content = current_task.lock().await;
                        *current_task_content = cached_task;

                        //compute the proof
                        p.new_work(    //work
                            project,
                            task_id,
                            task_content
                        )
                        .await;
                }
            }
        }
    });       
    info!("Created prover message handler");
    Ok(prover)
}

pub fn sender(&self) -> Arc<mpsc::Sender<ProverEvent>> {
    self.sender.clone()
}

async fn new_work(&self,project_name:String, block: String, task_content: String) {
    // self.current_block.store(block, Ordering::SeqCst);
    let client = self.client.clone();
    let project_map = PROJECT_LIST.lock().await;
    let project_map_info = (*project_map).clone();
    let project_name_bak = project_name.clone();
    let task_id: String = block.clone();
    let task_id_split: String = block.clone();

    let project_info = match project_map_info.get(&project_name.clone()){
        Some(r) => r.clone(),
        None => {
            error!("can not find this project {} info,ignore it",project_name.clone());
            return
        },
    };

    info!("receive task,project name is:{},task id is:{},task content is:{}",project_name.clone(),block.clone(),task_content);

    if project_name.clone()=="demo".to_string(){
        let _l2_rpc = project_info.rpc_url;

        //Parse the task content
        let input = String::from_utf8(hex::decode(task_content).unwrap()).unwrap();
        let task_vec: Vec<&str> = input.split("\"").collect(); 
        if task_vec.len() <= 2{  //invalid proof request paramter
            error!("{} task parameter error or requestor dummy task,ignore it",project_name.clone());
            return
        } 
        let demo_task_inputs=task_vec[1].to_string();
        info!("send the risc0 input is:{}",demo_task_inputs.clone());
        
        let client_2 = client.clone();
        let _ = task::spawn(async move { //maybe multi-thread compute task in future

            let _task_handle = tokio::task::spawn(async move{
                let _time_started = Instant::now();
                tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                let status:u8=1;
                let mut res = "".to_string();
                let _proof_result = "dummy_data".to_string();

                let _ = tokio::task::spawn(async move{ //main process
                    let time_started = Instant::now();

                    //parse the task_id content
                    let task_id_split_vec: Vec<&str>=task_id_split.split("@").collect();
                    if task_id_split.len()==1{
                        res = generate_proof(demo_task_inputs).await;
                    }else {
                        info!("invoke generate_segment_proof para is:{}-{}",demo_task_inputs,task_id_split_vec[1].to_string());
                        res = generate_segment_proof(demo_task_inputs,task_id_split_vec[1].to_string()).await;
                    }

                    let time_gap =(Instant::now().duration_since(time_started).as_millis() as u32)/1000;
                    if need_send_proof(project_name_bak.clone(), block.clone()).await {
                        let message = StratumMessage::Submit(
                            Id::Num(0),
                            project_name_bak.clone(),
                            block.to_string(),
                            res.clone(),
                            1,
                            time_gap,
                            status,
                        );
                        if let Err(error) = client_2.sender().send(message).await { 
                            error!("Failed to send PoolResponse: {}", error);
                        }else{
                            info!("zkpool:send the proof of block:{} success,proof is {:?},time consumed:{}",block,res,time_gap);
                        }
                        info!("zkpool:end computed the task of block:{}",block);
    
                        let current_task = LATEST_TASK_CONTENT.clone();
                        let mut current_task_content = current_task.lock().await;
                        *current_task_content = String::from("");

                    }
                });

                // let time_gap =(Instant::now().duration_since(time_started).as_millis() as u32)/1000;

                info!("start to send heartbeat msg when receive task");
                let message = StratumMessage::Heartbeat(
                    Id::Num(0),
                    project_name.clone(),
                    task_id.clone(),
                );
                if let Err(error) = client.sender().send(message).await { 
                    error!("Failed to send Heartbeat msg: {}", error);
                }else{
                    info!("zkpool:send the Heartbeat success,task id:{},project id:{}",project_name.clone(),task_id.clone());
                }

                // if need_send_proof(project_name.clone(), task_id.clone()).await {
                //     let task = LATEST_TASK_CONTENT.lock().await;
                //     let task_current = (*task).clone();
                //     let heart_msg: Vec<&str> =task_current.split("#").collect();
                //     if heart_msg.len()==2 {
                //         let message = StratumMessage::Heartbeat(
                //             Id::Num(0),
                //             heart_msg[0].to_string(),
                //             heart_msg[1].to_string(),
                //         );
                //         if let Err(error) = client.sender().send(message).await { 
                //             error!("Failed to send Heartbeat msg: {}", error);
                //         }else{
                //             info!("zkpool:send the Heartbeat success,task id:{},project id:{}",heart_msg[1].to_string(),heart_msg[0].to_string());
                //         }
                //     }else {
                //         let message = StratumMessage::Heartbeat(
                //             Id::Num(0),
                //             "".to_string(),
                //             "".to_string(),
                //         );
                //         if let Err(error) = client.sender().send(message).await { 
                //             error!("Failed to send Heartbeat msg: {}", error);
                //         }else{
                //             info!("zkpool:send the Heartbeat success");
                //         }
                //     }
                   
                //     // info!("zkpool:end computed the task of block:{}",block);

                //     // let current_task = LATEST_TASK_CONTENT.clone();
                //     // let mut current_task_content = current_task.lock().await;
                //     // *current_task_content = String::from("");
                // }  
            });

            // cache the task handle
            // let task_handle_vec = TASK_HANDLER.clone();
            // let mut queue = task_handle_vec.lock().await;
            // queue.push(task_handle);
        });
    }else{
        info!("ignore the unrecognized {} task",project_name);
    }
    info!("******one block task in process********");
}
}


pub async fn need_send_proof(_project:String,block:String) -> bool {
    let current_task = LATEST_TASK_CONTENT.clone();
    let current_task_content = current_task.lock().await;
    if *current_task_content==String::from(""){ //initial
        return true
    }else {
        let task_vec: Vec<&str> = (*current_task_content).split("*").collect();
        if task_vec.len() != 2{
            return true
        }else {
            let cached_block = task_vec[1].to_string();
            info!("cached task info is:{}",cached_block);
            if cached_block ==block{
                return true
            }
        }
    }
    return false
}