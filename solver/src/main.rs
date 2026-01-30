use std::collections::HashMap;
use num_bigint::BigUint;
use anyhow::Result;
use futures::StreamExt;

use tycho_simulation::tycho_common::{
    models::{token::Token, Chain},
    simulation::protocol_sim::ProtocolSim,
};
use tycho_simulation::protocol::models::ProtocolComponent;
use tycho_simulation::utils::load_all_tokens;
use tycho_simulation::evm::stream::ProtocolStreamBuilder;
use tycho_simulation::evm::protocol::{
    uniswap_v2::state::UniswapV2State,
    uniswap_v3::state::UniswapV3State,
};
use tycho_simulation::tycho_client::feed::component_tracker::ComponentFilter;

type PoolId = String;
const TYCHO_HOST: &str = "tycho-beta.propellerheads.xyz";

fn addr(s: &str) -> String { s.to_lowercase() }

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = std::env::var("TYCHO_API_KEY")
        .unwrap_or_else(|_| "63f27794-59dd-4dd9-80b8-0bb54f6b06c0".to_string());

    // Switch to WETH and USDC - The most liquid direct pair on Ethereum
    let t_in_addr = addr("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"); // WETH
    let t_out_addr = addr("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"); // USDC

    // Step 1: Loading tokens
    println!("Step 1: Loading tokens...");
    let token_data_res = load_all_tokens(
        TYCHO_HOST, false, Some(&api_key), false, Chain::Ethereum, None, None
    ).await;
    
    let token_data_map = token_data_res.expect("Failed to load tokens");
    let mut token_lookup: HashMap<String, Token> = HashMap::new();
    for t in token_data_map.values() {
        token_lookup.insert(t.address.to_string().to_lowercase(), t.clone());
    }

    // Step 2: Sync Snapshot (Minimum filter to ensure WETH/USDC is captured)
    println!("Step 2: Syncing pools...");
    // 0.1 threshold to catch almost all active pools without being too heavy
    let filter = ComponentFilter::with_tvl_range(0.1, 100.0); 

    let mut stream = ProtocolStreamBuilder::new(TYCHO_HOST, Chain::Ethereum)
        .exchange::<UniswapV2State>("uniswap_v2", filter.clone(), None)
        .exchange::<UniswapV3State>("uniswap_v3", filter.clone(), None)
        .auth_key(Some(api_key))
        .set_tokens(token_data_map) 
        .await
        .build()
        .await?;

    let mut components: HashMap<PoolId, ProtocolComponent> = HashMap::new();
    let mut states: HashMap<PoolId, Box<dyn ProtocolSim>> = HashMap::new();

    if let Some(msg) = stream.next().await {
        let message = msg?;
        for (id, comp) in message.new_pairs.iter() { components.insert(id.clone(), comp.clone()); }
        for (id, st) in message.states.iter() { states.insert(id.clone(), st.clone_box()); }
    }
    println!("Step 2: Synced {} total pools.", states.len());

    // Debug check
    let mut match_count = 0;
    for (pid, meta) in components.iter() {
        let pool_tokens: Vec<String> = meta.tokens.iter()
            .map(|t| t.address.to_string().to_lowercase()).collect();
        if pool_tokens.contains(&t_in_addr) && pool_tokens.contains(&t_out_addr) {
            match_count += 1;
        }
    }
    println!("   MATCH FOUND: {} pools contain WETH/USDC.", match_count);

    // Step 3: Greedy Simulation
    println!("Step 3: Simulating 100 chunks for 100 WETH...");
    let total_in = BigUint::from(100u64) * BigUint::from(10u64).pow(18); 
    let chunk_in = &total_in / 100u64;
    let mut total_out = BigUint::from(0u64);
    let mut successful_chunks = 0;

    for _i in 0..100 {
        let mut b_out = BigUint::from(0u64);
        let mut b_pool: Option<PoolId> = None;
        let mut b_st: Option<Box<dyn ProtocolSim>> = None;

        for (pid, st) in states.iter() {
            if let Some(meta) = components.get(pid) {
                let pool_tokens: Vec<String> = meta.tokens.iter()
                    .map(|t| t.address.to_string().to_lowercase()).collect();
                
                if pool_tokens.contains(&t_in_addr) && pool_tokens.contains(&t_out_addr) {
                    if let (Some(tin), Some(tout)) = (token_lookup.get(&t_in_addr), token_lookup.get(&t_out_addr)) {
                        // Simulation of swap
                        if let Ok(res) = st.get_amount_out(chunk_in.clone(), tin, tout) {
                            if res.amount > b_out {
                                b_out = res.amount.clone();
                                b_pool = Some(pid.clone());
                                b_st = Some(res.new_state); 
                            }
                        }
                    }
                }
            }
        }

        if let (Some(pid), Some(new_st)) = (b_pool, b_st) {
            states.insert(pid, new_st); // Apply state update for greedy routing
            total_out += b_out;
            successful_chunks += 1;
        }
    }

    println!("\n================ FINAL REPORT ================");
    if successful_chunks > 0 {
        println!("Status: SUCCESS");
        println!("Pair: WETH / USDC");
        println!("Chunks Matched: {}/100", successful_chunks);
        println!("Total USDC Received: {}", total_out);
    } else {
        println!("Status: FAILED. No direct WETH/USDC paths in these 586 pools.");
    }
    println!("==============================================");

    Ok(())
}