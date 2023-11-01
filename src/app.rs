#![deny(unused_must_use)]
#![deny(missing_docs)]

use std::error::Error;
use std::fs::File;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use clap::{arg, Parser};
use crate::errors::*;

/// Struct to hold command-line arguments
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = include_str!("long-about.txt"))]
pub struct TwasArgs {
	/// Random look-up table files to include. Supported formats: .txt, .csv. .json. yaml, and .yml
	/// (or any of these with .gz or .zip compression)
	#[args[short='i', long="include"]]
	includes: Vec<PathBuf>,
	/// Optional seed for making the random number generator deterministic
	#[arg(short='s', long="seed")]
	seed: Option<u64>,
	/// Option to specify that output is written to the given filepath instead of being printed to
	/// the terminal
	#[args[short='o', long="output"]]
	output: Option<PathBuf>,
	/// Option to read target text for substitution from one or more files
	#[args[short='f', long="file"]]
	input: Vec<PathBuf>,
	/// Text to perform substitution on, eg "Meet my pet ${animal}". At least one text string must
	/// be provided unless you are using -f/--file or providing the target text via pipe
	/// (eg `$ cat my-story.txt | twas -i my-lookups.zip`)
	pub target_text: Vec<String>
}

/// Main entry point for the twas CLI app
pub fn main() -> ExitCode {
	let args = TwasArgs::parse();
	match run(args) {
		Ok(_) => ExitCode::SUCCESS,
		Err(e) => {
			eprintln!("Failed due to the following error:\n{}", e);
			ExitCode::FAILURE
		}
	}
}

/// Run the twas CLI app with the provided arguments
/// # Parameters
/// * **args: TwasArgs** - A `TwasArgs` struct holding all arguments for the `twas` app (typically
/// parsed from the CLI args)
/// # Returns
/// Returns `Ok(())` result on success, and `Box<dyn Error>` if an error occurs
pub fn run(args: TwasArgs) -> Result<(), Box<dyn Error>>{
	let mut gen = match args.seed{
		None => twas::Interpreter::new(),
		Some(seed) => twas::Interpreter::from_seed(seed)
	};
	for inc in args.includes {
		gen.load_file(inc)?
	}
	// sanity checks
	let stdin = std::io::stdin();
	// read targets
	let mut targets = args.target_text;
	for filepath in args.input {
		targets.push(std::fs::read_to_string(filepath)?);
	}
	if ! stdin.is_terminal() {
		targets.push(read_stdin(&stdin)?)
	}
	let fout: Option<File> =
		match args.output {
			None => None,
			Some(outfile) => {
				File::create(outfile)?;
			}
		};
	for target in targets {
		let result = gen.eval(target.as_str())?;
		println!("{}", result);
		println!();
		match &fout {
			Some(f) => {write!(f, "{}\n\n", result)},
			None => {}
		}
	}
	Ok(())
}

/// Util function to read stdin to a String
fn read_stdin(stdin: &std::io::Stdin) -> Result<String, std::io::Error> {
	let mut input =  Vec::new();
	let mut handle = stdin.lock();
	handle.read_to_end(&mut input)?;
	Ok(String::from(input))
}
