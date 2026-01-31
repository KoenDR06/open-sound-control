//! **open-sound-control** is an Open Sound Control (OSC) protocol implementation in Rust.
//!
//! Find the code repository at: https://github.com/adamstark/open-sound-control

/// Represents an OSC Argument
pub mod argument;

/// Represents an OSC Message
pub mod message;

/// Represents an OSC Bundle
pub mod bundle;

/// Represents an OSC Time Tag
pub mod timetag;

/// Send OSC messages and bundles over UDP
pub mod sender;

/// Receive OSC messages and bundles over UDP
pub mod receiver;

/// Helper functions and types for parsing OSC data
mod helpers;

#[doc(inline)]
pub use {
    timetag::OscTimeTag,
    argument::OscArgument,
    message::OscMessage,
    bundle::OscBundle,
    sender::OscSender,
    receiver::{OscReceiver, OscPacket},
};