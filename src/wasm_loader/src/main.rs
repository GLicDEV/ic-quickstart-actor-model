use ic_agent::identity::AnonymousIdentity;
use sha2::{Digest, Sha256};
use std::{fs::File, io::Read, path::PathBuf};

use candid::Encode;

use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::Agent;
use ic_types::Principal;
use ic_utils::Canister;

use clap::Parser;

#[tokio::main]
async fn main() {
    let mut dev_mode = true;
    let opts = Opts::parse();

    let wasm = read_file_from_local_bin(&opts.wasm_path);

    if opts.url.contains("ic0.app") {
        dev_mode = false;
    }

    let transport = ReqwestHttpReplicaV2Transport::create(opts.url)
        .expect("Failed to create Reqwest transport");
    let timeout = std::time::Duration::from_secs(60 * 5);

    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(AnonymousIdentity)
        .with_ingress_expiry(Some(timeout))
        .build()
        .expect("Failed to build agent");

    if dev_mode {
        agent
            .fetch_root_key()
            .await
            .expect("Couldn't fetch root key");
    }

    let canister = Canister::builder()
        .with_agent(&agent)
        .with_canister_id(opts.canister_id)
        .build()
        .unwrap();

    let waiter = garcon::Delay::builder()
        .throttle(std::time::Duration::from_millis(500))
        .timeout(std::time::Duration::from_secs(60 * 5))
        .build();

    let (response,) = canister
        .update_("load_wasm")
        .with_arg_raw(Encode!(&wasm).unwrap())
        .build::<(bool,)>()
        .call_and_wait(waiter)
        .await
        .unwrap();

    // let response = agent
    //     .update(&opts.canister_id, "load_wasm")
    //     .with_arg(&Encode!(&wasm).unwrap())
    //     .call()
    //     .await
    //     .unwrap();

    // let result = Decode!(response.as_slice(), (bool,)).unwrap();

    println!("response: {:?}", response);
}

#[derive(Parser)]
struct Opts {
    url: String,
    canister_id: Principal,
    wasm_path: String,
}

pub fn read_file_from_local_bin(file_name: &str) -> Vec<u8> {
    let mut file_path = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("Failed to read CARGO_MANIFEST_DIR env variable"),
    );

    file_path.push(&file_name);

    let mut file = File::open(&file_path)
        .unwrap_or_else(|_| panic!("Failed to open file: {}", file_path.to_str().unwrap()));
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("Failed to read file");

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();

    println!(
        "Loaded a wasm file with the following checksum: {:x}",
        result
    );

    bytes
}
