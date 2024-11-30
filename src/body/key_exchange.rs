use crate::{Error, PacketMessage};
use bytes::{Buf, BufMut, Bytes};

pub struct KeyExchange {
    /// public key for dh key exchange
    pub public_key: Bytes,
    /// signature make by [ringlink_identity::Identity::sign]
    pub signature: Bytes,
}

impl PacketMessage for KeyExchange {
    fn encode(self, mut buf: impl BufMut) {
        buf.put_u32(self.public_key.len() as u32);
        buf.put(&*self.public_key);
        buf.put_u32(self.signature.len() as u32);
        buf.put(&*self.signature);
    }

    fn decode(mut buf: impl Buf) -> Result<Self, Error>
    where
        Self: Sized,
    {
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
            public_key,
            signature,
        })
    }
}

impl KeyExchange {
    pub fn len(&self) -> usize {
        self.public_key.len() + self.signature.len() + 2 * size_of::<u32>()
    }
}
