# Task Dispatcher (Central Dispatcher) — FIFO vs Optimized Scheduling

This project implements a queue-based task scheduling system in Rust using a **central dispatcher** (manager queue) and a **bounded worker pool**.  
It compares two scheduling policies (FIFO vs Optimized) across two workloads (Balanced vs Stressed), with **total runtime** as the primary metric.

---
## One piece of advice accepted
Keep queue ownership inside a single dispatcher thread and use channels for communication.
This reduced shared mutable state and avoided needing locks around the queues.
## One piece of advice rejected
Using additional external concurrency frameworks or advanced scheduling structures early on.
## Tool Use Disclosure
Microsoft Copilot
## Design Summary
## Experment Summary
## Build & Run
```bash
cargo build --release
cargo run
```
### Requirements
- Rust toolchain installed (Cargo + rustc)
- Dependency: `rand = "0.8"` in `Cargo.toml`

### Build (debug)
```bash
cargo build
```
