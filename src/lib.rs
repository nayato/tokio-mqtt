//#![allow(unused_imports, dead_code)]

extern crate futures;
extern crate tokio_io;
// extern crate tokio_proto;
// extern crate tokio_service;

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;

// mod errors {
//     // Create the Error, ErrorKind, ResultExt, and Result types
//     error_chain! { }
// }

use bytes::{Buf};
use tokio_io::codec::{Decoder, Encoder};
use std::io;

mod packet;

use packet::{Packet, QoS};
// use errors::*;

pub struct MqttCodec;

type OptRes<T> = Result<Option<T>, io::Error>;

impl MqttCodec {
    fn parse_var_len(&self, data: &[u8]) -> OptRes<(u32, usize)> {
        let mut shift = 0;
        let mut value: u32 = 0;
        let mut idx: usize = 0;
        loop {
            let enc = data[idx];
            value += ((enc & 127) << shift) as u32;

            if enc & 128 == 0 {
                return Ok(Some((value, idx + 1)));
            };

            shift += 7;
            idx += 1;
            if idx > 3 {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                                          "Remaining length exceeds 4 bytes in length "));
            }

            if data.len() == idx {
                return Ok(None); // not enough data to parse remaining length
            }
        }
    }

    fn parse_packet_id(&self, data: &[u8]) -> OptRes<u16> {
        Ok(None)
    }

    fn len_str(data: &mut [u8]) -> OptRes<String>{
        Ok(None)
    }

    fn parse_publish(&self, flags: u8, data: &[u8]) -> OptRes<Packet> {
        let dup = (flags & 8) == 1;
        let retain = (flags & 1) == 1;
        let qos_byte = ((flags & 4) << 1) + (flags & 2);
        let qos = QoS::from_u8(qos_byte)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Unexpected QoS value"))?;
        let topic = len_str(&mut remaining)?;

        let packet_id = match qos {
            QoS::QoS0 => None,
            _ => Some(try!(big_endian(&mut remaining))),
        };

        let offset = 2 + topic.len() +
                     match packet_id {
            Some(..) => 2,
            None => 0,
        };

        let payload = try!(arr(&mut remaining, remaining_length - offset));
        Ok(Some(Packet::PUBLISH {
            dup: dup,
            message: Message {
                topic: Topic::new_owned(topic),
                payload: Payload::new_owned(payload),
                qos: qos,
                retain: retain,
            },
            packet_id: packet_id.map(|pid| PacketId(pid)),
        }))
    }
}

impl Codec for MqttCodec {
    type In = Packet;
    type Out = Packet;

    fn decode(&mut self, buf: &mut EasyBuf) -> OptRes<Packet> {
        if buf.len() < 2 {
            return Ok(None); // min packet size is 2 bytes
        }

        let (&signature, input) = buf.as_ref().split_first().unwrap();
        if let Some((len, len_consumed)) = self.parse_var_len(&input)? {
            let input = &input[len_consumed + 1..];
            return match signature {
                sig if sig & (0xf << 4) == (3 << 4) => self.parse_publish(sig, &input),
                64 => Ok((self.parse_packet_id(&input)?).map(|id| Packet::PUBACK)),
                _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected fixed ")), // todo: ::Other instead?
            };
        } else {
            return Ok(None);
        }


        // let b = buf.as_slice();
        // let i = b.iter();
        // let len = buf.len();
        // // if ()
        // // buf.drain_to(len);
        // Ok(None)
        // // match buf.as_slice().iter().position(|&b| b == b'\n') {
        // //     Some(i) => Ok(Some(buf.drain_to(i + 1).into())),
        // //     None => Ok(None),
        // // }
    }

    fn decode_eof(&mut self, buf: &mut EasyBuf) -> io::Result<Packet> {
        // let amt = buf.len();
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "failed to parse packet"))
        // Ok(buf.drain_to(amt))
    }

    fn encode(&mut self, item: Packet, into: &mut Vec<u8>) -> io::Result<()> {
        into.push(1u8);
        // into.extend_from_slice(item.as_slice());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let x = 1;
    }
}
