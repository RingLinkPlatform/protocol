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

use crate::{Error, PacketMessage};
use bytes::{Buf, BufMut, Bytes};
use ringlink_identity::DeviceID;

/// p2p binding packet, similar to STUN bind message
pub struct Binding {
    pub from: DeviceID,

    pub body: Bytes,

    pub signature: Bytes,
}

impl PacketMessage for Binding {
    fn encode(self, mut buf: impl BufMut) {
        buf.put_slice(self.from.as_ref());

        buf.put_u32(self.body.len() as u32);
        buf.put(self.body);

        buf.put_u32(self.signature.len() as u32);
        buf.put(self.signature);
    }

    fn decode(mut buf: impl Buf) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let mut from = [0u8; DeviceID::LENGTH];
        (buf.remaining() >= DeviceID::LENGTH)
            .then(|| buf.copy_to_slice(&mut from))
            .ok_or(Error::InsufficientData)?;
        let from = DeviceID::from_bytes(from);

        let body_len = (buf.remaining() >= 4)
            .then(|| buf.get_u32() as usize)
            .ok_or(Error::InsufficientData)?;

        let body = (buf.remaining() >= body_len)
            .then(|| buf.copy_to_bytes(body_len))
            .ok_or(Error::InsufficientData)?;

        let signature_len = (buf.remaining() >= 4)
            .then(|| buf.get_u32() as usize)
            .ok_or(Error::InsufficientData)?;

        let signature = (buf.remaining() >= signature_len)
            .then(|| buf.copy_to_bytes(signature_len))
            .ok_or(Error::InsufficientData)?;

        Ok(Binding {
            from,
            body,
            signature,
        })
    }
}
