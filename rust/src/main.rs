#![allow(unused)]
use bitcoin::hex::DisplayHex;
use bitcoincore_rpc::bitcoin::Amount;
use bitcoincore_rpc::{Auth, Client, RpcApi};
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

// Node access params
const RPC_URL: &str = "http://127.0.0.1:18443"; // Default regtest RPC port
const RPC_USER: &str = "alice";
const RPC_PASS: &str = "password";

// You can use calls not provided in RPC lib API using the generic `call` function.
// An example of using the `send` RPC call, which doesn't have exposed API.
// You can also use serde_json `Deserialize` derivation to capture the returned json result.
fn send(rpc: &Client, addr: &str) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : 100 }]), // recipient address
        json!(null),            // conf target
        json!(null),            // estimate mode
        json!(null),            // fee rate in sats/vb
        json!(null),            // Empty option object
    ];

    #[derive(Deserialize)]
    struct SendResult {
        complete: bool,
        txid: String,
    }
    let send_result = rpc.call::<SendResult>("send", &args)?;
    assert!(send_result.complete);
    Ok(send_result.txid)
}

fn main() -> bitcoincore_rpc::Result<()> {
    // Connect to Bitcoin Core RPC
    let rpc = Client::new(
        RPC_URL,
        Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()),
    )?;

    // Get blockchain info
    let blockchain_info = rpc.get_blockchain_info()?;
    // println!("Blockchain Info: {:?}", blockchain_info);
    println!("Connected to Bitcoin Core. Chain: {}", blockchain_info.chain);

    // Create/Load the wallets, named 'Miner' and 'Trader'. Have logic to optionally create/load them if they do not exist or not loaded already.
        let miner_wallet = match rpc.create_wallet("Miner", None, None, None, None)
        {
            Ok(_)=> {
                println!("Created Miner wallet");
                Client::new(&format!("{}/wallet/Miner",RPC_URL), Auth::UserPass(RPC_USER.to_owned(),RPC_PASS.to_owned()))?
            }
            Err(_) => {
                //wallet exists, try to load it
                let _ = rpc.load_wallet("Miner");
                println!("Loaded existing Miner wallet");
                Client::new(&format!("{}/wallet/Miner",RPC_URL), Auth::UserPass(RPC_USER.to_owned(),RPC_PASS.to_owned()))?
            }
        };

        //load the Trader wallet

        let trader_wallet = match rpc.create_wallet("Trader", None, None,None,None){
            Ok(_) =>{
                println!("Created Trader wallet");
                Client::new(&format!("{}/wallet/Trader", RPC_URL),Auth::UserPass(RPC_USER.to_owned(), RPC_PASS.to_owned()))?
            }
            Err(_)=>{
                //wallet exists, try to load it
                let _ = rpc.load_wallet("Trader");
                println!("Loaded existing Trader wallet");
                Client::new(&format!("{}/wallet/Trader",RPC_URL), Auth::UserPass(RPC_USER.to_owned(),RPC_PASS.to_owned()))?
            }
        };

    println!("Both wallets ready");

    //Generate new address in Miner wallet with label "Mining Reward"

    let mining_address = miner_wallet.get_new_address(Some("Mining Reward"), None)?;
    println!("Mining address:{}",mining_address);

    //Mine 101 blocks to the mining address
    //Blocks rewards in bitcoin require 100 confirmations(Maturity period)
    //before they become spendable
    println!("Mining 101 blocks...");
    let _blocks = miner_wallet.generate_to_address(101, &mining_address)?;
    println!("Mined 101 blocks");
    
    //Get and print Miner wallet balance

    let balance = miner_wallet.get_balance(None, None)?;
    println!("Miner wallet balance:{}BTC", balance.to_btc());

    Ok(())
}
