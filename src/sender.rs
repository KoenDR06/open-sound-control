use std::io;
use std::net::UdpSocket;
use crate::message::OscMessage;
use crate::bundle::OscBundle;

/// Struct for sending OSC messages and bundles over UDP
pub struct OscSender {
  socket: UdpSocket,
  destination: String
}

impl OscSender {

  /// Constructs a new OscSender with a given destination ip address and send port
  pub fn new(ip_address: String, port: u32) -> Result<Self, io::Error> {
      Ok(Self { 
        socket: UdpSocket::bind("127.0.0.1:0")?,
        destination: format!("{}:{}", ip_address, port),
      })
  }

  /// Sends an OSC Message to the destination
  pub fn send_message(&self, message: &OscMessage) -> Result<(), io::Error> {
      self.socket.send_to(&message.to_bytes(), &self.destination)?;

      Ok(())
  }

  /// Sends an OSC Bundle to the destination
  pub fn send_bundle(&self, bundle: &OscBundle) -> Result<(), io::Error> {
      self.socket.send_to(&bundle.to_bytes(), &self.destination)?;

      Ok(())
  }
}
