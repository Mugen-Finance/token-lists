use dotenv::*;
use ethers::{
    core::types::{Address, Filter, U256},
    prelude::abigen,
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Write, sync::Arc};
use tokio;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TokenInfo {
    address: Address,
    name: String,
    symbol: String,
    decimals: u8,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct PairInfo {
    token_1: Address,
    token_2: Address,
    fee: String,
    pair: Address,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VeloPair {
    token_1: Address,
    token_2: Address,
    pair: Address,
}

const V3FACTORY_ADDRESS: &str = "0x1F98431c8aD98523631AE4a59f267346ea31F984";
const CAMELOT_FACTORY_ADDRESS: &str = "0x6EcCab422D763aC031210895C81787E87B43A652";
const SUSHI_FACTORY_ADDRESS: &str = "0xc35DADB65012eC5796536bD9864eD8773aBc74C4";
const VELO_FACTORY_ADDRESS: &str = "0x25CbdDb98b35ab1FF77413456B31EC81A6B6B746";
abigen!(IERC20, "src/json/IERC20.json");

#[tokio::main]
async fn main() -> Result<()> {
    //let results = get_uniswap_pools().await?;
    //let result_2 = get_camelot_pools().await?;
    //let result_3 = get_sushi_pairs().await?;
    //_ = get_camelot_tokens().await?;
    //_ = get_uniswap_tokens().await?;
    //_ = get_velo_pools().await?;
    _ = get_velo_tokens().await?;

    Ok(())
}

async fn get_uniswap_pools() -> Result<()> {
    dotenv().ok();
    let url = env::var("ARBITRUM_RPC_URL").unwrap();
    let mut pairs: Vec<PairInfo> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(V3FACTORY_ADDRESS.parse::<Address>()?)
        .event("PoolCreated(address,address,uint24,int24,address)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;
    println!("{} pools found!", logs.iter().len());
    for log in logs {
        let token_0 = Address::from(log.topics[1]);
        let token_1 = Address::from(log.topics[2]);
        let fee_pair = U256::from_big_endian(&log.topics[3].as_bytes()[29..32]).to_string();
        let pair = Address::from(&log.data[44..64].try_into()?);

        pairs.push(PairInfo {
            token_1: token_0,
            token_2: token_1,
            fee: fee_pair,
            pair: pair,
        })
    }
    let result_5 =
        serde_json::to_writer(&File::create("src/UniswapPairs.json").unwrap(), &pairs).unwrap();
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CamelotPair {
    token_0: Address,
    token_1: Address,
    pair_address: Address,
}

async fn get_camelot_pools() -> Result<()> {
    dotenv().ok();
    let url = env::var("ARBITRUM_RPC_URL").unwrap();
    let provider = Provider::<Http>::try_from(url)?;
    let mut pairs: Vec<CamelotPair> = Vec::new();
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(CAMELOT_FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address,address,address,uint256)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;
    println!("{} pools found!", logs.iter().len());
    for log in logs {
        let token_0 = Address::from(log.topics[1]);
        let token_1 = Address::from(log.topics[2]);
        let pair = Address::from(&log.data[12..32].try_into()?);
        pairs.push(CamelotPair {
            token_0,
            token_1,
            pair_address: pair,
        });
    }
    let result_5 =
        serde_json::to_writer(&File::create("src/CamelotPairs.json").unwrap(), &pairs).unwrap();
    println!("{:#?}", pairs[10]);
    //println!("{:#?}", serialized_pairs);
    Ok(())
}

async fn get_sushi_pairs() -> Result<()> {
    dotenv().ok();
    let url = env::var("ARBITRUM_RPC_URL").unwrap();
    let mut pairs: Vec<CamelotPair> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(SUSHI_FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address,address,address,uint256)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;
    println!("{} pools found!", logs.iter().len());
    for log in logs {
        let token_0 = Address::from(log.topics[1]);
        let token_1 = Address::from(log.topics[2]);
        let pair = Address::from(&log.data[12..32].try_into()?);
        pairs.push(CamelotPair {
            token_0,
            token_1,
            pair_address: pair,
        });
    }
    let result_5 =
        serde_json::to_writer(&File::create("src/SushiPairs.json").unwrap(), &pairs).unwrap();

    Ok(())
}

async fn get_camelot_tokens() -> Result<()> {
    dotenv().ok();
    let url = env::var("ARBITRUM_RPC_URL").unwrap();
    let mut tokens: Vec<TokenInfo> = Vec::new();
    let mut added_tokens: Vec<Address> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(CAMELOT_FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address,address,address,uint256)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;
    for log in logs {
        let token_1 = Address::from(log.topics[1]);
        let token_2 = Address::from(log.topics[2]);
        if added_tokens.contains(&token_1) {
            continue;
        } else {
            let erc = IERC20::new(token_1, client.clone());
            let decimals = erc.decimals().call().await?;
            let name = erc.name().call().await?;
            let symbol = erc.symbol().call().await?;

            tokens.push(TokenInfo {
                address: token_1,
                name,
                symbol,
                decimals,
            });
            added_tokens.push(token_1);
        }
        if added_tokens.contains(&token_2) {
            continue;
        } else {
            let erc_2 = IERC20::new(token_2, client.clone());
            let decimals_2 = erc_2.decimals().call().await?;
            let name_2 = erc_2.name().call().await?;
            let symbol_2 = erc_2.symbol().call().await?;
            tokens.push(TokenInfo {
                address: token_2,
                name: name_2,
                symbol: symbol_2,
                decimals: decimals_2,
            });
            added_tokens.push(token_2);
        }
    }
    _ = serde_json::to_writer(
        &File::create("src/json/token_lists/CamelotTokens.json").unwrap(),
        &tokens,
    )
    .unwrap();
    Ok(())
}

async fn get_uniswap_tokens() -> Result<()> {
    dotenv().ok();
    let url = env::var("ARBITRUM_RPC_URL").unwrap();
    let contents = std::fs::read_to_string("src/json/token_lists/UniswapTokens.json")
        .expect("unable to read from file");
    let mut data: Vec<TokenInfo> = serde_json::from_str(contents.as_str()).unwrap();
    let mut added_tokens: Vec<Address> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(V3FACTORY_ADDRESS.parse::<Address>()?)
        .event("PoolCreated(address,address,uint24,int24,address)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;

    for log in &logs[1500..1700] {
        let token_1 = Address::from(log.topics[1]);
        let token_2 = Address::from(log.topics[2]);
        if added_tokens.contains(&token_1) || contents.contains(&token_1.to_string()) {
            continue;
        } else {
            let erc = IERC20::new(token_1, client.clone());
            let decimals = erc.decimals().call().await?;
            let name = erc.name().call().await?;
            let symbol = erc.symbol().call().await?;

            data.push(TokenInfo {
                address: token_1,
                name,
                symbol,
                decimals,
            });
            added_tokens.push(token_1);
        }
        if added_tokens.contains(&token_2) || contents.contains(&token_2.to_string()) {
            continue;
        } else {
            let erc_2 = IERC20::new(token_2, client.clone());
            let decimals_2 = erc_2.decimals().call().await?;
            let name_2 = erc_2.name().call().await?;
            let symbol_2 = erc_2.symbol().call().await?;
            data.push(TokenInfo {
                address: token_2,
                name: name_2,
                symbol: symbol_2,
                decimals: decimals_2,
            });
            added_tokens.push(token_2);
        }
    }
    let serialized_data = serde_json::to_string(&data);
    let mut file = File::create("src/json/token_lists/UniswapTokens.json")?;
    file.write_all(serialized_data.unwrap().as_bytes())?;
    Ok(())
}

async fn get_velo_pools() -> Result<()> {
    dotenv().ok();
    let url = env::var("OPTIMISM_RPC_URL").unwrap();
    let mut pairs: Vec<VeloPair> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(VELO_FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address,address,bool,address,uint256)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;
    println!("{} pools found!", logs.iter().len());
    for log in logs {
        let token_1 = Address::from(log.topics[1]);
        let token_2 = Address::from(log.topics[2]);
        //let stable = (&log.data[50..54]);
        let pair = Address::from(&log.data[44..64].try_into()?);
        pairs.push(VeloPair {
            token_1,
            token_2,
            pair: pair,
        });
    }
    let result_5 = serde_json::to_writer(
        &File::create("src/json/token_lists/optimism/VeloPairs.json").unwrap(),
        &pairs,
    )
    .unwrap();

    Ok(())
}

async fn get_velo_tokens() -> Result<()> {
    dotenv().ok();
    let url = env::var("OPTIMISM_RPC_URL").unwrap();
    let contents = std::fs::read_to_string("src/json/token_lists/optimism/VeloTokens.json")
        .expect("unable to read from file");
    let mut data: Vec<TokenInfo> = Vec::new();
    let mut added_tokens: Vec<Address> = Vec::new();
    let provider = Provider::<Http>::try_from(url)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(VELO_FACTORY_ADDRESS.parse::<Address>()?)
        .event("PairCreated(address,address,bool,address,uint256)")
        .from_block(0);
    let logs = client.get_logs(&filter).await?;

    for log in logs {
        let token_1 = Address::from(log.topics[1]);
        let token_2 = Address::from(log.topics[2]);
        if added_tokens.contains(&token_1) || contents.contains(&token_1.to_string()) {
            continue;
        } else {
            let erc = IERC20::new(token_1, client.clone());
            let decimals = erc.decimals().call().await?;
            let name = erc.name().call().await?;
            let symbol = erc.symbol().call().await?;

            data.push(TokenInfo {
                address: token_1,
                name,
                symbol,
                decimals,
            });
            added_tokens.push(token_1);
        }
        if added_tokens.contains(&token_2) || contents.contains(&token_2.to_string()) {
            continue;
        } else {
            let erc_2 = IERC20::new(token_2, client.clone());
            let decimals_2 = erc_2.decimals().call().await?;
            let name_2 = erc_2.name().call().await?;
            let symbol_2 = erc_2.symbol().call().await?;
            data.push(TokenInfo {
                address: token_2,
                name: name_2,
                symbol: symbol_2,
                decimals: decimals_2,
            });
            added_tokens.push(token_2);
        }
    }
    let serialized_data = serde_json::to_string(&data);
    let mut file = File::create("src/json/token_lists/optimism/VeloTokens.json")?;
    file.write_all(serialized_data.unwrap().as_bytes())?;
    Ok(())
}
