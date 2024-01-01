use serde_json::Value;
use std::fs;
use futures_util::future::try_join_all;
use std::fs::File;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};
use tokio::task;
use std::time::{Duration, Instant};





#[tokio::main]
async fn main() {
    println!("select any command [- cargo run -- --mode=read ] or [cargo run -- --mode = cache --times= any_number] or [cargo run -- --mode=distributed]");
    let args: Vec<String> = std::env::args().collect();
    let mode = &args[1];
    match mode.as_str() {
        "--mode=cache" => {
            println!("selected cache mode !");

            let times = args[3].parse::<usize>().expect("Invalid times argument");
            println!("Cache mode will run {} times", times);
            cache_mode(times).await;
        }
        "--mode=read" => {
            println!("selected read mode !");
            read_mode().await;
        }
        "--mode=distributed" => {
            println!("Selected distributed mode!");

            // Shared state for averaging
            let shared_state = Arc::new(Mutex::new(AggregatorState::new()));

            // Spawn 5 client tasks
            let client_tasks = (0..5)
                .map(|_| {
                    let state = Arc::clone(&shared_state);
                    task::spawn(client_task(state))
                })
                .collect::<Vec<_>>();

            // Spawn the aggregator task
            let aggregator_task = task::spawn(aggregator_task(shared_state));

            // Wait for all tasks to complete
            try_join_all(client_tasks)
                .await
                .expect("Error in client tasks");

            // Wait for the aggregator to complete
            aggregator_task.await.expect("Error in aggregator task");
        }
        _ => {
            eprintln!("Invalid mode");
            std::process::exit(1);
        }
    }
}


struct AggregatorState {
    averages: Vec<f64>,
    completed_clients: usize,
}

impl AggregatorState {
    fn new() -> Self {
        Self {
            averages: Vec::new(),
            completed_clients: 0,
        }
    }
}




async fn cache_mode(times: usize) {
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
    let mut prices = Vec::new();

    for _ in 0..times {
        let response = reqwest::get(url)
            .await
            .expect("Failed to send HTTP request");
        let body = response.text().await.expect("Failed to read response body");
        let json: Value = serde_json::from_str(&body).expect("Error parsing JSON");

        if let Some(data) = json.get("data") {
            if let (Some(amount), Some(base), Some(currency)) =
                (data.get("amount"), data.get("base"), data.get("currency"))
            {
                let price: f64 = amount.as_str().unwrap().parse().unwrap();
                println!(
                    "Received data: Amount={}, Base={}, Currency={}",
                    amount, base, currency
                );
                prices.push(price);
            } else {
                eprintln!("Incomplete data in JSON response");
            }
        } else {
            eprintln!("No 'data' field in JSON response");
        }
    }

    let average_price: f64 = prices.iter().sum::<f64>() / times as f64;

    println!(
        "Cache complete. The average USD price of BTC is: {}",
        average_price
    );

    
    save_to_file(
        "cache_results.txt",
        &format!(
            "Average Price: {}\nData Points: {:?}",
            average_price, prices
        ),
    )
    .await
    .expect("Failed to save to file");
}

async fn read_mode() {
    if let Ok(contents) = fs::read_to_string("cache_results.txt") {
        println!(
            "Read complete. Contents of cache_results.txt:\n{}",
            contents
        );
    } else {
        eprintln!("Failed to read from file or file not found.");
    }
}

async fn save_to_file(filename: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}



async fn client_task(state: Arc<Mutex<AggregatorState>>) {
    let start_time = Instant::now();
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";

    // Simulate reading values for 10 seconds
    while start_time.elapsed() < Duration::from_secs(10) {
        // Perform HTTP request and computation of average
        let response = reqwest::get(url)
            .await
            .expect("Failed to send HTTP request");
        let body = response.text().await.expect("Failed to read response body");
        let json: Value = serde_json::from_str(&body).expect("Error parsing JSON");

        if let Some(data) = json.get("data") {
            if let Some(amount) = data.get("amount") {
                let price: f64 = amount.as_str().unwrap().parse().unwrap();
                println!("Received data: Amount={}", amount);
                
                
                let average = price; 
                let mut state = state.lock().unwrap();
                state.averages.push(average);
                state.completed_clients += 1;

                // Break if received averages from all client tasks
                if state.completed_clients == 5 {
                    break;
                }
            } else {
                eprintln!("Incomplete data in JSON response");
            }
        } else {
            eprintln!("No 'data' field in JSON response");
        }

        // Simulate waiting between fetching data points
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}


async fn aggregator_task(state: Arc<Mutex<AggregatorState>>) {
    // Wait for all client tasks to complete
    while state.lock().unwrap().completed_clients < 5 {
        task::yield_now().await;
    }

    let state = state.lock().unwrap();

    // Compute the final average
    let final_average: f64 = state.averages.iter().sum::<f64>() / state.averages.len() as f64;

    // Display the final average
    println!(
        "Distributed mode complete. The final average USD price of BTC is: {}",
        final_average
    );
}
