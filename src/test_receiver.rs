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
    let mut file_name = None;

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;
        let packet: Packet = match bincode::deserialize(&buf[..len]) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to deserialize packet: {}", e);
                continue;
            }
        };

        // Handle EOF
        if packet.sno == u32::MAX {
            break;
        }

        // Verify checksum
        let calculated_checksum = calculate_checksum(&packet.payload);
        if packet.checksum != calculated_checksum {
            eprintln!("Checksum mismatch for packet {}", packet.sno);
            continue;
        }

        // Send ACK
        let ack = packet.sno.to_be_bytes();
        socket.send_to(&ack, addr).await?;
        println!("Received and ACKed packet {}", packet.sno);

        if packet.sno == 0 {
            file_name = Some(from_utf8(&packet.payload).unwrap().to_string());
        } else {
            packets.insert(packet.sno, packet.payload);
        }
    }

    if let Some(name) = file_name {
        let mut file = File::create(name)?;
        let mut ordered: Vec<_> = packets.into_iter().collect();
        ordered.sort_by_key(|(k, _)| *k);
        for (_, data) in ordered {
            file.write_all(&data)?;
        }
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
