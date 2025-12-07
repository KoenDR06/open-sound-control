use open_sound_control::*;

fn main() {
    let sender = OscSender::new("127.0.0.1".to_string(), 9000);
    
    //--------------------------------------------------------------
    // Send an OSC Message 

    let m1 = OscMessage {
      address: String::from("/hello"),
      arguments: vec![
        OscArgument::Int32(123),
        OscArgument::String("abc".to_string())
      ]
    };

    sender.send_message (&m1);


    //--------------------------------------------------------------
    // Send an OSC Bundle
    
    let m2 = OscMessage {
      address: String::from("/frequency"),
      arguments: vec![
        OscArgument::Float32(440.0),
      ]
    };

    let m3 = OscMessage {
      address: String::from("/instrument"),
      arguments: vec![
        OscArgument::String("drums".to_string()),
        OscArgument::Int32(4)
      ]
    };

    let b1 = OscBundle {
      time_tag: OscTimeTag::now(),
      messages: vec![m2, m3]
    };

    sender.send_bundle (&b1);
}