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
fn send(rpc: &Client, addr: &str, amount:f64) -> bitcoincore_rpc::Result<String> {
    let args = [
        json!([{addr : amount }]), // recipient address
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

//Create a trader address and send 20 BTC from miner to Trader
//Generate new address in Trader wallet with label "Received"

let trader_address = trader_wallet.get_new_address(Some("Received"),None)?;
println!("Trader receiving address:{}", trader_address);

//Send 20 BTC from Miner to Trader

let txid = send(&miner_wallet, &trader_address.to_string(), 20.0)?;


println!("Transaction sent! TxID: {}", txid);

//Query mempool and confirm transaction

//Fetch unconfirmed transactions

println!("\nFetching transaction from mempool...");
let mempool_entry: serde_json::Value = miner_wallet.call("getmempoolentry", &[json!(txid)])?;
println!("Mempool entry: {:?}", mempool_entry);
//Mine 1 block to confirm the transaction
println!("\nMining 1 block to confirm transaction...");
let _confirm_blocks = miner_wallet.generate_to_address(1, &mining_address)?;
println!("Transaction confirmed");

 // Extract transaction details and write to out.txt
    let tx_details: serde_json::Value = miner_wallet.call("gettransaction", &[json!(txid), json!(null), json!(true)])?;
    
 // Extract basic info
    let decoded = &tx_details["decoded"];
    let block_hash = tx_details["blockhash"].as_str().unwrap();
    let block_height = tx_details["blockheight"].as_u64().unwrap();
    let fee = tx_details["fee"].as_f64().unwrap().abs();
    
    // Extract input details (vin)
    let vin = &decoded["vin"][0];
    let input_txid = vin["txid"].as_str().unwrap();
    let input_vout = vin["vout"].as_u64().unwrap();
    
    
// Get the previous transaction to find input address and amount
    let prev_tx: serde_json::Value = miner_wallet.call("gettransaction", &[json!(input_txid), json!(null), json!(true)])?;
    let prev_vout = &prev_tx["decoded"]["vout"][input_vout as usize];
    let miner_input_address = prev_vout["scriptPubKey"]["address"].as_str().unwrap();
    let miner_input_amount = prev_vout["value"].as_f64().unwrap();
    
    // Extract output details (vout)
    let vout = &decoded["vout"];
    
    // Identify trader output vs change output by matching addresses
    let mut trader_output_address = "";
    let mut trader_output_amount = 0.0;
    let mut miner_change_address = "";
    let mut miner_change_amount = 0.0;
    
 for output in vout.as_array().unwrap() {
    let addr = output["scriptPubKey"]["address"].as_str().unwrap();
    let amount = output["value"].as_f64().unwrap();
        
        if addr == trader_address.to_string() {
            trader_output_address = addr;
            trader_output_amount = amount;
        } else {
            miner_change_address = addr;
            miner_change_amount = amount;
        }
    }
    
    // Write to out.txt 
    let mut file = File::create("../out.txt")?;
    writeln!(file, "{}", txid)?;
    writeln!(file, "{}", miner_input_address)?;
    writeln!(file, "{}", miner_input_amount)?;
    writeln!(file, "{}", trader_output_address)?;
    writeln!(file, "{}", trader_output_amount)?;
    writeln!(file, "{}", miner_change_address)?;
    writeln!(file, "{}", miner_change_amount)?;
    writeln!(file, "{}", fee)?;
    writeln!(file, "{}", block_height)?;
    writeln!(file, "{}", block_hash)?;
    
    println!("\nTransaction details written to out.txt");



    Ok(())
}
