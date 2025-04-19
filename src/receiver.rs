use tokio::net::TcpListener;
use std::fs::File;
use std::io::Write;
use std::str::from_utf8;
use serde::{Serialize, Deserialize};
use bincode;
use crc16::*;
use tokio::io::{AsyncReadExt, BufReader};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub header: u64,
    pub sno: u32,
    pub payload_length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}


pub async fn receiver_main() -> tokio::io::Result<()> {
    let listener = TcpListener::bind("[::]:42070").await?;
    println!("TCP server waiting for connection...");

    let (mut socket, _) = listener.accept().await?;
    let mut reader = BufReader::new(&mut socket);
    let mut packets = Vec::new();

    loop {
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).await.is_err() {
            break; 
        }
        let packet_len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; packet_len];
        reader.read_exact(&mut buf).await?;

        let packet: Packet = bincode::deserialize(&buf).unwrap();
        let calculated_checksum = calculate_checksum(&packet.payload);
        if packet.checksum != calculated_checksum {
            eprintln!("Checksum mismatch for packet {}", packet.sno);
            continue;
        }

        packets.push(packet.clone());
        println!("Received packet {}", packet.sno);

        if packet.sno != 0 && packet.payload_length < 1024 {
            break;
        }
    }

    if !packets.is_empty() {
        packets_to_file(packets);
        println!("File saved successfully!");
    }
    Ok(())
}

pub fn packets_to_file(mut packets: Vec<Packet>) {
    let file_name = from_utf8(&packets[0].payload).unwrap();
    let mut file = File::create(file_name).expect("Failed to create file");
    packets[1..].sort_by(|a, b| a.sno.cmp(&b.sno));
    for packet in &packets[1..] {
        file.write_all(&packet.payload).expect("Failed to write data");
    }
}

pub fn calculate_checksum(data: &[u8]) -> u16 {
    State::<ARC>::calculate(data)
}

fn main(){}