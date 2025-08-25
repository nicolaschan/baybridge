use bincode::config::standard;
use bincode::{Decode, Encode};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};

pub trait Signable: Clone + Encode + Serialize {}

#[derive(Clone, Serialize, Deserialize, Encode, Decode)]
pub struct Signed<T: Signable> {
    pub inner: T,
    verifying_key: [u8; PUBLIC_KEY_LENGTH],
    signature: Vec<u8>,
}

impl<T: Signable> Signed<T> {
    pub fn new(inner: T, verifying_key: VerifyingKey, signature: Signature) -> Self {
        Signed {
            inner,
            verifying_key: verifying_key.to_bytes(),
            signature: signature.to_bytes().to_vec(),
        }
    }

    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        let serialized = bincode::encode_to_vec(&self.inner, standard()).unwrap();
        verifying_key
            .verify_strict(&serialized, &self.signature())
            .is_ok()
    }

    pub fn verifying_key(&self) -> VerifyingKey {
        VerifyingKey::from_bytes(&self.verifying_key).unwrap()
    }

    pub fn signature(&self) -> Signature {
        Signature::from_bytes(&self.signature.as_slice().try_into().unwrap())
    }
}
