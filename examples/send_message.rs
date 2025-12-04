use open_sound_control::*;
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket");

    let m1 = OscMessage {
      address: String::from("/hello"),
      arguments: vec![
        OscArgument::Int32(123),
        OscArgument::String("abc".to_string())
      ]
    };

    socket.send_to(&m1.to_bytes(), "127.0.0.1:8000".to_string()).expect("Failed to send message");
}