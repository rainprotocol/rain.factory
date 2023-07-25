
use std::str::FromStr;
use std::{convert::TryFrom, sync::Arc};
use anyhow::anyhow;
use ethers::prelude::SignerMiddleware;
use ethers::providers::{Provider, Http, Middleware} ;
use ethers::core::abi::Abi ;
use ethers::core::types::{Address,H160}; 
use ethers::contract::Contract;
use ethers::signers::LocalWallet;
use ethers::types::{Eip1559TransactionRequest, U64, Bytes, H256};


use crate::cli::deploy::{Deployer, RainContract};
use crate::deploy::deploy_contract;
use crate::deploy::dis::{DISpair, replace_dis_pair};
use crate::deploy::registry::RainNetworkOptions ;
use crate::deploy::transaction::get_transaction_data;
use crate::subgraph::get_transaction_hash;

use super::registry::{RainNetworks, Ethereum, Polygon, Mumbai, Fuji};

/// CLI function handler to cross deploy exxpression deployer
pub async fn expression_deployer(deployer_data: Deployer) -> anyhow::Result<()> {  

    let from_network = deployer_data.get_origin_network_details().unwrap() ;
    let to_network = deployer_data.get_target_network_details().unwrap() ; 

    let origin_deployer = match H160::from_str(&deployer_data.origin_deployer) {
        Ok(d) => d ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect Address Format Provided")) ;
        } 
    } ;

    let (interepreter_, store_) = get_interpreter_store(
        origin_deployer.clone(),from_network.clone()
    ).await.unwrap() ; 

    // Deploy Interpreter on Target Network
    let i_tx = get_transaction_hash(
        from_network.clone(),
        interepreter_.clone()
    ).await.unwrap() ;

    let i_data = get_transaction_data(
        from_network.clone(),
    i_tx).await.unwrap() ;  

    let (i_tx_hash, i_address) = deploy_contract(
        to_network.clone(),
        deployer_data.private_key.clone(),i_data
    ).await.unwrap() ; 

    let print_str = format!(
        "{}{}{}{}{}" ,
        String::from("\nInterpreter Deployed on target network !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(i_tx_hash.as_bytes().to_vec())), 
        String::from("\nContract Address: "),
        format!("0x{}",hex::encode(i_address.as_bytes().to_vec())),
        String::from("\n-----------------------------------\n")
    ) ; 
    println!(
        "{}",
        print_str
    ) ;

    // Deploy Store
    let s_tx = get_transaction_hash(from_network.clone(),store_.clone()).await.unwrap() ;
    let s_data = get_transaction_data(from_network.clone(), s_tx).await.unwrap() ;  

    let (s_tx_hash, s_address) = deploy_contract(to_network.clone(),deployer_data.private_key.clone(),s_data).await.unwrap() ; 

    let print_str = format!(
        "{}{}{}{}{}" ,
        String::from("\nStore Deployed on target network !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(s_tx_hash.as_bytes().to_vec())), 
        String::from("\nContract Address: "),
        format!("0x{}",hex::encode(s_address.as_bytes().to_vec())),
        String::from("\n-----------------------------------\n")
    ) ; 
    println!(
        "{}",
        print_str
    ) ;
 
    // Deploy Expression Deployer 
    let d_tx = get_transaction_hash(from_network.clone(),origin_deployer).await.unwrap() ;
    let d_data = get_transaction_data(from_network.clone(), d_tx).await.unwrap() ;   
    let d_data = replace_dis_pair(
        d_data ,
        DISpair{
            interpreter : Some(interepreter_),
            store : Some(store_),
            deployer : None
        } ,
        DISpair{
            interpreter : Some(i_address),
            store : Some(s_address),
            deployer : None
        } 
    ).unwrap() ; 

    let (d_tx_hash, d_address) = deploy_contract(to_network.clone(),deployer_data.private_key.clone(),d_data).await.unwrap() ;
    
    let print_str = format!(
        "{}{}{}{}{}" ,
        String::from("\nExpression Deployer deployed on target network !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(d_tx_hash.as_bytes().to_vec())), 
        String::from("\nContract Address: "),
        format!("0x{}",hex::encode(d_address.as_bytes().to_vec())),
        String::from("\n-----------------------------------\n")
    ) ; 
    println!(
        "{}",
        print_str
    ) ;

    Ok(())
} 

/// CLI function handler to cross deploy rain consumer contract
pub async fn rain_contract(contract: RainContract) -> anyhow::Result<()> {  
    
    // Get Origin Network Details
    let from_network = contract.get_origin_network_details().unwrap() ;

    // Get Target Network Details
    let to_network = contract.get_target_network_details().unwrap() ; 

    let contract_address = match H160::from_str(&contract.contract_address) {
        Ok(c) => c ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect Address Format Provided")) ;
        } 
    } ;
    
    // Check if transaction hash is provided
    let tx_hash = match contract.transaction_hash {
        Some(hash) => {
            match H256::from_str(&hash) {
                Ok(hash) => hash,
                Err(_) => {
                    return Err(anyhow!("\n ❌Incorrect Transaction Format Provided")) ;
                }
            }
        } ,
        None => {
            get_transaction_hash(from_network.clone(), contract_address.clone()).await?
        }
     } ;    

    let tx_data = get_transaction_data(from_network.clone(), tx_hash).await? ;

    // Get source IS 
    let source_deployer = match H160::from_str(&contract.origin_deployer.clone().unwrap()) {
        Ok(d) => d ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect Address Format for Origin Deployer")) ;
        } 
    } ;
    let (source_interpreter,source_store) = get_interpreter_store(
        source_deployer,
        from_network.clone()
    ).await.unwrap() ;

    // Get target IS 
    let target_deployer = match H160::from_str(&contract.target_deployer.clone().unwrap()) {
        Ok(d) => d ,
        Err(_) => {
            return Err(anyhow!("\n ❌Incorrect Address Format for Target Deployer")) ;
        } 
    } ;
    let (target_interpreter,target_store) = get_interpreter_store(
        target_deployer,
        to_network.clone()
    ).await.unwrap() ; 

    // Prepare Data
    let tx_data = replace_dis_pair(
        tx_data,
        DISpair { interpreter: Some(source_interpreter), store: Some(source_store), deployer: Some(source_deployer) },
        DISpair { interpreter: Some(target_interpreter), store: Some(target_store), deployer: Some(target_deployer) },
    ).unwrap() ; 

    // Deploy Contract
    let (contract_hash, contract_address) = deploy_contract(
        to_network.clone(),
        contract.private_key,tx_data
    ).await.unwrap() ; 

    let print_str = format!(
        "{}{}{}{}{}" ,
        String::from("\nContract Deployed on target network !!\n#################################\n✅ Hash : "),
        format!("0x{}",hex::encode(contract_hash.as_bytes().to_vec())), 
        String::from("\nContract Address: "),
        format!("0x{}",hex::encode(contract_address.as_bytes().to_vec())), 
        String::from("\n-----------------------------------\n")
    ) ; 
    println!(
        "{}",
        print_str
    ) ; 
     
    Ok(())
}  

/// Function to get Rainterpreter and RainterpreterStore corresponding to deployer
pub async fn get_interpreter_store(
    deployer_address: H160 ,
    network : RainNetworks
) -> anyhow::Result<(H160,H160)> { 


    let provider_url = match network {
        RainNetworks::Ethereum(network) => {
            network.rpc_url
        },
        RainNetworks::Polygon(network) => {
            network.rpc_url
        }
        RainNetworks::Mumbai(network) => {
            network.rpc_url
        }
        RainNetworks::Fuji(network) => {
            network.rpc_url
        }
    } ;  
 
    let abi: Abi= serde_json::from_str(r#"[{"inputs":[],"name":"store","outputs":[{"internalType":"contract IInterpreterStoreV1","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"interpreter","outputs":[{"internalType":"contract IInterpreterV1","name":"","type":"address"}],"stateMutability":"view","type":"function"}]"#)?;

    // connect to the network
    let client = Provider::<Http>::try_from(provider_url).unwrap();

    // create the contract object at the address
    let contract = Contract::new(deployer_address, abi, Arc::new(client));  
    
    let store: H160 = contract.method::<_, H160>("store", ())?.call().await? ; 
    let intepreter: H160 = contract.method::<_, H160>("interpreter", ())?.call().await? ;  

    Ok((intepreter,store))

} 
