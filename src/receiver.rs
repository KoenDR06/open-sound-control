use std::net::UdpSocket;
use crate::helpers::{self, OscParseError};
use crate::message::OscMessage;
use crate::bundle::OscBundle;

/// Struct for receiving OSC messages and bundles over UDP
pub struct OscReceiver {
  socket: UdpSocket
}

/// Represents either an OSC Message or an OSC Bundle
pub enum OscPacket {
  Message(OscMessage),
  Bundle(OscBundle)
}

#[derive(Debug, PartialEq)]
pub enum OscNetworkError {
  CouldNotBindToSocket
}

impl OscReceiver {

  /// Constructs a new OscReceiver listening on the given port
  pub fn new(port: u32) -> Result<Self, OscNetworkError> {
    let addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(addr)
      .map_err(|_| OscNetworkError::CouldNotBindToSocket)?;

    Ok (OscReceiver { socket })
  }

  /// Receives OSC Messages or Bundles from the UDP socket
  pub fn get_messages(&self) -> Result<OscPacket, OscParseError> {

    let mut buf = [0u8; 2048];

    match self.socket.recv_from(&mut buf) {
      Ok((number_of_bytes, _source_address)) => {
        let bytes = &mut buf[..number_of_bytes];
        if helpers::is_bundle(&bytes) {
          let bundle = OscBundle::from_bytes(&bytes)?;
          return Ok(OscPacket::Bundle(bundle));
        } else {
          let message = OscMessage::from_bytes(&bytes)?;
          return Ok(OscPacket::Message(message));
        }
      },
      Err(_) => return Err(OscParseError::NotEnoughData),
    }
  }
}
