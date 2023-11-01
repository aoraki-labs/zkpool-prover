extern crate core;

#[forbid(unsafe_code)]
mod client;
mod prover;

use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

use std::{net::ToSocketAddrs, sync::Arc};

use clap::Parser;

use ::prover::shared_state::generate_proof;
use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;

use crate::prover::ProjectInfo;

use machine_uid;

use crate::{
    client::{start, Client},
    prover::Prover,
};

use crate::prover::PROJECT_LIST;

extern crate serde_yaml;
extern crate serde;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProverConfig {
    name_list: Vec<String>,
    rpc_url_list: Vec<String>,
}

#[derive(Debug, Parser)]
#[clap(name = "prover", about = "Standalone prover.")]
struct Opt {
    /// Enable debug logging
    #[clap(short = 'd', long = "debug")]
    debug: bool,

    /// Enable get version
    #[clap(short = 'v', long = "version")]
    version: bool,

    /// Prover access key (...)
    #[clap(short = 'k', long = "access_key")]
    access: Option<String>,

     /// Prover device id (...)
     #[clap(short = 'u', long = "uuid")]
     unique_id: Option<String>,

    /// Pool server address
    #[clap(short = 'p', long = "pool")]
    pool: Option<String>,

    /// Number of threads
    #[clap(short = 't', long = "threads")]
    threads: Option<u16>,

    /// Output log to file
    #[clap(short = 'o', long = "log")]
    log: Option<String>,
}

use std::time::Instant;
use tokio::{self, runtime::Runtime, time};

#[tokio::main]
async fn main() {


    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("unable to set global default subscriber");

    info!("*************TEST***********");
    // loop{
        let task_handle = task::spawn(async move {
            let time_started = Instant::now();
            let test = generate_proof
            ("https://rpc.jolnir.taiko.xyz/".to_string(),
            20865 as u64,
            "94061Fd498291Ff1F1b8C0d1a94e2EDC2a0A2f9D".to_string(),
            "cD5e2bebd3DfE46e4BF96aE2ac7B89B22cc6a982".to_string(),
            "1000777700000000000000000000000000000007".to_string(),
            "1000777700000000000000000000000000000001".to_string(),
            "322e41c411a8223cce152999b30ee00b8f29dc5e62e02f43e0dc7a77aa862fa8".to_string(),
            "c73622fae1fbc1d1d9e4a9b7bbdb6733595c1c98a2470ea59ca3b9fee9ba3894".to_string(),
            "afcb03ea890fb2d5ba0042fcda321d8879687fb87a8d68b8ef4417dbc86754b0".to_string(),
            "9cc94396d73d6c51d8185249a1bcc7c55c87b3d6b67ce72600cfc8448dadc007".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            1241987 as u64,
            328517 as u64,
            8000000, 
            0,
            120000,
            )
            .await.unwrap();
        let time_gap =(Instant::now().duration_since(time_started).as_millis() as u32)/1000;
        println!("testcase 20865:proof result is {},time consumed:{}s",format!("{}",test.proof),time_gap);
    });
    
        info!("*************SLEEP***********");
        time::sleep(time::Duration::from_secs(400)).await; //estimated time
    
        info!("*************cancel the above task***********");
        task_handle.abort(); 
        drop(task_handle);
    
        info!("*************compute another one task agin***********");
        let time_started = Instant::now();
        let test = generate_proof
            ("https://rpc.jolnir.taiko.xyz/".to_string(),
            942261 as u64,
            "2909Db987AA74120a15f743197c58bE1B8D5e83b".to_string(),
            "cD5e2bebd3DfE46e4BF96aE2ac7B89B22cc6a982".to_string(),
            "1000777700000000000000000000000000000007".to_string(),
            "1000777700000000000000000000000000000001".to_string(),
            "73656a6ac138308428f6bb3fc55f2180f38e396c951d0069629b876456a8d564".to_string(),
            "f1f4d8c9deac229fcf7f62b9c4940648248564dbcdf7402e0b47cbcfafd38307".to_string(),
            "cb7e54df5335fcab508c38ba3954835794368cd81c02d1ce1f345e50c36a1b02".to_string(),
            "4a611f97b71b197b26ff47e1844f7db878afb5bee3d711b0c9d33c0aee6e986b".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            127844 as u64,
            127844 as u64,
            8000000, 
            0,
            120000,
        )
        .await.unwrap();
        let time_gap =(Instant::now().duration_since(time_started).as_millis() as u32)/1000;
        println!("testcase 942261:proof result is {},time consumed:{}s",format!("{}",test.proof),time_gap);
    // }
}


