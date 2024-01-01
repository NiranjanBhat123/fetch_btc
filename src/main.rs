use std::fs::File;
use tokio::task::spawn_blocking;
use std::io::{self, Write};
use reqwest::blocking::Client;
use serde_json::Value;
use tokio::task;
use tokio::spawn;
use futures_util::stream::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;



#[tokio::main]
async fn main() {
    println!("select any command [- cargo run -- --mode=read ] or [cargo run -- --mode = cache --times=10] ");
    let args: Vec<String> = std::env::args().collect();
    println!("Command-line arguments: {:?}", args);
    let mode = &args[1];
    match mode.as_str() {
        "--mode=cache" => {
            println!("selected cache mode !");
            println!("args is :{:?}",args);
            cache_mode(10).await;
        }
        "--mode=read" => {
            println!("selected read mode !");
            read_mode().await;
        }
        _ => {
            eprintln!("Invalid mode");
            std::process::exit(1);
        }
    }
}

async fn cache_mode(times: usize) {
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
    let mut prices = Vec::new();

    for _ in 0..times {
        let response = reqwest::get(url).await.expect("Failed to send HTTP request");
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

    println!("Cache complete. The average USD price of BTC is: {}", average_price);

    // Save result and data points to a file
    save_to_file(
        "cache_results.txt",
        &format!("Average Price: {}\nData Points: {:?}", average_price, prices),
    )
    .await
    .expect("Failed to save to file");
}



async fn read_mode() {
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
    let mut prices = Vec::new();
    for _ in 0..30 {
        let response = reqwest::get(url).await.expect("Failed to make HTTP request");
        let body = response.text().await.expect("Failed to read response body");
        
        let price: f64 = serde_json::from_str::<Value>(&body)
            .unwrap()
            .get("data")
            .and_then(|data| data.get("amount"))
            .and_then(|amount| amount.as_str())
            .and_then(|amount_str| amount_str.parse::<f64>().ok())
            .unwrap_or_default();
        
        prices.push(price);
    }

    let average_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

    println!("Read complete. The average USD price of BTC is: {}", average_price);

   
    save_this_to_file(
        "read_results.txt",
        &format!("Average Price: {}\nData Points: {:?}", average_price, prices),
    )
    .unwrap();
}


async fn save_to_file(filename: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn save_this_to_file(filename: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}