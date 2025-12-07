use std::net::UdpSocket;
use crate::message::OscMessage;
use crate::bundle::OscBundle;

pub struct OscSender {
  socket: UdpSocket,
  destination: String
}

impl OscSender {

  /// Constructs a new OscSender with a given destination ip address and send port
  pub fn new(ip_address: String, port: u32) -> Self {
      Self { 
        socket: UdpSocket::bind("127.0.0.1:0").expect("Failed to bind socket"),
        destination: format!("{}:{}", ip_address, port),
      }
  }

  pub fn send_message(&self, message: &OscMessage) {
      self.socket.send_to(&message.to_bytes(), &self.destination).expect("Failed to send message");
  }

  pub fn send_bundle(&self, bundle: &OscBundle) {
      self.socket.send_to(&bundle.to_bytes(), &self.destination).expect("Failed to send message");
  }
}