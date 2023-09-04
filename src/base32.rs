use std::{fmt::Display, error::Error};

const BASE32_CHARS: [char; 32] = [ '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'M', 'N', 'P', 'Q', 'R', 'S', 'T', 'V', 'W', 'X', 'Y', 'Z' ];

#[derive(Debug)]
#[non_exhaustive]
pub enum Base32DecodeError {
	InvalidChar(char),
}

pub fn encode(bytes: &[u8]) -> String {
	let len = (bytes.len() as f64 * 8.0 / 5.0).ceil() as usize;
	let mut buf = String::with_capacity(len);
	for c in bytes.chunks_exact(5) {
		let indices = [
			(c[0] & 0b11111000) >> 3,
			(c[0] & 0b00000111) << 2 | (c[1] & 0b11000000) >> 6,
			(c[1] & 0b00111110) >> 1,
			(c[1] & 0b00000001) << 4 | (c[2] & 0b11110000) >> 4,
			(c[2] & 0b00001111) << 1 | (c[3] & 0b10000000) >> 7,
			(c[3] & 0b01111100) >> 2,
			(c[3] & 0b00000011) << 3 | (c[4] & 0b11100000) >> 5,
			(c[4] & 0b00011111),
		];

		for i in indices {
			buf.push(BASE32_CHARS[i as usize]);
		}
	}

	let r = bytes.chunks_exact(5).remainder();
	let mut i = [0; 7];
	let len = match r.len() {
		0 => 0,
		1 => {
			i[0] = (r[0] & 0b11111000) >> 3;
			i[1] = (r[0] & 0b00000111) << 2;
			2
		},
		2 => {
			i[0] = (r[0] & 0b11111000) >> 3;
			i[1] = (r[0] & 0b00000111) << 2 | (r[1] & 0b11000000) >> 6;
			i[2] = (r[1] & 0b00111110) >> 1;
			i[3] = (r[1] & 0b00000001) << 4;
			4
		},
		3 => {
			i[0] = (r[0] & 0b11111000) >> 3;
			i[1] = (r[0] & 0b00000111) << 2 | (r[1] & 0b11000000) >> 6;
			i[2] = (r[1] & 0b00111110) >> 1;
			i[3] = (r[1] & 0b00000001) << 4 | (r[2] & 0b11110000) >> 4;
			i[4] = (r[2] & 0b00001111) << 1;
			5
		},
		4 => {
			i[0] = (r[0] & 0b11111000) >> 3;
			i[1] = (r[0] & 0b00000111) << 2 | (r[1] & 0b11000000) >> 6;
			i[2] = (r[1] & 0b00111110) >> 1;
			i[3] = (r[1] & 0b00000001) << 4 | (r[2] & 0b11110000) >> 4;
			i[4] = (r[2] & 0b00001111) << 1 | (r[3] & 0b10000000) >> 7;
			i[5] = (r[3] & 0b01111100) >> 2;
			i[6] = (r[3] & 0b00000011) << 3;
			7
		},
		_ => unreachable!(),
	};

	for i in &i[..len] {
		buf.push(BASE32_CHARS[*i as usize]);
	}

	buf
}

pub fn decode(s: &str) -> Result<Vec<u8>, Base32DecodeError> {
	let mut nums = vec![0; s.len()];
	for c in s.chars() {
		let c = convert_char(c)?;
		let v = BASE32_CHARS.iter().position(|e| *e == c).unwrap();
		nums.push(v as u8);
	}

	let len = (s.len() as f64 * 5.0 / 8.0).ceil() as usize;
	let mut buf = Vec::with_capacity(len);
	for c in nums.chunks_exact(8) {
		let bytes = [
			(c[0] & 0b11111) << 3 | (c[1] & 0b11100) >> 2,
			(c[1] & 0b00011) << 6 | (c[2] & 0b11111) << 1 | (c[3] & 0b10000) >> 4,
			(c[3] & 0b01111) << 4 | (c[4] & 0b11110) >> 1,
			(c[4] & 0b00001) << 7 | (c[5] & 0b11111) << 2 | (c[6] & 0b11000) >> 3,
			(c[6] & 0b00111) << 5 | (c[7] & 0b11111),
		];

		buf.extend_from_slice(&bytes);
	}

	let r = nums.chunks_exact(8).remainder();
	let mut i = [0; 5];
	let len = match r.len() {
		0 => 0,
		1 => {
			i[0] = (r[0] & 0b11111) << 3;
			1
		},
		2 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6;
			2
		},
		3 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6 | (r[2] & 0b11111) << 1;
			2
		},
		4 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6 | (r[2] & 0b11111) << 1 | (r[3] & 0b10000) >> 4;
			i[2] = (r[3] & 0b01111) << 4;
			3
		},
		5 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6 | (r[2] & 0b11111) << 1 | (r[3] & 0b10000) >> 4;
			i[2] = (r[3] & 0b01111) << 4 | (r[4] & 0b11110) >> 1;
			i[3] = (r[4] & 0b00001) << 7;
			4
		},
		6 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6 | (r[2] & 0b11111) << 1 | (r[3] & 0b10000) >> 4;
			i[2] = (r[3] & 0b01111) << 4 | (r[4] & 0b11110) >> 1;
			i[3] = (r[4] & 0b00001) << 7 | (r[5] & 0b11111) << 2;
			4
		},
		7 => {
			i[0] = (r[0] & 0b11111) << 3 | (r[1] & 0b11100) >> 2;
			i[1] = (r[1] & 0b00011) << 6 | (r[2] & 0b11111) << 1 | (r[3] & 0b10000) >> 4;
			i[2] = (r[3] & 0b01111) << 4 | (r[4] & 0b11110) >> 1;
			i[3] = (r[4] & 0b00001) << 7 | (r[5] & 0b11111) << 2 | (r[6] & 0b11000) >> 3;
			i[4] = (r[6] & 0b00111) << 5;
			5
		},
		_ => unreachable!(),
	};

	buf.extend_from_slice(&i[..len]);
	Ok(buf)
}

fn convert_char(c: char) -> Result<char, Base32DecodeError> {
	if !c.is_ascii_alphanumeric() || c.to_ascii_uppercase() == 'U' {
		return Err(Base32DecodeError::InvalidChar(c));
	}

	let c = match c.to_ascii_uppercase() {
		'I' => '1',
		'L' => '1',
		'O' => '0',
		c => c,
	};

	Ok(c)
}

impl Display for Base32DecodeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::InvalidChar(c) => write!(f, "Invalid base 32 character '{c}'"),
		}
	}
}

impl Error for Base32DecodeError {}
