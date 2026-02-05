use std::collections::{HashMap, HashSet};
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

    // 17 个关注代币的地址列表
    let token_addresses: HashSet<String> = vec![
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", // USDC
        "0xdac17f958d2ee523a2206206994597c13d831ec7", // USDT
        "0x6b175474e89094c44da98b954eedeac495271d0f", // DAI
        "0xf1c9acdc66974dfb6decb12aa385b9cd01190e38", // osETH
        "0x2260fac5e5542a773aa44fbcfedf7c193bc2c599", // WBTC
        "0x40d16fc0246ad3160ccc09b8d0d3a2cd28ae6c2f", // GHO
        "0x4c9edd5852cd905f086c759e8383e09bff1e68b3", // USDe
        "0x8d0d000ee44948fc98c9b98a4fa4921476f08b0d", // DOLA
        "0x6c3ea9036406852006290770bedfcaba0e23a0e8", // PYUSD
        "0xfa2b947eec368f42195f24f36d2af29f7c24cec2", // FRAX
        "0xf939e0a03fb07f59a73314e73794be0e57ac1b4e", // crvUSD
        "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2", // WETH
        "0xae7ab96520de3a18e5e111b5eaab095312d7fe84", // stETH
        "0x7f39c581f595b53c5cb19bd0b3f8da6c935e2ca0", // wstETH
        "0xcd5fe23c85820f7b72d0926fc9b05b43e359b7ee", // weETH
        "0xae78736cd615f374d3085123a210448e74fc6393", // rETH
        "0x514910771af9ca656af840dff83e8264ecf986ca", // LINK
    ].into_iter().map(addr).collect();

    println!("Step 1: Loading token metadata...");
    let token_data_map = load_all_tokens(TYCHO_HOST, false, Some(&api_key), false, Chain::Ethereum, None, None)
        .await
        .expect("Failed to load tokens");

    let mut token_lookup: HashMap<String, Token> = HashMap::new();
    for t in token_data_map.values() {
        let a = t.address.to_string().to_lowercase();
        if token_addresses.contains(&a) {
            token_lookup.insert(a, t.clone());
        }
    }

    println!("Step 2: Syncing pools for {} tokens...", token_addresses.len());
    let filter = ComponentFilter::with_tvl_range(0.0, 100.0); 

    // 修复点：先获取 builder 实例，再进行链式调用
    let mut builder = ProtocolStreamBuilder::new(TYCHO_HOST, Chain::Ethereum).await;
    builder = builder
        .auth_key(Some(api_key))
        .set_tokens(token_data_map)
        .exchange::<UniswapV2State>("uniswap_v2", filter.clone(), None)
        .exchange::<UniswapV2State>("sushi_v2", filter.clone(), None)
        .exchange::<UniswapV3State>("uniswap_v3", filter.clone(), None);

    let mut stream = builder.build().await?;

    let mut components: HashMap<PoolId, ProtocolComponent> = HashMap::new();
    let mut states: HashMap<PoolId, Box<dyn ProtocolSim>> = HashMap::new();

    if let Some(msg) = stream.next().await {
        let message = msg?;
        for (id, comp) in message.new_pairs.iter() {
            // 只要池子包含列表中任何一个代币就抓取，确保全面性
            let is_relevant = comp.tokens.iter().any(|t| token_addresses.contains(&t.address.to_string().to_lowercase()));
            if is_relevant {
                components.insert(id.clone(), comp.clone());
            }
        }
        for (id, st) in message.states.iter() {
            if components.contains_key(id) {
                states.insert(id.clone(), st.clone_box());
            }
        }
    }
    println!("Synced {} relevant pools.", states.len());

    // 设定测试参数：100 osETH -> USDC
    let t_in_addr = addr("0xf1c9acdc66974dfb6decb12aa385b9cd01190e38");
    let t_out_addr = addr("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
    
    let t_in = token_lookup.get(&t_in_addr).expect("Token In not found");
    let t_out = token_lookup.get(&t_out_addr).expect("Token Out not found");
    
    // 以 $100K 为例测试
    let total_amount = BigUint::from(100u64) * BigUint::from(10u64).pow(18); 

    // Step 3: 贪心叠加模拟（1% 步进）
    println!("Step 3: Simulating 100 steps of 1% each...");
    let n_steps = 100;
    let chunk_size = &total_amount / n_steps as u64;
    let mut current_total_out = BigUint::from(0u64);

    for step in 1..=n_steps {
        let (mut best_chunk_out, mut best_pool_id, mut best_new_state) = (BigUint::from(0u64), None, None);

        for (pid, state) in states.iter() {
            let comp = &components[pid];
            let pool_toks: HashSet<String> = comp.tokens.iter().map(|t| t.address.to_string().to_lowercase()).collect();
            
            // 仅对包含目标对的池子进行模拟
            if pool_toks.contains(&t_in_addr) && pool_toks.contains(&t_out_addr) {
                if let Ok(res) = state.get_amount_out(chunk_size.clone(), t_in, t_out) {
                    if res.amount > best_chunk_out {
                        best_chunk_out = res.amount;
                        best_pool_id = Some(pid.clone());
                        best_new_state = Some(res.new_state);
                    }
                }
            }
        }

        if let (Some(pid), Some(new_st)) = (best_pool_id, best_new_state) {
            // 核心逻辑：叠加。下一次循环时，该池子的状态已更新（考虑了滑点）
            states.insert(pid, new_st);
            current_total_out += best_chunk_out;
        }
    }

    println!("\nFinal Greedy Result: Total USDC received: {}", current_total_out);
    Ok(())
}