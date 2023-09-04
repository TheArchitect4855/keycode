use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Cli {
	#[command(subcommand)]
	pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
	#[clap(about = "Generate a keycode")]
	Generate {
		#[clap(help = "The name of the key to generate")]
		name: String,
	},
	#[clap(about = "Get a keycode")]
	Get {
		#[clap(help = "Name of the keycode to get")]
		name: String,
	},
	#[clap(about = "List keycodes")]
	List,
}
