# Tycho-Test-Single: Iterative Greedy Solver Simulation

## 1. Project Overview
* **Core Logic**: Successfully implemented an **Iterative Greedy Solver** with a 100-chunk price impact simulation. 
* **In-place State Update**: The solver splits the total volume into chunks, identifies the optimal pool for each chunk, and updates the pool state (`new_state`) locally to simulate real-time price impact.
* **Protocol Compatibility**: Fully supports **Uniswap V2** and **Uniswap V3**. 
    * *Note*: Resolved the "Missing attributes `tick_liquidities`" runtime error for V3 by explicitly requesting tick data during subscription.

## 2. Stability & Environment
* **Dependency Locking**: Locked `tycho-simulation` to a stable revision (`rev = "d1b732c"`).
    * *Context*: As of Jan 30, 2026 (approx. 6:00 PM), the upstream `main` branch contains experimental Ekubo V3 code that causes compilation failures. This lock ensures a stable and reproducible build.

---

## 3. Simulation Results (WETH/USDC)
* **Input**: 100 WETH (Split into 100 chunks of 1 WETH each).
* **Output**: **~275,316.00 USDC** * *Raw Data*: `275315985821` (USDC has 6 decimals).
* **Execution**: 100/100 Chunks matched successfully.
* **Analysis**: The result accurately reflects cumulative price impact in a static snapshot environment (without external arbitrage/backrun rebalancing).

---

## 4. Discovery & Limitations (Caveats)
* **Pool Discovery**: 
    * **WETH/USDC**: **SUCCESS**. Direct liquidity pools were correctly identified and utilized.
    * **osETH/USDC**: **FAILED**. No direct pools found in the current sync snapshot. 
* **Next Steps**: Development of **Multi-hop Routing** (e.g., `osETH -> WETH -> USDC`) is required to support low-liquidity pairs that lack direct trading pools.

---

## 5. Execution Logs

### ✅ Successful Test (WETH/USDC)
```text
Step 1: Loading tokens...
Step 2: Syncing pools...
Step 2: Synced 586 total pools.
    MATCH FOUND: 5 pools contain WETH/USDC.
Step 3: Simulating 100 chunks for 100 WETH...

================ FINAL REPORT ================
Status: SUCCESS
Pair: WETH / USDC
Chunks Matched: 100/100
Total USDC Received: 275315985821
==============================================

### ❌ Failed Test (wstETH/USDC - Missing Direct Pool)
Step 1: Loading tokens...
Step 2: Syncing pool snapshot...
Step 2: Synced 586 total pools.
Step 3: Simulating 100 chunks for 100 wstETH...

================ SIMULATION REPORT ================
Status: FAILED
Reason: No direct pools found for this pair in the synced snapshot.
