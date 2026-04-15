use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

const TERMINATION_SIGNAL: i32 = -1;

fn main() {
    const ITEM_COUNT: usize = 20;

    let (tx, rx) = mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));

    let mut producer_handles = Vec::new();
    for id in 0..2 {
        let tx_clone = tx.clone();
        producer_handles.push(thread::spawn(move || {
            producer(id, tx_clone, ITEM_COUNT);
        }));
    }

    let mut consumer_handles = Vec::new();
    for id in 0..3 {
        let rx_clone = Arc::clone(&rx);
        consumer_handles.push(thread::spawn(move || {
            consumer(id, rx_clone);
        }));
    }

    for handle in producer_handles {
        handle.join().unwrap();
    }

    for _ in 0..3 {
        tx.send(TERMINATION_SIGNAL).unwrap();
    }

    for handle in consumer_handles {
        handle.join().unwrap();
    }

    println!("All items have been produced and consumed!");
}

fn producer(id: usize, tx: mpsc::Sender<i32>, item_count: usize) {
    for i in 0..item_count {
        let value = (id as i32 * 100) + i as i32;
        println!("Producer {} produced {}", id, value);
        tx.send(value).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}

fn consumer(id: usize, rx: Arc<Mutex<mpsc::Receiver<i32>>>) {
    loop {
        let value = rx.lock().unwrap().recv().unwrap();

        if value == TERMINATION_SIGNAL {
            println!("Consumer {} received termination signal", id);
            break;
        }

        println!("Consumer {} consumed {}", id, value);
        thread::sleep(Duration::from_millis(150));
    }
}