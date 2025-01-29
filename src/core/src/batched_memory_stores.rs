//
// Why keep unwrap/expect here? The mutex handles a single thread, so really this could be
// a HashMap. We leave it as is for similarity to the privacypass-rust example code.
// In this context, a thread that panics durinc access to the Mutex (and hence makes .lock() fail)
// is the only thread, and a further panic in this file will never be triggered.
// Just in case, we add a message to the panic.

use async_trait::async_trait;
use p384::NistP384;
use privacypass::{Nonce, NonceStore, TruncatedTokenKeyId};
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use voprf::{Ristretto255, VoprfServer};

#[derive(Default)]
pub struct MemoryNonceStore {
    nonces: Mutex<HashSet<Nonce>>,
}

#[async_trait]
impl NonceStore for MemoryNonceStore {
    async fn exists(&self, nonce: &Nonce) -> bool {
        let nonces = self
            .nonces
            .lock()
            .expect("MemoryNonceStore .lock() failed on .exists()");
        nonces.contains(nonce)
    }

    async fn insert(&self, nonce: Nonce) {
        let mut nonces = self
            .nonces
            .lock()
            .expect("MemoryNonceStore .lock() failed on .insert()");
        nonces.insert(nonce);
    }
}

#[derive(Default)]
pub struct MemoryKeyStoreRistretto255 {
    keys: Mutex<HashMap<TruncatedTokenKeyId, VoprfServer<Ristretto255>>>,
}

#[async_trait]
impl privacypass::batched_tokens_ristretto255::server::BatchedKeyStore
    for MemoryKeyStoreRistretto255
{
    async fn insert(
        &self,
        truncated_token_key_id: TruncatedTokenKeyId,
        server: VoprfServer<Ristretto255>,
    ) {
        let mut keys = self
            .keys
            .lock()
            .expect("MemoryKeyStoreRistretto255 .lock() failed on .insert()");
        keys.insert(truncated_token_key_id, server);
    }

    async fn get(
        &self,
        truncated_token_key_id: &TruncatedTokenKeyId,
    ) -> Option<VoprfServer<Ristretto255>> {
        self.keys
            .lock()
            .expect("MemoryKeyStoreRistretto255 .lock() failed on .get()")
            .get(truncated_token_key_id)
            .cloned()
    }
}

#[derive(Default)]
pub struct MemoryKeyStoreP384 {
    keys: Mutex<HashMap<TruncatedTokenKeyId, VoprfServer<NistP384>>>,
}

#[async_trait]
impl privacypass::batched_tokens_p384::server::BatchedKeyStore for MemoryKeyStoreP384 {
    async fn insert(
        &self,
        truncated_token_key_id: TruncatedTokenKeyId,
        server: VoprfServer<NistP384>,
    ) {
        let mut keys = self
            .keys
            .lock()
            .expect("MemoryKeyStoreP384 .lock() failed on .insert()");
        keys.insert(truncated_token_key_id, server);
    }

    async fn get(
        &self,
        truncated_token_key_id: &TruncatedTokenKeyId,
    ) -> Option<VoprfServer<NistP384>> {
        self.keys
            .lock()
            .expect("MemoryKeyStoreP384 .lock() failed on .get()")
            .get(truncated_token_key_id)
            .cloned()
    }
}
