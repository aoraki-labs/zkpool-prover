extern crate core;

#[forbid(unsafe_code)]
mod client;
mod prover;

use std::{net::ToSocketAddrs, sync::Arc};

use clap::Parser;
use tracing::{debug, error, info};
use tracing_subscriber::layer::SubscriberExt;

use crate::{
    client::{start, Client},
    prover::Prover,
};

use machine_uid;

#[derive(Debug, Parser)]
#[clap(name = "zkpool-aleo-prover", about = "zkpool-aleo-prover.")]
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

    /// Output log to file
    #[clap(short = 'o', long = "log")]
    log: Option<String>,
}

#[tokio::main]
async fn main() {

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
        println!("0.1.0");
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

    info!("Starting zkpool aleo prover");

    let client = Client::init(access_key.clone(),unique_id, pool);

    let prover: Arc<Prover> = match Prover::init(client.clone()).await {
        Ok(prover) => prover,
        Err(e) => {
            error!("Unable to initialize prover: {}", e);
            std::process::exit(1);
        }
    };
    debug!("Zkpool Aleo Prover initialized");

    start(prover.sender(), client.clone());

    std::future::pending::<()>().await;
}


