use std::{
    sync::{
        atomic::{Ordering,AtomicUsize},
        Arc,
    }, time::Instant, thread,
};
use std::collections::VecDeque;
use aleo_stratum::message::StratumMessage;
use anyhow::Result;
use json_rpc_types::Id;
use rand::{thread_rng, RngCore};

use snarkvm::{prelude::{Testnet3, coinbase::{PuzzleConfig, CoinbasePuzzle}}, utilities::ToBytes};
use snarkvm::prelude::Address;
use snarkvm::prelude::FromBytes;
use snarkvm::prelude::UniversalSRS;
use snarkvm::prelude::coinbase::EpochChallenge;

use tokio::{
    sync::mpsc,
    task,
};
use tracing::{debug, error, info};
use hex::FromHex;

use crate::Client;

use parking_lot:: RwLock;
use core::time::Duration;
const BATCH_SIZE: usize = 256;

use lazy_static::lazy_static;

use std::sync::Mutex;


lazy_static! {
    pub static ref PROOF_DATA: Arc<Mutex<VecDeque<Proofdata>>> = Arc::new(Mutex::new(VecDeque::new()));
}

#[derive(Debug,Clone)]
pub struct Proofdata{
    job_id:String,
    proof_hex:String,
}

#[derive(Clone)]
pub struct Prover {
    sender: Arc<mpsc::Sender<ProverEvent>>,
    client: Arc<Client>,
    coinbase_puzzle: CoinbasePuzzle<Testnet3>,

    //add by zkpool
    latest_epoch_number: Arc<RwLock<Option<u64>>>,
    latest_difficulty: Arc<RwLock<Option<u64>>>,
    latest_address: Arc<RwLock<Option<String>>>,
    latest_job_id: Arc<RwLock<Option<String>>>,
    latest_nonce_prefix:Arc<RwLock<Option<String>>>,
    latest_epoch_challenge:Arc<RwLock<Option<EpochChallenge<Testnet3>>>>,
}

#[allow(clippy::large_enum_variant)]
pub enum ProverEvent {
    //difficulty, epoch_number, epoch_challenge, address, job_id, nonce_prefix
    NewWork(u64, u64, EpochChallenge<Testnet3>, String, String,String),
    Result(bool, Option<String>),
}

static CURRENT_PROVES_COMM: AtomicUsize = AtomicUsize::new(0);

impl Prover {
    pub async fn init(
        client: Arc<Client>,
    ) -> Result<Arc<Self>> {
     
        let (sender, mut receiver) = mpsc::channel(1024);

        info!("Initializing universal SRS");
        let coinbase_puzzle = CoinbasePuzzle::<Testnet3>::load()?;
        info!("Coinbase proving key initialized");

        let prover = Arc::new(Self {
            sender: Arc::new(sender),
            client,
            coinbase_puzzle,

            latest_epoch_number: Default::default(),
            latest_difficulty: Default::default(),
            latest_address: Default::default(),
            latest_job_id: Default::default(),
            latest_nonce_prefix:Default::default(),
            latest_epoch_challenge:Default::default(),
        });

        let p = prover.clone();
        let _ = task::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    ProverEvent::NewWork(difficulty,epoch_number, epoch_challenge, address, job_id, nonce_prefix) => {
                        info!("Received new work: epoch {}, difficulty {},address {},job id {},nonce_prefix {}", epoch_number, difficulty,address.clone(),job_id.clone(),nonce_prefix.clone());
                        p.latest_epoch_number.write().replace(epoch_number);
                        p.latest_difficulty.write().replace(difficulty);
                        p.latest_address.write().replace(address);
                        p.latest_job_id.write().replace(job_id.clone());
                        p.latest_nonce_prefix.write().replace(nonce_prefix.clone());
                        p.latest_epoch_challenge.write().replace(epoch_challenge);
                    }
                    ProverEvent::Result(success, error) => {
                        p.result(success, error).await;
                    }
                }
            }
        });
        debug!("Created prover message handler");

        let p = prover.clone();
        let client = p.client.clone();
        let _ = task::spawn(async move {
            loop{
                let mut queue = PROOF_DATA.lock().unwrap().clone();
                for i in 0..queue.len(){
                    match queue.pop_front(){
                        Some(r) => {
                            let message = StratumMessage::Submit(
                                Id::Num(0),
                                r.job_id,
                                r.proof_hex,
                            );
                            if let Err(error) = client.sender().send(message).await {
                                error!("Failed to send PoolResponse: {}", error);
                            }
                        },
                        None => {
                            continue;
                        },
                   }
                    p.delete(i).await;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
         }
        });
        debug!("Created prover proof handler");

        prover.initialize_coinbase_puzzle().await;

        Ok(prover)
    }

    pub fn sender(&self) -> Arc<mpsc::Sender<ProverEvent>> {
        self.sender.clone()
    }

    async fn initialize_coinbase_puzzle(&self) {
        let cpu_num = num_cpus::get().saturating_sub(2).clamp(1, 6);
        let speed = Instant::now();
        info!("initialize_coinbase_puzzle finished with {} CPU",cpu_num);
        for i in 0..cpu_num {
            let prover = self.clone();
            let i: usize = i as usize % cpu_num;
            info!("start to compute the coinbase puzzle of thread {}",i);
            rayon::spawn(move || prover.coinbase_puzzle_loop());
        }

        rayon::spawn(move || {
            let mut prev_elapsed = 0f32;
            let mut prev_proofs = 0;
            loop {
                let elapsed = speed.elapsed().as_secs_f32();
                let secs = elapsed - prev_elapsed;
                if secs > 10f32 {
                    let proves_count = CURRENT_PROVES_COMM.load(Ordering::SeqCst) * BATCH_SIZE; 
                    info!(
                        "total speed: {} h/s; last 10s: {}  h/s",
                        proves_count / elapsed as usize,
                        (proves_count - prev_proofs) / secs as usize
                    );
                    prev_elapsed = elapsed;
                    prev_proofs = proves_count;
                } else {
                    thread::sleep(Duration::from_millis(20));
                }
            }
        });
    }

    async fn delete(&self,index:usize) {
        let mut w = PROOF_DATA.lock().unwrap();
         w.remove(index);
         
    }

    fn coinbase_puzzle_loop(&self) {
        // init_stream_ctx_safe(device_id);
        loop {
            let latest_epoch_number = match self.latest_epoch_number.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };
            let latest_difficulty = match self.latest_difficulty.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };

            let latest_address = match self.latest_address.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };
            let latest_job_id = match self.latest_job_id.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };
            let latest_nonce_prefix = match self.latest_nonce_prefix.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };
            let latest_epoch_challenge = match self.latest_epoch_challenge.read().clone(){
                Some(r) => r,
                None => {
                    continue;
                },
            };

            debug!("latest_epoch_number is:{},latest_difficulty/proof target is :{},latest_address is:{},latest_job_id is:{},latest_nonce_prefix is:{},",
                latest_epoch_number,
                latest_difficulty,
                latest_address,
                latest_job_id,
                latest_nonce_prefix,
            );
            let my_address = match hex::decode(latest_address.clone()){
                Ok(r) => r,
                Err(_) => {
                    continue;
                },
            };

            let nonce = thread_rng().next_u64();
            let string_to_bytes = <[u8;2]>::from_hex(latest_nonce_prefix).unwrap();
            let my_prefix:u64 = u64::from(u16::from_be_bytes(string_to_bytes));
            let nonce: u64 = (my_prefix << 48) | (nonce >> 16);


            CURRENT_PROVES_COMM.fetch_add(1, Ordering::SeqCst);
            let proof_result = match self.coinbase_puzzle.prove(
            &latest_epoch_challenge,
            Address::read_le(&my_address[..]).unwrap(),
            nonce,
            Some(latest_difficulty),
            ){
                Ok(r)=>r,
                Err(_e)=>{
                    continue
               }
            };
            info!("found one solution:{:?},send it to zkpool",hex::encode(proof_result.to_bytes_le().unwrap()));

            let proof: Proofdata = Proofdata { 
                job_id:latest_job_id.clone(),
                proof_hex:hex::encode(proof_result.to_bytes_le().unwrap()),
             } ;


            let mut w = PROOF_DATA.lock().unwrap();
            w.push_back(proof);
        }
        // deinit_stream_ctx_safe(device_id);
     }

    async fn result(&self, success: bool, msg: Option<String>) {
        debug!("ignore the msg:{},{:?}",success,msg);
    }
}

