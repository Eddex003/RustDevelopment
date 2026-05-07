# Task Dispatcher (Central Dispatcher) — FIFO vs Optimized

This project implements a queue-based task scheduling system in Rust using a **central dispatcher (manager queue)** and a **bounded worker pool (8 workers)**.  
A **global CPU cap of 100%** is enforced via a CPU “reservation” model. The primary optimization goal is **total runtime**. All other metrics are supplementary and included to help explain behavior.

---

## Build and Run
### Build (debug)
```bash
cargo build --release
cargo run --release
