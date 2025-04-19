use tokio::net::TcpStream;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde::{Serialize, Deserialize};
use bincode;
use rfd::FileDialog;
use crc16::*;
use tokio::io::AsyncWriteExt;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub header: u64,
    pub sno: u32,
    pub payload_length: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

pub async fn send_file_tcp(file_path: &Path, server_addr: &str) -> tokio::io::Result<()> {
    let mut stream = TcpStream::connect(server_addr).await?;
    let packets = file_to_packets(file_path);
    for packet in packets {
        let encoded = bincode::serialize(&packet).unwrap();
        let len = (encoded.len() as u32).to_be_bytes(); 
        stream.write_all(&len).await?;
        stream.write_all(&encoded).await?;
    }
    println!("File sent via TCP.");
    Ok(())
}

pub fn file_to_packets(file_path: &Path) -> Vec<Packet> {
    let mut file = File::open(file_path).expect("Failed to open file");
    let mut packets = Vec::new();
    let mut buffer = [0; 1024];
    let mut seq_num = 1;

    let file_name = file_path.file_name().unwrap().to_str().unwrap();
    let initial_packet = Packet {
        header: 0x12345678ABCDEF00,
        sno: 0,
        payload_length: file_name.len() as u16,
        checksum: calculate_checksum(file_name.as_bytes()),
        payload: file_name.as_bytes().to_vec(),
    };
    packets.push(initial_packet);

    while let Ok(bytes_read) = file.read(&mut buffer) {
        if bytes_read == 0 { break; }

        let packet = Packet {
            header: 0x12345678ABCDEF00,
            sno: seq_num,
            payload_length: bytes_read as u16,
            checksum: calculate_checksum(&buffer[..bytes_read]),
            payload: buffer[..bytes_read].to_vec(),
        };
        packets.push(packet);
        seq_num += 1;
    }

    packets
}

pub fn calculate_checksum(data: &[u8]) -> u16 {
    State::<ARC>::calculate(data)
}


pub async fn sender_main(receiver_addr:&str) {
    let file_path = FileDialog::new().pick_file();
    if let Some(path) = file_path {
        send_file_tcp(&path, receiver_addr).await.unwrap();
    } else {
        println!("No file selected.");
    }
}

fn main(){}
