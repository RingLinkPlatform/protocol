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

/// data packet body
pub struct Data {
    pub data: Bytes,
}

impl PacketMessage for Data {
    fn encode(self, mut buf: impl BufMut) {
        buf.put_u64(self.data.len() as u64);
        buf.put(self.data);
    }

    fn decode(mut buf: impl Buf) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let len = (buf.remaining() >= 8)
            .then(|| buf.get_u64() as usize)
            .ok_or(Error::InsufficientData)?;

        let data = (buf.remaining() >= len)
            .then(|| buf.copy_to_bytes(len))
            .ok_or(Error::InsufficientData)?;

        Ok(Data { data })
    }
}

impl Data {
    pub fn len(&self) -> usize {
        self.data.len()
    }
}
