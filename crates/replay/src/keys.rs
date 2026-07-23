use preflight_shared::{Error, KeypairBytes, Result};
use solana_keypair::Keypair;

pub fn keypair_from_bytes(label: &str, bytes: &KeypairBytes) -> Result<Keypair> {
    Keypair::try_from(bytes.0.as_slice()).map_err(|_| Error::InvalidKeypair {
        label: label.to_string(),
    })
}
