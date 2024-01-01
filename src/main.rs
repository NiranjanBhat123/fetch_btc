use serde_json::Value;
use std::fs;
use std::fs::File;
use std::io::{self, Write};




#[tokio::main]
async fn main() {
    println!("select any command [- cargo run -- --mode=read ] or [cargo run -- --mode = cache --times= any_number] ");
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
