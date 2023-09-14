use std::sync::{
        atomic::{AtomicU64,Ordering},
        Arc,
    };

use log::debug;
use taiko_stratum::message::StratumMessage;
use json_rpc_types::Id;

use lazy_static::lazy_static;
use tokio::sync::Mutex;

use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};
use tracing::{error, info};

use crate::Client;

use std::collections::HashMap;


use std::time::Instant;

//taiko A5 testnet lib core
use zkevm_common::prover::ProofResult;
use prover::shared_state::generate_proof;

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
    NewWork(String,u64, String),
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
                                       drop(task_handle);
                                       queue.remove(0);
                                   }
                                   debug!("clear the old task over");
                               });
                            }

                            //Cache the newest block number
                            let cached_task=format!("{}#{}",project,task_id);
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

    async fn new_work(&self,project_name:String, block: u64, task_content: String) {
        self.current_block.store(block, Ordering::SeqCst);
        let client = self.client.clone();
        let project_map = PROJECT_LIST.lock().await;
        let project_map_info = (*project_map).clone();

        let project_info = match project_map_info.get(&project_name.clone()){
            Some(r) => r.clone(),
            None => {
                error!("can find this project {} info,ignore it",project_name.clone());
                return
            },
        };

        info!("receive task,project name is:{},task id is:{},task content is:{}",project_name.clone(),block,task_content);

        if project_name.clone()=="taikoA5".to_string(){
            let l2_rpc = project_info.rpc_url;
            let task_vec: Vec<&str> = task_content.split("#").collect(); //Parse the task content
            if task_vec.len() != 14{
                error!("{} task parameter error,ignore it",project_name.clone());
                return
            }
            let prover_address=task_vec[0].to_string();
            let l1_signal_service=task_vec[1].to_string();
            let l2_signal_service=task_vec[2].to_string();
            let taiko_12=task_vec[3].to_string();
            let meta_hash=task_vec[4].to_string();
            let blockhash=task_vec[5].to_string();
            let parenthash=task_vec[6].to_string();
            let signalroot=task_vec[7].to_string();
            let graffiti=task_vec[8].to_string();

            let gasused=task_vec[9].to_string().parse::<u64>().unwrap();
            let parentgasused=task_vec[10].parse::<u64>().unwrap();
            let blockmaxgasimit=task_vec[11].parse::<u64>().unwrap();
            let maxtransactionsperblock=task_vec[12].parse::<u64>().unwrap();
            let maxbytespertxlist=task_vec[13].parse::<u64>().unwrap();

            
            let _ = task::spawn(async move { //maybe multi-thread compute task in future
    
                let task_handle = task::spawn(async move {
                    let time_started = Instant::now();
                    let agg_proof_result = match generate_proof(
                    l2_rpc,
                    block,
                    prover_address.clone(),
                    l1_signal_service.clone(),
                    l2_signal_service.clone(),
                    taiko_12.clone(),
                    meta_hash.clone(),
                    blockhash.clone(),
                    parenthash.clone(),
                    signalroot.clone(),
                    graffiti.clone(),
                    gasused,
                    parentgasused,
                    blockmaxgasimit,
                    maxtransactionsperblock,
                    maxbytespertxlist).await{
                        Ok(r) => r,
                        Err(_) => ProofResult::default(),
                    };                  
                    let time_gap =(Instant::now().duration_since(time_started).as_millis() as u32)/1000;
                    info!("try to sumbit the block {} proof to zkpool,proof is {:?},time consumed:{}",block,agg_proof_result,time_gap);

                    if need_send_proof(project_name.clone(), block).await {
                        let mut proofoutput =String::from("");
                        for var in &agg_proof_result.instance{
                            proofoutput=format!("{}#{}",proofoutput,var.to_string())
                        }
                        let proof_res = format!("{}#{}",proofoutput,agg_proof_result.proof);
                    
            
                        let message = StratumMessage::Submit(
                            Id::Num(0),
                            project_name.clone(),
                            block.to_string(),
                            proof_res,
                            agg_proof_result.k,
                            time_gap,
                        );
                        if let Err(error) = client.sender().send(message).await { 
                            error!("Failed to send PoolResponse: {}", error);
                        }else{
                            info!("zkpool:send the proof of block:{} success,time consumed:{}",block,time_gap);
                        }
                        info!("zkpool:end computed the task of block:{}",block);

                        let current_task = LATEST_TASK_CONTENT.clone();
                        let mut current_task_content = current_task.lock().await;
                        *current_task_content = String::from("");

                    }  
                });
    
                // cache the task handle
                let task_handle_vec = TASK_HANDLER.clone();
                let mut queue = task_handle_vec.lock().await;
                queue.push(task_handle);
            });
        }else{
            info!("ignore the unrecognized {} task",project_name);
        }
        info!("******one block task in process********");
    }
}


pub async fn need_send_proof(project:String,block:u64) -> bool {
    let current_task = LATEST_TASK_CONTENT.clone();
    let current_task_content = current_task.lock().await;
    if *current_task_content==String::from(""){ //initial
        return true
    }else {
        let task_vec: Vec<&str> = (*current_task_content).split("#").collect();
        if task_vec.len() != 2{
            return true
        }else {
            let project_now=task_vec[0];
            let cached_block = task_vec[1].parse::<u64>().unwrap();
            info!("cached task info is:{}+{}",project_now,cached_block);
            if project==project_now{
                if block>=cached_block{
                    return true
                }else {
                    return false
                }
            }else {
                return false
            }
        }
    }
}