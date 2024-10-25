use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::serde_as;

pub trait Signable: Clone + Serialize {}

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "SerializableSigned<T>", into = "SerializableSigned<T>")]
pub struct Signed<T: Signable> {
    pub inner: T,
    pub verifying_key: VerifyingKey,
    pub signature: Signature,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
pub struct SerializableSigned<T> {
    pub inner: T,
    #[serde_as(as = "Base64")]
    pub verifying_key: Vec<u8>,
    #[serde_as(as = "Base64")]
    pub signature: Vec<u8>,
}

impl<T: Signable> Signed<T> {
    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        let serialized = bincode::serialize(&self.inner).unwrap();
        verifying_key
            .verify_strict(&serialized, &self.signature)
            .is_ok()
    }
}

impl<T: Signable> From<Signed<T>> for SerializableSigned<T> {
    fn from(value: Signed<T>) -> Self {
        let verifying_key = value.verifying_key.to_bytes().to_vec();
        let signature = value.signature.to_bytes().to_vec();
        SerializableSigned {
            inner: value.inner,
            verifying_key,
            signature,
        }
    }
}

impl<T: Signable> From<SerializableSigned<T>> for Signed<T> {
    fn from(value: SerializableSigned<T>) -> Self {
        let verifying_key =
            VerifyingKey::from_bytes(&value.verifying_key.try_into().unwrap()).unwrap();
        let signature = Signature::from_bytes(&value.signature.try_into().unwrap());
        Signed {
            inner: value.inner,
            verifying_key,
            signature,
        }
    }
}
