/*
 * Copyright 2024 RingNet
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 *  limitations under the License.
 *
 */

//! RingLink protocols
//!
//! the packet format used in RingLink platform
//!
use bitflags::bitflags;
use bytes::{Buf, BufMut, BytesMut};
use std::mem::size_of;
use std::sync::atomic::{AtomicU64, Ordering};

pub use body::PacketMessage;
pub use error::Error;
pub use id::NetId;
use ringlink_identity::DeviceID;

pub mod body;
mod error;
mod id;

/// default packet id sequence
static PACKET_ID: AtomicU64 = AtomicU64::new(0);
/// default packet ttl
const DEFAULT_TTL: u8 = 0b111;

#[repr(u8)]
#[non_exhaustive]
pub enum PacketKind {
    /// main data packet
    Data = 0x01,
    /// key exchange packet
    KeyExchange = 0x10,
    /// p2p
    ///
    /// the detail of p2p message is not this package's concern
    P2P = 0x06,
}

impl TryFrom<u8> for PacketKind {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(PacketKind::Data),
            0x10 => Ok(PacketKind::KeyExchange),
            0x06 => Ok(PacketKind::P2P),
            _ => Err(Error::UnknownKind),
        }
    }
}

/// packet body
#[non_exhaustive]
pub enum PacketBody {
    Data(body::Data),
    KeyExchange(body::KeyExchange),
    P2P(body::Binding),
}

bitflags! {
    #[derive(Copy, Clone, Eq, PartialEq)]
    pub struct PacketFlags: u32 {
        /// reserved all bits
        const _ = !0;
    }
}

pub struct PacketHeader {
    /// 3bit ttl + unique id of packet
    pub packet_id: u64,
    // packet kind
    pub kind: PacketKind,
    /// source device id
    pub from: DeviceID,
    /// destination device id
    pub to: DeviceID,
    /// packet flags
    pub flags: PacketFlags,
}

/// full packet
pub struct Packet {
    pub header: PacketHeader,
    // packet body
    pub body: PacketBody,
}

impl Packet {
    /// create a new packet with body
    ///
    /// packet id will be generated automatically
    pub fn new(from: DeviceID, to: DeviceID, body: PacketBody) -> Packet {
        let id = PACKET_ID.fetch_add(1, Ordering::Relaxed);

        Self::with_id(id, from, to, body)
    }

    /// create a new packet with custom id and body
    pub fn with_id(id: u64, from: DeviceID, to: DeviceID, body: PacketBody) -> Packet {
        let kind = match &body {
            PacketBody::Data(_) => PacketKind::Data,
            PacketBody::KeyExchange(_) => PacketKind::KeyExchange,
            PacketBody::P2P(_) => PacketKind::P2P,
        };

        Packet {
            header: PacketHeader {
                packet_id: id | ((DEFAULT_TTL as u64) << 61),
                kind,
                from,
                to,
                flags: PacketFlags::empty(),
            },
            body,
        }
    }

    /// Get packet total length, include header and body
    pub fn len(&self) -> usize {
        PacketHeader::len() + self.body.len()
    }

    /// Encode packet into bytes
    pub fn encode_into_bytes(self) -> BytesMut {
        let mut buff = BytesMut::with_capacity(self.len());
        self.encode(&mut buff);

        buff
    }
}

impl PacketMessage for Packet {
    fn encode(self, mut buf: impl BufMut) {
        buf.put_u64(self.header.packet_id);
        buf.put_u8(self.header.kind as u8);
        buf.put(&*self.header.from);
        buf.put(&*self.header.to);
        buf.put_u32(self.header.flags.bits());

        match self.body {
            PacketBody::Data(body) => body.encode(buf),
            PacketBody::KeyExchange(body) => body.encode(buf),
            PacketBody::P2P(body) => body.encode(buf),
        }
    }

    fn decode(mut buf: impl Buf) -> Result<Self, Error>
    where
        Self: Sized,
    {
        // ensure packet have enough data
        if buf.remaining() < PacketHeader::len() {
            return Err(Error::InsufficientData);
        }

        let id = buf.get_u64();
        let kind = buf.get_u8();
        let kind = PacketKind::try_from(kind)?;

        let mut from = [0u8; DeviceID::LENGTH];
        let mut to = [0u8; DeviceID::LENGTH];
        buf.copy_to_slice(&mut from);
        buf.copy_to_slice(&mut to);

        let flags = buf.get_u32();
        let flags = PacketFlags::from_bits_truncate(flags);

        let body = match kind {
            PacketKind::Data => PacketBody::Data(body::Data::decode(buf)?),
            PacketKind::KeyExchange => PacketBody::KeyExchange(body::KeyExchange::decode(buf)?),
            PacketKind::P2P => PacketBody::P2P(body::Binding::decode(buf)?),
        };

        Ok(Packet {
            header: PacketHeader {
                packet_id: id,
                kind,
                from: DeviceID::from_bytes(from),
                to: DeviceID::from_bytes(to),
                flags,
            },
            body,
        })
    }
}

impl PacketHeader {
    /// Get packet header length
    pub const fn len() -> usize {
        size_of::<u64>() + size_of::<u8>() + 2 * DeviceID::LENGTH + size_of::<u32>()
    }

    /// Get packet ttl
    pub const fn ttl(&self) -> u8 {
        ((self.packet_id >> 61) & 0b111) as u8
    }
}

impl PacketBody {
    /// Get packet body length
    pub fn len(&self) -> usize {
        match self {
            PacketBody::Data(data) => data.len(),
            PacketBody::KeyExchange(kex) => kex.len(),
            PacketBody::P2P(_) => 0,
        }
    }
}
