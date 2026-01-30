# Tycho-Test-Single

Summary:
1. Current Status
Core Logic: Successfully implemented an iterative greedy solver for 100-chunk price impact simulation.

Protocol Support: Fully compatible with Uniswap V2 and Uniswap V3 (handled the tick_liquidities data dependency).

Update Frequency: Using a stable commit (rev = "d1b732c") to bypass recent upstream compilation errors.

2. Simulation Results (WETH/USDC)
Input: 100 WETH (Split into 100 chunks of 1 WETH each).

Output: ~275,315.98 USDC.

Matched Chunks: 100/100 (Verified in-place state updates working correctly).

Observation: The total output reflects a cumulative price impact in a static environment without external arbitrage rebalancing.

3. Important Notes & "Caveats" 
Upstream Issue: As of Jan 30(about 6pm), 2026, the main branch of tycho-simulation contains experimental code (Ekubo V3) that fails to compile. My project is locked to a stable revision to ensure reliability.

Pool Discovery:

WETH/USDC: Successful execution (Direct pair exists).

osETH/USDC: Currently returning "No Pools" maybe because it is a low-liquidity pair that requires Multi-hop (e.g., osETH -> WETH -> USDC), which is the next planned feature.

# Some failed Test
@ShengShuYan ➜ /workspaces/Tycho-Test-Single/solver (main) $ cargo run
   Compiling solver v0.1.0 (/workspaces/Tycho-Test-Single/solver)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.77s
     Running `target/debug/solver`
Step 1: Loading tokens...
Step 2: Syncing pool snapshot...
Step 2: Synced 586 total pools.
Step 3: Simulating 100 chunks for 100 wstETH...
Progress: 10/100 processed
Progress: 20/100 processed
Progress: 30/100 processed
Progress: 40/100 processed
Progress: 50/100 processed
Progress: 60/100 processed
Progress: 70/100 processed
Progress: 80/100 processed
Progress: 90/100 processed
Progress: 100/100 processed

================ SIMULATION REPORT ================
Status: FAILED
Reason: No direct pools found for this pair in the synced snapshot.

# Successful Test
@ShengShuYan ➜ /workspaces/Tycho-Test-Single/solver (main) $ cargo run
   Compiling solver v0.1.0 (/workspaces/Tycho-Test-Single/solver)
warning: unused variable: `pid`
  --> src/main.rs:71:10
   |
71 |     for (pid, meta) in components.iter() {
   |          ^^^ help: if this is intentional, prefix it with an underscore: `_pid`
   |
   = note: `#[warn(unused_variables)]` (part of `#[warn(unused)]`) on by default

warning: `solver` (bin "solver") generated 1 warning (run `cargo fix --bin "solver" -p solver` to apply 1 suggestion)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.37s
     Running `target/debug/solver`
Step 1: Loading tokens...
Step 2: Syncing pools...
Step 2: Synced 586 total pools.
   MATCH FOUND: 5 pools contain WETH/USDC.
Step 3: Simulating 100 chunks for 100 WETH...

================ FINAL REPORT ================
Status: SUCCESS
Pair: WETH / USDC
Chunks Matched: 100/100
Total USDC Received: 275315985821 （USDC has 6 decimal， here is 275,316 USDC）
==============================================
