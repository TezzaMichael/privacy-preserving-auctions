use sha3::{Digest, Sha3_256};

#[derive(Default, Clone)]
pub struct FiatShamirTranscript {
    state: Vec<u8>,
}

impl FiatShamirTranscript {
    pub fn new() -> Self { Self::default() }

    pub fn domain(&mut self, label: &str) -> &mut Self {
        self.absorb_raw(b"domain", label.as_bytes()); self
    }
    pub fn absorb(&mut self, label: &str, data: &[u8]) -> &mut Self {
        self.absorb_raw(label.as_bytes(), data); self
    }
    pub fn absorb_u64(&mut self, label: &str, v: u64) -> &mut Self {
        self.absorb_raw(label.as_bytes(), &v.to_le_bytes()); self
    }
    pub fn absorb_u128(&mut self, label: &str, v: u128) -> &mut Self {
        self.absorb_raw(label.as_bytes(), &v.to_le_bytes()); self
    }

    pub fn challenge_scalar(&self) -> curve25519_dalek::scalar::Scalar {
        let mut wide = [0u8; 64];
        wide[..32].copy_from_slice(&self.hash_tag(b"challenge-lo"));
        wide[32..].copy_from_slice(&self.hash_tag(b"challenge-hi"));
        curve25519_dalek::scalar::Scalar::from_bytes_mod_order_wide(&wide)
    }

    pub fn challenge_bytes(&self) -> [u8; 32] { self.hash_tag(b"challenge-raw") }

    fn absorb_raw(&mut self, label: &[u8], data: &[u8]) {
        self.state.extend_from_slice(&(label.len() as u32).to_le_bytes());
        self.state.extend_from_slice(label);
        self.state.extend_from_slice(&(data.len() as u64).to_le_bytes());
        self.state.extend_from_slice(data);
    }

    fn hash_tag(&self, tag: &[u8]) -> [u8; 32] {
        let mut h = Sha3_256::new();
        h.update(b"auction-fiat-shamir-v1:");
        h.update(tag);
        h.update(b":");
        h.update((self.state.len() as u64).to_le_bytes());
        h.update(&self.state);
        h.finalize().into()
    }
}

impl std::fmt::Debug for FiatShamirTranscript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FiatShamirTranscript {{ state_len: {} }}", self.state.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(domain: &str, label: &str, data: &[u8]) -> FiatShamirTranscript {
        let mut t = FiatShamirTranscript::new();
        t.domain(domain).absorb(label, data);
        t
    }

    #[test] fn deterministic() {
        assert_eq!(t("d","x",b"hello").challenge_bytes(), t("d","x",b"hello").challenge_bytes());
    }
    #[test] fn idempotent() {
        let tr = t("d","x",b"v");
        assert_eq!(tr.challenge_bytes(), tr.challenge_bytes());
    }
    #[test] fn sensitive_to_data() {
        assert_ne!(t("d","x",b"a").challenge_bytes(), t("d","x",b"b").challenge_bytes());
    }
    #[test] fn sensitive_to_domain() {
        assert_ne!(t("A","x",b"v").challenge_bytes(), t("B","x",b"v").challenge_bytes());
    }
    #[test] fn sensitive_to_order() {
        let mut t1 = FiatShamirTranscript::new(); t1.domain("d").absorb("a",b"1").absorb("b",b"2");
        let mut t2 = FiatShamirTranscript::new(); t2.domain("d").absorb("b",b"2").absorb("a",b"1");
        assert_ne!(t1.challenge_bytes(), t2.challenge_bytes());
    }
    #[test] fn label_length_disambiguates() {
        let mut t1 = FiatShamirTranscript::new(); t1.domain("d").absorb("ab",b"c");
        let mut t2 = FiatShamirTranscript::new(); t2.domain("d").absorb("a",b"bc");
        assert_ne!(t1.challenge_bytes(), t2.challenge_bytes());
    }
    #[test] fn u64_matches_le_bytes() {
        let mut t1 = FiatShamirTranscript::new(); t1.domain("d").absorb_u64("v", 99);
        let mut t2 = FiatShamirTranscript::new(); t2.domain("d").absorb("v", &99u64.to_le_bytes());
        assert_eq!(t1.challenge_bytes(), t2.challenge_bytes());
    }
    #[test] fn scalar_nonzero() {
        use curve25519_dalek::scalar::Scalar;
        let mut tr = FiatShamirTranscript::new(); tr.domain("t").absorb("x",b"data");
        assert_ne!(tr.challenge_scalar(), Scalar::ZERO);
    }
    #[test] fn bytes_and_scalar_differ() {
        let mut tr = FiatShamirTranscript::new(); tr.domain("t").absorb("x",b"data");
        assert_ne!(tr.challenge_bytes(), tr.challenge_scalar().to_bytes());
    }
}