use crate::{Error, PacketMessage};
use bytes::{Buf, BufMut, Bytes};

pub const KEY_EXCHANGE_REQUEST: u8 = 0x01;
pub const KEY_EXCHANGE_REPLY: u8 = 0x02;

#[derive(Clone)]
pub struct KeyExchange {
    /// type of key exchange message
    pub typ: u8,
    /// public key for dh key exchange
    pub public_key: Bytes,
    /// signature make by [ringlink_identity::Identity::sign]
    pub signature: Bytes,
}

impl PacketMessage for KeyExchange {
    fn encode(self, mut buf: impl BufMut) {
        buf.put_u8(self.typ);
        buf.put_u32(self.public_key.len() as u32);
        buf.put(&*self.public_key);
        buf.put_u32(self.signature.len() as u32);
        buf.put(&*self.signature);
    }

    fn decode(mut buf: impl Buf) -> Result<Self, Error>
    where
        Self: Sized,
    {
        let typ = (buf.remaining() >= 1)
            .then(|| buf.get_u8())
            .ok_or(Error::InsufficientData)?;

        let public_key_len = (buf.remaining() >= 4)
            .then(|| buf.get_u32() as usize)
            .ok_or(Error::InsufficientData)?;
        let public_key = (buf.remaining() >= public_key_len)
            .then(|| buf.copy_to_bytes(public_key_len))
            .ok_or(Error::InsufficientData)?;

        let signature_len = (buf.remaining() >= 4)
            .then(|| buf.get_u32() as usize)
            .ok_or(Error::InsufficientData)?;
        let signature = (buf.remaining() >= signature_len)
            .then(|| buf.copy_to_bytes(signature_len))
            .ok_or(Error::InsufficientData)?;

        Ok(KeyExchange {
            typ,
            public_key,
            signature,
        })
    }
}

impl KeyExchange {
    pub fn len(&self) -> usize {
        self.public_key.len() + self.signature.len() + 2 * size_of::<u32>() + size_of::<u8>()
    }
}
