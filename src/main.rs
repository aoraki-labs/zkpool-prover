extern crate core;

#[forbid(unsafe_code)]
mod client;
mod prover;

use std::{net::ToSocketAddrs, sync::Arc};

use clap::Parser;

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

#[tokio::main]
async fn main() {
     let yaml_str = include_str!("../app.yml");
     let prover_config: ProverConfig = serde_yaml::from_str(yaml_str)
         .expect("app.yaml read failed!");
    
     for i in 0..prover_config.name_list.len(){
        let one_project = ProjectInfo {
            name:prover_config.name_list[i].clone(),
            rpc_url:prover_config.rpc_url_list[i].clone(),
         }; 

         let pk_temp = PROJECT_LIST.clone();
         let mut pk_map = pk_temp.lock().await;
         pk_map.insert(prover_config.name_list[i].clone(), one_project);

     }
     
    let opt = Opt::parse();

    let tracing_level = if opt.debug {
        tracing::Level::DEBUG
    } else {
        tracing::Level::INFO
    };

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing_level)
        .finish();

    if let Some(log) = opt.log {
        let file = std::fs::File::create(log).unwrap();
        let file = tracing_subscriber::fmt::layer().with_writer(file).with_ansi(false);
        tracing::subscriber::set_global_default(subscriber.with(file))
            .expect("unable to set global default subscriber");
    } else {
        tracing::subscriber::set_global_default(subscriber).expect("unable to set global default subscriber");
    }   

    
    if opt.version {
        println!("0.1.1");
        std::process::exit(1);
    }

    let unique_id=match opt.unique_id{
        Some(r)=>r,
        None=>{
            machine_uid::get().unwrap()
        }
    };
    
    if opt.pool.is_none() {
        error!("Pool address is required!");
        std::process::exit(1);
    }
    if opt.access.is_none() {
        error!("Prover access key is required!");
        std::process::exit(1);
    }

    let access_key = opt.access.unwrap();
    let pool = opt.pool.unwrap();

    if let Err(e) = pool.to_socket_addrs() {
        error!("Invalid pool address {}: {}", pool, e);
        std::process::exit(1);
    }

    info!("Starting taiko prover:");

    let client = Client::init(access_key.clone(),unique_id, pool);

    let prover: Arc<Prover> = match Prover::init(client.clone()).await {
        Ok(prover) => prover,
        Err(e) => {
            error!("Unable to initialize prover: {}", e);
            std::process::exit(1);
        }
    };
    info!("Prover initialized");

    start(prover.sender(), client.clone()).await;

    std::future::pending::<()>().await;
}


