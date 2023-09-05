mod args;
use std::{error::Error, process, fs::{File, OpenOptions}, io::{self, Write, Read, Seek, SeekFrom}, time::SystemTime};
use args::*;
use clap::Parser;
use keycode::{Key, base32};

fn main() {
	let args = Cli::parse();
	let res = match args.command {
		Command::Generate { name } => generate(name),
		Command::Get { name } => get(name),
		Command::List => list(),
	};

	if let Err(e) = res {
		eprintln!("Error: {e}");
		process::exit(1);
	}
}

fn generate(name: String) -> Result<(), Box<dyn Error>> {
	if name.len() > 255 {
		return Err("Name length must be less than 255 bytes".into());
	}

	let key = Key::generate();
	let mut keyfile = open_keyfile(OpenOptions::new().append(true).to_owned())?;

	let len = name.len() as u8;
	let buf = [len];
	keyfile.write_all(&buf)?;

	let len = key.as_bytes().len() as u32;
	let buf = len.to_be_bytes();
	keyfile.write_all(&buf)?;

	let buf = name.as_bytes();
	keyfile.write_all(buf)?;

	let buf = key.as_bytes();
	keyfile.write_all(buf)?;

	let b32 = base32::encode(key.as_bytes());
	println!("{b32}");
	Ok(())
}

fn get(name: String) -> Result<(), Box<dyn Error>> {
	if name.len() > 255 {
		return Err("Name length must be less than 255 bytes".into());
	}

	let name_bytes = name.as_bytes();
	let mut keyfile = open_keyfile(OpenOptions::new().read(true).to_owned())?;
	let key_bytes = loop {
		let mut buf = [0; 1];
		if let Err(_) = keyfile.read_exact(&mut buf) {
			return Err("Key does not exist".into());
		};

		let name_len = buf[0] as usize;

		let mut buf = [0; 4];
		keyfile.read_exact(&mut buf)?;
		let key_len = u32::from_be_bytes(buf);
		if name_len != name.len() {
			let total_len = name_len as i64 + key_len as i64;
			keyfile.seek(SeekFrom::Current(total_len))?;
			continue;
		}

		let mut buf = [0; 255];
		keyfile.read_exact(&mut buf[..name_len])?;
		if name_bytes != &buf[..name_len] {
			keyfile.seek(SeekFrom::Current(key_len as i64))?;
			continue;
		}

		let mut buf = vec![0; key_len as usize];
		keyfile.read_exact(&mut buf)?;
		break buf;
	};

	let key = Key::import(key_bytes);
	let code = key.code(SystemTime::now())?;
	println!("{code} ({}s)", code.ttl().as_secs());
	Ok(())
}

fn list() -> Result<(), Box<dyn Error>> {
	let mut file = open_keyfile(OpenOptions::new().read(true).to_owned())?;
	let now = SystemTime::now();
	loop {
		let mut buf = [0];
		if let Err(_) = file.read_exact(&mut buf) {
			break Ok(());
		}

		let name_len = buf[0] as usize;

		let mut buf = [0; 4];
		file.read_exact(&mut buf)?;
		let key_len = u32::from_be_bytes(buf) as usize;

		let mut buf = vec![0; name_len];
		file.read_exact(&mut buf)?;
		let name = String::from_utf8(buf)?;

		let mut buf = vec![0; key_len];
		file.read_exact(&mut buf)?;
		let key = Key::import(buf);
		let code = key.code(now)?;
		println!("{name}\t{code} ({}s)", code.ttl().as_secs());
	}
}

fn open_keyfile(opts: OpenOptions) -> Result<File, io::Error> {
	let mut path = dirs::config_dir().unwrap();
	path.push("keycodes.dat");
	if !path.exists() {
		File::create(&path)?;
	}

	opts.open(path)
}
