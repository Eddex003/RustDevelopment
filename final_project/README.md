# Task Dispatcher (Central Dispatcher) — FIFO vs Optimized Scheduling

This project implements a queue-based task scheduling system in Rust using a **central dispatcher** (manager queue) and a **bounded worker pool**.  
It compares two scheduling policies (FIFO vs Optimized) across two workloads (Balanced vs Stressed), with **total runtime** as the primary metric.

---

## Build & Run

### Requirements
- Rust toolchain installed (Cargo + rustc)
- Dependency: `rand = "0.8"` in `Cargo.toml`

### Build (debug)
```bash
cargo build
