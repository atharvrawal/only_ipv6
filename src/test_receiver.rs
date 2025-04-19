use tokio::net::UdpSocket;
use std::fs::File;
use std::io::Write;
use std::str::from_utf8;
use serde::{Serialize, Deserialize};
use bincode;
use crc16::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub header: u64,
    pub sno: u32,
    pub payload_length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

pub async fn receiver_main() -> tokio::io::Result<()> {
    let socket = UdpSocket::bind("[::]:42070").await?;
    println!("UDP server waiting for packets...");

    let mut buf = [0u8; 1500];
    let mut packets = HashMap::new();
    let mut sender = None;

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        sender.get_or_insert(addr); // Save sender once

        let packet: Packet = bincode::deserialize(&buf[..len]).unwrap();
        let calculated_checksum = calculate_checksum(&packet.payload);
        if packet.checksum != calculated_checksum {
            eprintln!("Checksum mismatch for packet {}", packet.sno);
            continue;
        }

        // Send ACK
        let ack = packet.sno.to_be_bytes();
        socket.send_to(&ack, addr).await?;

        packets.insert(packet.sno, packet.clone());
        println!("Received packet {}", packet.sno);

        if packet.sno != 0 && packet.payload_length < 1024 {
            break;
        }
    }

    if !packets.is_empty() {
        let mut ordered: Vec<_> = packets.into_values().collect();
        ordered.sort_by_key(|p| p.sno);
        packets_to_file(ordered);
        println!("File saved successfully!");
    }
    Ok(())
}

pub fn packets_to_file(mut packets: Vec<Packet>) {
    let file_name = from_utf8(&packets[0].payload).unwrap();
    let mut file = File::create(file_name).expect("Failed to create file");
    for packet in &packets[1..] {
        file.write_all(&packet.payload).expect("Failed to write data");
    }
}

pub fn calculate_checksum(data: &[u8]) -> u16 {
    State::<ARC>::calculate(data)
}

fn main(){}
