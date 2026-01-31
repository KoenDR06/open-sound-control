# Open Sound Control

<!-- Version and License Badges -->

![Version](https://img.shields.io/badge/version-0.9.0-green.svg?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)
![Language](https://img.shields.io/badge/language-Rust-yellow.svg?style=flat-square)

**open-sound-control** is a Rust implementation of the OSC (Open Sound Control) protocol.

## Installation

Add `open-sound-control` to your project's Cargo.toml:

```
[dependencies]
open-sound-control = "0.9.0"
```

then run:

```
cargo build
```

## Usage

Import the library to your Rust code:

```
use open_sound_control::*;
```

### Sending Messages

```
// create a sender with destination IP and port
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
```

### Sending Bundles

```
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
```

### Receiving Messages

```
let listen_port = 9000;
let receiver = OscReceiver::new(listen_port).unwrap();

loop {
    match receiver.get_messages() {
        Ok(OscPacket::Message(msg)) => println!("Got message: {}", msg.to_string()),
        Ok(OscPacket::Bundle(bundle)) => println!("Got bundle: {:?}", bundle.time_tag.seconds),
        Err(err) => eprintln!("Parse error: {:?}", err),
    }
}
```

## Licence

MIT License

Copyright (c) 2025 Adam Stark

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
