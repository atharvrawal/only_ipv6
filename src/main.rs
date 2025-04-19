use receiver::receiver_main;
use tokio::net::TcpStream;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};
use bincode;
use rfd::FileDialog;
use crc16::*;
use tokio::io::AsyncWriteExt;
mod receiver;
use serde_json::Value;
mod sender;
mod get_ipv6;
mod encryption_module;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::io::{self, Write};
use get_ipv6::get_pip_port_json;
use sender::sender_main;

pub fn print_keys_from_json_string(json_string: String){
    match serde_json::from_str::<Value>(&json_string) {
        Ok(Value::Object(map)) => {
            for key in map.keys() {
                println!("- {}", key);
            }
        }
        Ok(_) => {
            println!("The JSON is valid but not a JSON object.");
        }
        Err(e) => {
            println!("Failed to parse JSON: {}", e);
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub header: u64,
    pub sno: u32,
    pub payload_length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let url = Url::parse("ws://54.66.23.75:8765").unwrap();
    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    println!("Connected to server");
    print!("Enter your username : ");
    io::stdout().flush().unwrap();
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read line");
    let username = username.trim();
    let user_info = get_pip_port_json(&username).to_string();
    
    ws_stream.send(user_info.into()).await.expect("Failed to send");

    if let Some(Ok(msg)) = ws_stream.next().await {
        println!("Received: {}", msg);
    }

    print!("Do u want to send[0] or receive[1] files: ");
    io::stdout().flush().unwrap(); 
    let mut choice_input = String::new();
    io::stdin().read_line(&mut choice_input).expect("Failed to read choice");
    let choice: i32 = match choice_input.trim().parse() {
        Ok(num) => num,
        Err(_) => {
            println!("Invalid input! Please enter 0 or 1.");
            return;
        }
    };

    if choice==0{
        ws_stream.send(r#"{"type":"get_peers"}"#.into()).await.expect("Failed to send");
        if let Some(Ok(peers)) = ws_stream.next().await {
            println!("{}", peers);
            println!("Select Users:");
            print_keys_from_json_string(peers.to_string());
            println!("Type username to send file to : ");
            io::stdout().flush().unwrap(); 
            let mut target = String::new();
            io::stdin().read_line(&mut target).expect("Failed to read line");
            let target = target.trim();
            let peers_json: Value = serde_json::from_str(&peers.to_string()).unwrap();
            let receiver_addr = format!("{}:{}", peers_json[target]["ipv6_ip"].as_str().unwrap(), peers_json[target]["ipv6_port"]);
            let _ = sender_main(receiver_addr.as_str()).await;
            io::stdout().flush().unwrap(); 
        }
    }
    if choice==1{
        let _ = receiver_main().await;
        io::stdout().flush().unwrap(); 
    }
}