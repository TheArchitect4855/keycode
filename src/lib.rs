pub mod base32;
use std::{time::{SystemTime, Duration, SystemTimeError}, fmt::Display};
use rand_chacha::ChaCha20Rng;
use rand::{SeedableRng, RngCore};
use sha2::{Sha512, Digest};

const KEY_TTL_SECONDS: u64 = 60;

#[derive(Debug)]
pub struct Code([u8; 64], u64);

#[derive(Debug)]
pub struct Key(Vec<u8>);

impl Code {
	pub fn ttl(&self) -> Duration {
		Duration::from_secs(self.1)
	}
}

impl Key {
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	pub fn code(&self, time: SystemTime) -> Result<Code, SystemTimeError> {
		let iat = time.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
		let time = iat / KEY_TTL_SECONDS;
		let exp = (time + 1) * KEY_TTL_SECONDS;
		let ttl = exp - iat;
		let time = time.to_be_bytes();

		let mut hash = Sha512::new();
		hash.update(self.as_bytes());
		hash.update(&time);
		let code = hash.finalize();
		Ok(Code(code.into(), ttl))
	}

	pub fn generate() -> Self {
		let mut rng = ChaCha20Rng::from_entropy();
		let mut buf = vec![0; 256];
		rng.fill_bytes(&mut buf);
		Self(buf)
	}

	pub fn import(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}
}

impl Display for Code {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", base32::encode(&self.0[..5]))
	}
}
