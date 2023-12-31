use std::fs::File;
// use futures_util::stream::StreamExt;
use std::io::{self, Write};
// use tokio::main;
// use core::future;
use reqwest::blocking::Client;
// use tokio::runtime::Runtime;
// use tokio_tungstenite::tungstenite::Message;
// use tokio_tungstenite::connect_async;
// use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use serde_json::Value;



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

// async fn cache_mode(times: usize) {
//     let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
//     let mut prices = Vec::new();

//     for _ in 0..times {
//         let response = Client::new().get(url).send().unwrap();
//         let body = response.text().unwrap();
//         let json: Value = serde_json::from_str(&body).unwrap();
        
//         if let Some(data) = json.get("data") {
//             if let (Some(amount), Some(base), Some(currency)) = (data.get("amount"), data.get("base"), data.get("currency")) {
//                 let price: f64 = amount.as_str().unwrap().parse().unwrap();
//                 println!("price is :price ={}",price);
//                 prices.push(price);
//                 println!("Received data: Amount={}, Base={}, Currency={}", amount, base, currency);
//             } else {
//                 eprintln!("Incomplete data in JSON response");
//             }
//         } else {
//             eprintln!("No 'data' field in JSON response");
//         }
//     }

//     let average_price: f64 = prices.iter().sum::<f64>() / times as f64;

//     println!("Cache complete. The average USD price of BTC is: {}", average_price);

   
//     let _ =tokio::task::spawn_blocking(move || {
//         save_to_file("cache_results.txt", &format!("Average Price: {}\nData Points: {:?}", average_price, prices))
//     }).await.unwrap();
// }


async fn cache_mode(times: usize) {
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
    let mut prices = Vec::new();

    for _ in 0..times {
        let response = reqwest::blocking::get(url).expect("Failed to make HTTP request");
        let body = response.text().expect("Failed to read response body");
        let json: Value = serde_json::from_str(&body).expect("Failed to parse JSON");

        if let Some(data) = json.get("data") {
            if let (Some(amount), Some(base), Some(currency)) = (
                data.get("amount"),
                data.get("base"),
                data.get("currency"),
            ) {
                let price: f64 = amount.as_str().unwrap().parse().unwrap();
                println!("Received data: Amount={}, Base={}, Currency={}", amount, base, currency);
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

    // Use spawn_blocking to offload blocking operation to a separate thread
    let _ = tokio::task::spawn_blocking(move || {
        save_to_file(
            "cache_results.txt",
            &format!("Average Price: {}\nData Points: {:?}", average_price, prices),
        )
    })
    .await
    .unwrap();
}





// async fn read_mode() {
//     let url = "wss://api.coinbase.com/v2/prices/spot?currency=USD";
//     let mut prices = Vec::new();

//     let start_time = Instant::now();

//     let (ws_stream, _) = connect_async(url)
//         .await
//         .expect("Failed to connect to WebSocket");
    
//     let ws_stream = ws_stream
//         .take_while(|_msg| future::ready(start_time.elapsed() < Duration::from_secs(5)))
//         .for_each(|msg| {
//             if let Ok(Message::Text(text)) = msg {
//                 let price: f64 = serde_json::from_str::<Value>(&text)
//                     .unwrap()
//                     .get("data")
//                     .and_then(|data| data.get("amount"))
//                     .and_then(|amount| amount.as_str())
//                     .and_then(|amount_str| amount_str.parse::<f64>().ok())
//                     .unwrap_or_default();
//                 prices.push(price);
//             }
//             future::ready(())
//         });

//     ws_stream.await;

//     let average_price: f64 = prices.iter().sum::<f64>() / prices.len() as f64;

//     println!("Read complete. The average USD price of BTC is: {}", average_price);

//     // Save result and data points to a file
//     save_to_file(
//         "read_results.txt",
//         &format!("Average Price: {}\nData Points: {:?}", average_price, prices),
//     )
//     .unwrap();
// }


async fn read_mode() {
    let url = "https://api.coinbase.com/v2/prices/spot?currency=USD";
    let mut prices = Vec::new();

    // let start_time = Instant::now();

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

    // Save result and data points to a file
    save_to_file(
        "read_results.txt",
        &format!("Average Price: {}\nData Points: {:?}", average_price, prices),
    )
    .unwrap();
}






fn save_to_file(filename: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(filename)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}