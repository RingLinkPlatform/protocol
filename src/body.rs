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

//! packet body defines
use bytes::{Buf, BufMut};

mod data;
mod key_exchange;
mod p2p;

pub use data::Data;
pub use key_exchange::{KeyExchange, KEY_EXCHANGE_REPLY, KEY_EXCHANGE_REQUEST};
pub use p2p::Binding;

/// trait for encode and decode RingLink packet
pub trait PacketMessage {
    /// encode a packet into a buffer
    fn encode(self, buf: impl BufMut);

    /// decode a packet from buffer
    fn decode(buf: impl Buf) -> Result<Self, super::error::Error>
    where
        Self: Sized;
}
