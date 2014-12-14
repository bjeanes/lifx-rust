#![crate_name = "lifx"]

#![allow(dead_code)]

extern crate serialize;

use std::default::Default;
use std::io::{IoResult,Reader,BufReader};

pub struct Message {
  pub size: u16,
  pub version: u16, // encoded in 12 bits
  pub addressable: bool,
  pub tagged: bool,
  pub target: [u8, .. 8],
  pub site: [u8, .. 6],
  pub acknowledge: bool,
  pub at_time: u64,
  pub kind: u16,
  payload: Payload,
}

struct HSBK {
  hue: u16,
  saturation: u16,
  brightness: u16,
  kelvin: u16,
}

enum Payload {
  EMPTY,
  DeviceGetVersion, // 32
  DeviceStateVersion { vendor: u32, product: u32, version: u32 }, // 33
  DeviceGetInfo, // 34
  DeviceStateInfo { time: u64, uptime: u64, downtime: u64 }, // 35
  LightState { color: HSBK, dim: i16, power: u16, label: [u8, ..32], tags: u64 } // 107
}

impl Default for Payload {
  fn default() -> Payload { Payload::EMPTY }
}

impl Default for Message {
  fn default() -> Message {
    Message {
      size: 0,
      version: 0,
      addressable: false,
      tagged: false,
      target: [0, 0, 0, 0, 0, 0, 0, 0],
      site: [0, 0, 0, 0, 0, 0],
      acknowledge: false,
      at_time: 0,
      kind: 0,
      payload: Payload::EMPTY,
    }
  }
}

impl Message {

  pub fn new() -> Message {
    Message { .. Default::default() }
  }

  pub fn from_reader<T: Reader>(reader: &mut T) -> IoResult<Message> {
    let mut mesg = Message::new();

    mesg.size        = try!(reader.read_le_u16());
    let bitfield     = try!(reader.read_le_u16());
    mesg.version     = bitfield & 0b0000111111111111;
    mesg.addressable = bitfield & 0b0001000000000000 > 0;
    mesg.tagged      = bitfield & 0b0010000000000000 > 0;

    let _ = try!(reader.read_le_u32());
    try!(reader.read(mesg.target.as_mut_slice()));
    try!(reader.read(mesg.site.as_mut_slice()));
    mesg.acknowledge = try!(reader.read_le_u16()) & 1 == 1; // only 1st bit used
    mesg.at_time     = try!(reader.read_le_u64());
    mesg.kind        = try!(reader.read_le_u16());
    mesg.payload     = Payload::EMPTY; // FIXME: read actual payload!

    Ok(mesg)
  }

  pub fn from_bytes(bytes: &[u8]) -> IoResult<Message> {
    Message::from_reader(&mut BufReader::new(bytes))
  }
}

#[cfg(test)]
mod test {
  use super::Message;

  #[test]
  fn test_decode() {
    let bytes: &[u8] = &[0x39, 0x00, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00,
      0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x31, 0x6c, 0x69, 0x66, 0x78,
      0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x67,
      0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xac,
      0x0d, 0xc8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x00];

    let mesg = Message::from_bytes(bytes).ok().expect("unable to parse bytes into message");

    assert!(mesg.size == 57)
    assert!(mesg.version == 1024)
    assert!(mesg.target == [0, 0, 0, 0, 0, 0, 0, 0])
    assert!(mesg.site == b"1lifx1")
    assert!(mesg.tagged)
    assert!(mesg.addressable)
    assert!(mesg.kind == 103)
  }
}
