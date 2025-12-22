
use open_sound_control::*;

fn main() {
  let receiver = OscReceiver::new(9000).unwrap();

  loop {
      match receiver.get_messages() {
          Ok(OscPacket::Message(msg)) => println!("Got message: {:?}", msg.address),
          Ok(OscPacket::Bundle(bundle)) => println!("Got bundle: {:?}", bundle.time_tag.seconds),
          Err(err) => eprintln!("Parse error: {:?}", err),
      }
  }
}