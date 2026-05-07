# Final Project
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand::rngs::StdRng;

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::sync::{
    atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    mpsc, Arc,
};
use std::thread;
use std::time::{Duration, Instant};

const TASKS: usize = 1000;
const WORKERS: usize = 8;

const CPU_CAP: u32 = 100;              // GLOBAL CPU cap
const MONITOR_INTERVAL_MS: u64 = 10;   // monitor sampling interval

// Task CPU costs (resource model)
const IO_CPU_MIN: u32 = 5;             // IO uses ~10% CPU (variable)
const IO_CPU_MAX: u32 = 15;
const CPU_TASK_CPU: u32 = 35;          // CPU tasks use 35% CPU

// -------------------- types --------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Io,
    Cpu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Policy {
    /// Strict FIFO: only the head of the queue can be scheduled.
    /// If head doesn't fit CPU cap, dispatch blocks (head-of-line blocking).
    FifoStrict,
    /// Optimized: chooses tasks that *fit* the CPU cap to keep workers busy.
    OptimizedFit,
}

#[derive(Clone, Copy, Debug)]
struct SimConfig {
    // labels
    experiment_name: &'static str, // "Experiment A (Balanced)" etc.
    policy_name: &'static str,     // "FIFO simulation" etc.
    policy: Policy,

    // reproducibility
    seed: u64,

    // workload mix
    io_ratio: f64, // 0.70 means 70% IO, 30% CPU

    // arrivals
    base_interarrival_ms: u64, // typical spacing between tasks (e.g., 20ms)
    burst_prob: f64,           // probability of starting a burst at a given emission point
    burst_size: usize,         // number of tasks to emit in a burst
    burst_gap_ms: u64,         // delay between tasks within burst (0 or 1ms)
    post_burst_sleep_ms: u64,  // delay after a burst finishes (often base_interarrival_ms)

    // uneven durations
    io_dur_min_ms: u64,
    io_dur_max_ms: u64,
    cpu_dur_min_ms: u64,
    cpu_dur_max_ms: u64,

    // monitor output
    monitor_csv: &'static str,
}

#[derive(Clone, Debug)]
struct Task {
    id: u64,
    kind: Kind,
    cpu_cost: u32,
    duration: Duration,
    arrival: Instant,
}

#[derive(Clone, Debug)]
struct TaskResult {
    task_id: u64,
    kind: Kind,
    cpu_cost: u32,
    arrival: Instant,
    start: Instant,
    end: Instant,
}

#[derive(Debug)]
enum Event {
    NewTask(Task),
    GeneratorDone { total: usize },
    WorkerReady { worker_id: usize },
    WorkerCompleted { worker_id: usize, result: TaskResult },
}

#[derive(Debug)]
enum WorkerCmd {
    Run(Task),
    Shutdown,
}

#[derive(Debug)]
enum CollectorMsg {
    TaskCompleted(TaskResult),
    MonitorSample { cpu_in_use: u32, active_workers: usize },
    MonitorDone { samples: usize, csv_path: String },
    SimulationDone { sim_start: Instant, sim_end: Instant, total: usize },
}

// -------------------- generator --------------------
// Exact distribution (70/30 or 20/80) by building a list and shuffling.
// Emits in either normal mode (base_interarrival_ms) or burst mode (burst_size with burst_gap_ms).
// Durations are uneven per cfg (especially stressed experiment).
fn generator_loop(cfg: SimConfig, event_tx: mpsc::Sender<Event>) {
    let mut rng = StdRng::seed_from_u64(cfg.seed);

    // exact counts
    let io_count = ((TASKS as f64) * cfg.io_ratio).round() as usize;
    let io_count = io_count.min(TASKS);
    let cpu_count = TASKS - io_count;

    // build + shuffle (randomizer())
    let mut kinds: Vec<Kind> = Vec::with_capacity(TASKS);
    kinds.extend(std::iter::repeat(Kind::Io).take(io_count));
    kinds.extend(std::iter::repeat(Kind::Cpu).take(cpu_count));
    kinds.shuffle(&mut rng);

    let mut idx = 0usize;
    while idx < TASKS {
        let do_burst = cfg.burst_prob > 0.0 && rng.gen_bool(cfg.burst_prob);
        let batch = if do_burst { cfg.burst_size.max(1) } else { 1 };
        let batch = batch.min(TASKS - idx);

        // emit batch tasks
        for _ in 0..batch {
            let kind = kinds[idx];

            let cpu_cost = match kind {
                Kind::Io => rng.gen_range(IO_CPU_MIN..=IO_CPU_MAX), // variable ~10%
                Kind::Cpu => CPU_TASK_CPU,
            };

            let dur_ms = match kind {
                Kind::Io => rng.gen_range(cfg.io_dur_min_ms..=cfg.io_dur_max_ms),
                Kind::Cpu => rng.gen_range(cfg.cpu_dur_min_ms..=cfg.cpu_dur_max_ms),
            };

            let task = Task {
                id: idx as u64,
                kind,
                cpu_cost,
                duration: Duration::from_millis(dur_ms),
                arrival: Instant::now(),
            };

            event_tx.send(Event::NewTask(task)).expect("dispatcher hung up");
            idx += 1;

            // inter-arrival pacing
            if do_burst {
                if cfg.burst_gap_ms > 0 {
                    thread::sleep(Duration::from_millis(cfg.burst_gap_ms));
                }
            } else {
                thread::sleep(Duration::from_millis(cfg.base_interarrival_ms));
            }

            if idx >= TASKS {
                break;
            }
        }

        // after burst, sleep a bit to create bursty arrival pattern
        if do_burst && cfg.post_burst_sleep_ms > 0 && idx < TASKS {
            thread::sleep(Duration::from_millis(cfg.post_burst_sleep_ms));
        }
    }

    let _ = event_tx.send(Event::GeneratorDone { total: TASKS });
}

// -------------------- worker --------------------
// Executes tasks for their duration (sleep). CPU usage is modeled via dispatcher reservation.
fn worker_loop(
    worker_id: usize,
    cmd_rx: mpsc::Receiver<WorkerCmd>,
    event_tx: mpsc::Sender<Event>,
    active_workers: Arc<AtomicUsize>,
) {
    let _ = event_tx.send(Event::WorkerReady { worker_id });

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            WorkerCmd::Run(task) => {
                active_workers.fetch_add(1, Ordering::Relaxed);
                let start = Instant::now();

                thread::sleep(task.duration);

                let end = Instant::now();
                active_workers.fetch_sub(1, Ordering::Relaxed);

                let result = TaskResult {
                    task_id: task.id,
                    kind: task.kind,
                    cpu_cost: task.cpu_cost,
                    arrival: task.arrival,
                    start,
                    end,
                };

                let _ = event_tx.send(Event::WorkerCompleted { worker_id, result });
                let _ = event_tx.send(Event::WorkerReady { worker_id });
            }
            WorkerCmd::Shutdown => break,
        }
    }
}

// -------------------- dispatcher (central manager queue) --------------------
fn dispatcher_loop(
    cfg: SimConfig,
    event_rx: mpsc::Receiver<Event>,
    worker_cmds: Vec<mpsc::Sender<WorkerCmd>>,
    collector_tx: mpsc::Sender<CollectorMsg>,
    cpu_in_use_atomic: Arc<AtomicU32>,
    stop_flag: Arc<AtomicBool>, 
    sim_start: Instant,
) {
    // For FIFO strict, use one queue.
    let mut fifo_q: VecDeque<Task> = VecDeque::new();
    // For optimized, use split queues.
    let mut io_q: VecDeque<Task> = VecDeque::new();
    let mut cpu_q: VecDeque<Task> = VecDeque::new();

    let mut available_workers: VecDeque<usize> = VecDeque::new();

    let mut generator_done = false;
    let mut total_tasks = 0usize;
    let mut completed = 0usize;

    let mut cpu_in_use: u32 = 0;

    loop {
        // receive events with small timeout so we can keep dispatching
        match event_rx.recv_timeout(Duration::from_millis(2)) {
            Ok(ev) => match ev {
                Event::NewTask(task) => {
                    match cfg.policy {
                        Policy::FifoStrict => fifo_q.push_back(task),
                        Policy::OptimizedFit => match task.kind {
                            Kind::Io => io_q.push_back(task),
                            Kind::Cpu => cpu_q.push_back(task),
                        },
                    }
                }
                Event::GeneratorDone { total } => {
                    generator_done = true;
                    total_tasks = total;
                }
                Event::WorkerReady { worker_id } => {
                    available_workers.push_back(worker_id);
                }
                Event::WorkerCompleted { worker_id: _, result } => {
                    completed += 1;

                    // free reserved CPU
                    cpu_in_use = cpu_in_use.saturating_sub(result.cpu_cost);
                    cpu_in_use_atomic.store(cpu_in_use, Ordering::Relaxed);

                    let _ = collector_tx.send(CollectorMsg::TaskCompleted(result));
                }
            },
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        // dispatch as much as possible
        'dispatch: while let Some(wid) = available_workers.pop_front() {
            let chosen: Option<Task> = match cfg.policy {
                Policy::FifoStrict => {
                    // strict FIFO: only head can be scheduled; if it doesn't fit, block
                    if let Some(front) = fifo_q.front() {
                        if cpu_in_use + front.cpu_cost <= CPU_CAP {
                            fifo_q.pop_front()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Policy::OptimizedFit => {
                    // optimized: choose tasks that fit CPU cap to maximize worker activity
                    // heuristic:
                    // when CPU is tight, prefer IO (lower cpu_cost)
                    // otherwise prefer CPU first (keeps CPU lanes utilized), fallback to IO
                    let cpu_tight = cpu_in_use >= 80;

                    let pick_from = |q: &mut VecDeque<Task>, cpu_now: u32| -> Option<Task> {
                        if let Some(front) = q.front() {
                            if cpu_now + front.cpu_cost <= CPU_CAP {
                                return q.pop_front();
                            }
                        }
                        None
                    };

                    if cpu_tight {
                        pick_from(&mut io_q, cpu_in_use).or_else(|| pick_from(&mut cpu_q, cpu_in_use))
                    } else {
                        pick_from(&mut cpu_q, cpu_in_use).or_else(|| pick_from(&mut io_q, cpu_in_use))
                    }
                }
            };

            let task = match chosen {
                Some(t) => t,
                None => {
                    // can't dispatch now due to empty queues or CPU cap blocking
                    available_workers.push_front(wid);
                    break 'dispatch;
                }
            };

            // reserve CPU
            cpu_in_use += task.cpu_cost;
            cpu_in_use_atomic.store(cpu_in_use, Ordering::Relaxed);

            worker_cmds[wid].send(WorkerCmd::Run(task)).expect("worker hung up");
        }

        let queues_empty = match cfg.policy {
            Policy::FifoStrict => fifo_q.is_empty(),
            Policy::OptimizedFit => io_q.is_empty() && cpu_q.is_empty(),
        };

        if generator_done && queues_empty && completed == total_tasks {
            let sim_end = Instant::now();

            // stop monitor first
            stop_flag.store(true, Ordering::Relaxed);

            // shutdown workers
            for tx in &worker_cmds {
                let _ = tx.send(WorkerCmd::Shutdown);
            }

            let _ = collector_tx.send(CollectorMsg::SimulationDone {
                sim_start,
                sim_end,
                total: total_tasks,
            });

            break;
        }
    }
}

// -------------------- monitor --------------------
// Samples CPU-in-use and active workers every 10ms and writes a CSV.
fn monitor_loop(
    stop_flag: Arc<AtomicBool>,
    cpu_in_use: Arc<AtomicU32>,
    active_workers: Arc<AtomicUsize>,
    collector_tx: mpsc::Sender<CollectorMsg>,
    csv_path: String,
) {
    let mut file = File::create(&csv_path).expect("could not create monitor csv");
    let _ = writeln!(file, "sample,cpu_in_use,active_workers");

    let mut samples: usize = 0;

    while !stop_flag.load(Ordering::Relaxed) {
        let cpu = cpu_in_use.load(Ordering::Relaxed);
        let active = active_workers.load(Ordering::Relaxed);

        let _ = writeln!(file, "{},{},{}", samples, cpu, active);

        let _ = collector_tx.send(CollectorMsg::MonitorSample {
            cpu_in_use: cpu,
            active_workers: active,
        });

        samples += 1;
        thread::sleep(Duration::from_millis(MONITOR_INTERVAL_MS));
    }

    let _ = collector_tx.send(CollectorMsg::MonitorDone {
        samples,
        csv_path,
    });
}

// -------------------- collector --------------------
fn collector_loop(rx: mpsc::Receiver<CollectorMsg>, cfg: SimConfig) {
    let mut results: Vec<TaskResult> = Vec::new();

    // monitor aggregation
    let mut monitor_samples = 0usize;
    let mut cpu_sum: u64 = 0;
    let mut active_sum: u64 = 0;
    let mut monitor_csv = cfg.monitor_csv.to_string();

    // per-class waits (for optimized output)
    let mut io_wait_sum = Duration::ZERO;
    let mut cpu_wait_sum = Duration::ZERO;
    let mut io_n = 0usize;
    let mut cpu_n = 0usize;

    // max wait
    let mut max_wait = Duration::ZERO;
    let mut max_wait_task: u64 = 0;

    // done flags
    let mut sim_done: Option<(Instant, Instant, usize)> = None;
    let mut monitor_done = false;

    while let Ok(msg) = rx.recv() {
        match msg {
            CollectorMsg::TaskCompleted(r) => {
                let wait = r.start.duration_since(r.arrival);
                if wait > max_wait {
                    max_wait = wait;
                    max_wait_task = r.task_id;
                }

                match r.kind {
                    Kind::Io => { io_wait_sum += wait; io_n += 1; }
                    Kind::Cpu => { cpu_wait_sum += wait; cpu_n += 1; }
                }

                results.push(r);
            }
            CollectorMsg::MonitorSample { cpu_in_use, active_workers } => {
                monitor_samples += 1;
                cpu_sum += cpu_in_use as u64;
                active_sum += active_workers as u64;
            }
            CollectorMsg::MonitorDone { samples: _, csv_path } => {
                monitor_csv = csv_path;
                monitor_done = true;
            }
            CollectorMsg::SimulationDone { sim_start, sim_end, total } => {
                sim_done = Some((sim_start, sim_end, total));
            }
        }

        // Only exit once received BOTH end markers
        if sim_done.is_some() && monitor_done {
            break;
        }
    }

    let (sim_start, sim_end, total_expected) = sim_done.expect("missing SimulationDone");

    let total_runtime = sim_end.duration_since(sim_start);

    // makespan: from first task arrival to last completion
    let earliest_arrival = results.iter().map(|r| r.arrival).min().unwrap_or(sim_start);
    let latest_end = results.iter().map(|r| r.end).max().unwrap_or(sim_end);
    let makespan = latest_end.duration_since(earliest_arrival);

    let completed = results.len();
    let io_done = results.iter().filter(|r| r.kind == Kind::Io).count();
    let cpu_done = results.iter().filter(|r| r.kind == Kind::Cpu).count();

    // avg wait + turnaround
    let mut wait_sum = Duration::ZERO;
    let mut turn_sum = Duration::ZERO;
    for r in &results {
        wait_sum += r.start.duration_since(r.arrival);
        turn_sum += r.end.duration_since(r.arrival);
    }
    let avg_wait = wait_sum / (completed as u32).max(1);
    let avg_turn = turn_sum / (completed as u32).max(1);

    // monitor averages (simple average of samples, which approximates time-weighted average at fixed 10ms interval)
    let avg_cpu_usage = if monitor_samples > 0 {
        cpu_sum as f64 / monitor_samples as f64
    } else { 0.0 };

    let avg_workers_active = if monitor_samples > 0 {
        active_sum as f64 / monitor_samples as f64
    } else { 0.0 };

    // helpful formatting
    let io_pct = (cfg.io_ratio * 100.0).round() as u32;
    let cpu_pct = 100u32.saturating_sub(io_pct);

    let ms_u128 = |d: Duration| d.as_millis();
    let ms_f64 = |d: Duration| d.as_secs_f64() * 1000.0;

    // header
    println!("\n== {} — {} ==", cfg.experiment_name, cfg.policy_name);
    println!(
        "{} tasks, {}% IO / {}% CPU, {} workers, cap {}%",
        total_expected, io_pct, cpu_pct, WORKERS, CPU_CAP
    );

    // show workload knobs (helps write-up and proves stressors)
    println!(
        "arrivals: base {}ms, burst_prob {:.2}, burst_size {}, burst_gap {}ms, post_burst {}ms",
        cfg.base_interarrival_ms, cfg.burst_prob, cfg.burst_size, cfg.burst_gap_ms, cfg.post_burst_sleep_ms
    );
    println!(
        "durations(ms): IO [{}..{}], CPU [{}..{}]",
        cfg.io_dur_min_ms, cfg.io_dur_max_ms, cfg.cpu_dur_min_ms, cfg.cpu_dur_max_ms
    );

    println!("\n— results —");
    println!("total runtime           : {} ms", ms_u128(total_runtime));
    println!("makespan                : {} ms", ms_u128(makespan));
    println!(
        "tasks completed         : {} (IO={}, CPU={})",
        completed, io_done, cpu_done
    );

    match cfg.policy {
        Policy::FifoStrict => {
            println!("avg wait time           : {:.2} ms", ms_f64(avg_wait));
            println!("avg turnaround time     : {:.2} ms", ms_f64(avg_turn));
            println!("max wait time           : {} ms", ms_u128(max_wait));
        }
        Policy::OptimizedFit => {
            let io_avg_wait = if io_n > 0 { io_wait_sum / io_n as u32 } else { Duration::ZERO };
            let cpu_avg_wait = if cpu_n > 0 { cpu_wait_sum / cpu_n as u32 } else { Duration::ZERO };

            println!("avg wait time           : {:.2} ms", ms_f64(avg_wait));
            println!("avg wait (IO only)      : {:.2} ms", ms_f64(io_avg_wait));
            println!("avg wait (CPU only)     : {:.2} ms", ms_f64(cpu_avg_wait));
            println!("avg turnaround time     : {:.2} ms", ms_f64(avg_turn));
            println!(
                "max wait time           : {} ms (task #{})",
                ms_u128(max_wait),
                max_wait_task
            );
        }
    }

    println!("avg CPU usage           : {:.2} %", avg_cpu_usage);
    println!("avg workers active      : {:.2} / {}", avg_workers_active, WORKERS);
    println!("monitor samples         : {}", monitor_samples);
    println!("monitor csv             : {}", monitor_csv);
}

// -------------------- run one configured simulation --------------------
fn run_sim(cfg: SimConfig) {
    let sim_start = Instant::now();

    // Generator/Workers -> Dispatcher
    let (event_tx, event_rx) = mpsc::channel::<Event>();
    // Dispatcher/Monitor -> Collector
    let (collector_tx, collector_rx) = mpsc::channel::<CollectorMsg>();

    // shared state for monitor
    let cpu_in_use = Arc::new(AtomicU32::new(0));
    let active_workers = Arc::new(AtomicUsize::new(0));
    let stop_flag = Arc::new(AtomicBool::new(false));

    // collector
    let collector_cfg = cfg;
    let collector_handle = thread::spawn(move || collector_loop(collector_rx, collector_cfg));

    // monitor
    let mon_stop = stop_flag.clone();
    let mon_cpu = cpu_in_use.clone();
    let mon_active = active_workers.clone();
    let mon_tx = collector_tx.clone();
    let mon_csv = cfg.monitor_csv.to_string();
    let monitor_handle = thread::spawn(move || monitor_loop(mon_stop, mon_cpu, mon_active, mon_tx, mon_csv));

    // workers
    let mut worker_cmds = Vec::with_capacity(WORKERS);
    let mut worker_handles = Vec::with_capacity(WORKERS);

    for wid in 0..WORKERS {
        let (cmd_tx, cmd_rx) = mpsc::channel::<WorkerCmd>();
        worker_cmds.push(cmd_tx);

        let ev_tx = event_tx.clone();
        let active = active_workers.clone();
        worker_handles.push(thread::spawn(move || worker_loop(wid, cmd_rx, ev_tx, active)));
    }

    // dispatcher
    let disp_cfg = cfg;
    let disp_cpu = cpu_in_use.clone();
    let disp_stop = stop_flag.clone();
    let disp_col_tx = collector_tx.clone();
    let dispatcher_handle = thread::spawn(move || {
        dispatcher_loop(disp_cfg, event_rx, worker_cmds, disp_col_tx, disp_cpu, disp_stop, sim_start)
    });

    // generator
    let gen_cfg = cfg; // SimConfig is Copy
    let gen_tx = event_tx.clone();
    let generator_handle = thread::spawn(move || generator_loop(gen_cfg, gen_tx));

    // join all
    generator_handle.join().expect("generator panicked");
    dispatcher_handle.join().expect("dispatcher panicked");
    for h in worker_handles {
        h.join().expect("worker panicked");
    }
    monitor_handle.join().expect("monitor panicked");
    collector_handle.join().expect("collector panicked");
}

// -------------------- main: 4 runs (A FIFO, A OPT, B FIFO, B OPT) --------------------
fn main() {
    // Experiment A: Balanced 70/30, regular arrivals, mild/near-uniform durations
    let exp_a_fifo = SimConfig {
        experiment_name: "Experiment A (Balanced)",
        policy_name: "FIFO simulation",
        policy: Policy::FifoStrict,
        seed: 42,
        io_ratio: 0.70,

        base_interarrival_ms: 20,
        burst_prob: 0.00,        // basically no bursts in balanced
        burst_size: 1,
        burst_gap_ms: 0,
        post_burst_sleep_ms: 0,

        io_dur_min_ms: 180,
        io_dur_max_ms: 220,
        cpu_dur_min_ms: 180,
        cpu_dur_max_ms: 260,

        monitor_csv: "monitor_expA_fifo.csv",
    };

    let exp_a_opt = SimConfig {
        policy_name: "Optimized simulation",
        policy: Policy::OptimizedFit,
        monitor_csv: "monitor_expA_opt.csv",
        ..exp_a_fifo
    };

    // Experiment B: Stressed 20/80 (CPU-heavy) + bursts + uneven durations (long CPU tail)
    let exp_b_fifo = SimConfig {
        experiment_name: "Experiment B (Stressed)",
        policy_name: "FIFO simulation",
        policy: Policy::FifoStrict,
        seed: 99,
        io_ratio: 0.20, // 20% IO / 80% CPU (CPU-heavy)

        base_interarrival_ms: 20,
        burst_prob: 0.15,       // bursts happen fairly often
        burst_size: 35,         // big bursts
        burst_gap_ms: 0,        // burst tasks arrive almost instantly
        post_burst_sleep_ms: 20,// return to base spacing after a burst

        io_dur_min_ms: 100,
        io_dur_max_ms: 220,
        cpu_dur_min_ms: 250,
        cpu_dur_max_ms: 900,    // uneven and long tail => stress

        monitor_csv: "monitor_expB_fifo.csv",
    };

    let exp_b_opt = SimConfig {
        policy_name: "Optimized simulation",
        policy: Policy::OptimizedFit,
        monitor_csv: "monitor_expB_opt.csv",
        ..exp_b_fifo
    };

    // ---- 4 runs ----
    run_sim(exp_a_fifo);
    run_sim(exp_a_opt);
    run_sim(exp_b_fifo);
    run_sim(exp_b_opt);
}