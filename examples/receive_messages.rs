
use open_sound_control::*;

fn main() {
    let listen_port = 9000;
    let receiver = OscReceiver::new(listen_port).unwrap();

    loop {
        match receiver.get_messages() {
            Ok(OscPacket::Message(msg)) => println!("Got message: {}", msg.to_string()),
            Ok(OscPacket::Bundle(bundle)) => println!("Got bundle: {:?}", bundle.time_tag.seconds),
            Err(err) => eprintln!("Parse error: {:?}", err),
        }
    }
}