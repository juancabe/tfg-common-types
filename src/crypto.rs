use chacha20poly1305::{
    ChaCha20Poly1305, Nonce, Tag,
    aead::{AeadInPlace, KeyInit},
};
use rand_core::RngCore;
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Debug, defmt::Format)]
pub enum CryptoError {
    /// The buffer provided to `send_to` isn't large enough to hold the Nonce + Payload + MAC.
    BufferTooSmall { required: usize, provided: usize },
    /// The packet provided to `receive_from` is too small to even be a valid packet.
    PacketTooSmall { size: usize },
    /// The hardware Random Number Generator failed to produce bytes.
    RngError,
    /// The AEAD cipher failed to encrypt the data.
    EncryptionFailed,
    /// The MAC didn't match, or the wrong key was used. Hacker alert!
    DecryptionFailed,
}

pub struct Crypto {
    priv_key: StaticSecret,
    public_key: PublicKey,
}

impl Crypto {
    pub fn new(priv_key: StaticSecret) -> Self {
        Self {
            public_key: PublicKey::from(&priv_key),
            priv_key,
        }
    }

    pub fn get_pub_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Encrypts the payload directly into the provided `out_buffer`.
    /// `out_buffer` needs to be 12 + 16 bytes larger than payload
    pub fn send_to(
        &self,
        peer: &PublicKey,
        payload: &[u8],
        out_buffer: &mut [u8],
        rng: &mut impl RngCore,
    ) -> Result<usize, CryptoError> {
        let packet_len = 12 + payload.len() + 16;

        if out_buffer.len() < packet_len {
            return Err(CryptoError::BufferTooSmall {
                required: packet_len,
                provided: out_buffer.len(),
            });
        }

        let shared_secret = self.priv_key.diffie_hellman(peer);
        let key = chacha20poly1305::Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(key);

        let (nonce_section, rest) = out_buffer.split_at_mut(12);
        let (cipher_section, mac_section) = rest.split_at_mut(payload.len());

        rng.try_fill_bytes(nonce_section)
            .map_err(|_| CryptoError::RngError)?;

        let nonce = Nonce::from_slice(nonce_section);

        cipher_section.copy_from_slice(payload);

        let mac = cipher
            .encrypt_in_place_detached(nonce, b"", cipher_section)
            .map_err(|_| CryptoError::EncryptionFailed)?;

        mac_section[..16].copy_from_slice(&mac);

        Ok(packet_len)
    }

    /// Decrypts a packet IN PLACE.
    pub fn receive_from<'a>(
        &self,
        peer: &PublicKey,
        packet: &'a mut [u8],
    ) -> Result<&'a [u8], CryptoError> {
        if packet.len() < 28 {
            return Err(CryptoError::PacketTooSmall { size: packet.len() });
        }

        let shared_secret = self.priv_key.diffie_hellman(peer);
        let key = chacha20poly1305::Key::from_slice(shared_secret.as_bytes());
        let cipher = ChaCha20Poly1305::new(key);

        let payload_len = packet.len() - 28;
        let (nonce_bytes, rest) = packet.split_at_mut(12);
        let (ciphertext, mac_bytes) = rest.split_at_mut(payload_len);

        let nonce = Nonce::from_slice(nonce_bytes);
        let mac = Tag::from_slice(mac_bytes);

        cipher
            .decrypt_in_place_detached(nonce, b"", ciphertext, mac)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        Ok(ciphertext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::{CryptoRng, RngCore};
    use x25519_dalek::StaticSecret;

    // A dummy RNG
    struct MockRng;
    impl RngCore for MockRng {
        fn next_u32(&mut self) -> u32 {
            1
        }
        fn next_u64(&mut self) -> u64 {
            1
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            // Fill with a predictable pattern for testing
            dest.fill(0xAB);
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }

    impl CryptoRng for MockRng {}

    fn setup_peers() -> (Crypto, Crypto) {
        let mut rng = MockRng;
        let alice_secret = StaticSecret::random_from_rng(&mut rng);
        let bob_secret = StaticSecret::random_from_rng(&mut rng);

        (Crypto::new(alice_secret), Crypto::new(bob_secret))
    }

    #[test]
    fn test_roundtrip_success() {
        let (alice, bob) = setup_peers();
        let mut rng = MockRng;

        let original_payload = b"Hello from the ESP32!";
        let mut buffer = [0u8; 128];

        let packet_size = alice
            .send_to(&bob.public_key, original_payload, &mut buffer, &mut rng)
            .expect("Encryption failed");

        let packet = &mut buffer[..packet_size];

        let decrypted = bob
            .receive_from(&alice.public_key, packet)
            .expect("Decryption failed");

        assert_eq!(decrypted, original_payload);
    }

    #[test]
    fn test_buffer_too_small() {
        let (alice, bob) = setup_peers();
        let mut rng = MockRng;

        let payload = b"Data";
        let mut tiny_buffer = [0u8; 10];

        let result = alice.send_to(&bob.public_key, payload, &mut tiny_buffer, &mut rng);

        assert!(matches!(
            result,
            Err(CryptoError::BufferTooSmall {
                required: 32,
                provided: 10
            })
        ));
    }

    #[test]
    fn test_tampering_rejected() {
        let (alice, bob) = setup_peers();
        let mut rng = MockRng;

        let payload = b"Top Secret";
        let mut buffer = [0u8; 64];

        let size = alice
            .send_to(&bob.public_key, payload, &mut buffer, &mut rng)
            .unwrap();
        let packet = &mut buffer[..size];

        // A hacker flips a single bit in the ciphertext or MAC!
        packet[15] ^= 0x01;

        // Bob attempts to decrypt
        let result = bob.receive_from(&alice.public_key, packet);

        // It should fail
        assert!(matches!(result, Err(CryptoError::DecryptionFailed)));
    }
}
