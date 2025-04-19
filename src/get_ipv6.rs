use serde_json::{json, Value};
use std::net::{ToSocketAddrs, UdpSocket, IpAddr};
use stunclient::StunClient;

pub fn print_json(value: &Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                println!("Key: {}", k);
                print_json(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                print_json(v);
            }
        }
        _ => {
            println!("Value: {}", value);
        }
    }
}

pub fn get_pip_port_json(username:&str) -> serde_json::Value {
    let mut ipv4_ip = None;let mut ipv4_port = None;let mut ipv6_ip = None;let mut ipv6_port = None;
    let stun_hostname = "stun.l.google.com:19302"; 

    if let Some(stun_ipv4) = stun_hostname.to_socket_addrs().ok().unwrap().find(|a| a.is_ipv4()) {
        if let Ok(socket_v4) = UdpSocket::bind("0.0.0.0:42069") {
            let client_v4 = StunClient::new(stun_ipv4);
            if let Ok(addr) = client_v4.query_external_address(&socket_v4) {
                ipv4_ip = Some(addr.ip().to_string());
                ipv4_port = Some(addr.port());
            }
        }
    }

    if let Some(stun_ipv6) = stun_hostname.to_socket_addrs().ok().unwrap().find(|a| a.is_ipv6()) {
        if let Ok(socket_v6) = UdpSocket::bind("[::]:42070") {
            let client_v6 = StunClient::new(stun_ipv6);
            if let Ok(addr) = client_v6.query_external_address(&socket_v6) {
                ipv6_ip = Some(addr.ip().to_string());
                ipv6_port = Some(addr.port());
            }
        }
    }
    json!({"type" : "register","username":username,"ipv4_ip": ipv4_ip,"ipv4_port": ipv4_port, "ipv6_ip": ipv6_ip,"ipv6_port": ipv6_port})
}

fn main(){
    let json_string = get_pip_port_json("atharv");
    print_json(&json_string);
}