#![deny(unused_must_use)]
#![deny(missing_docs)]
#![doc = include_str!("../README.md")]
use dicexp::{DiceBag, new_simple_rng, simple_rng};
use rand::prelude::*;
use regex::Regex;
use serde_json;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, ErrorKind, Read};
use std::path::{Path, PathBuf};
use std::{fs, io};
use utf8_chars::BufReadCharsExt;
use zip;
use zip::result::ZipError;
mod data;
mod errors;
mod subspec;
use crate::data::{Item, LookUpTable};
use crate::errors::*;
use crate::subspec::SubstitutionOptions;

/// Marks the start of a substitution expression
const SUB_START: &str = "${";
/// Marks the start of a dice number expression
const DICE_START: &str = "#{";

/** The `Interpreter` struct is the text parsing engine for `twas`. It is initialized with a random
 number generator and then loaded with random lookup tables with the various `load_...()`
 methods. Text substitution processing is performed by the `eval(T)` method (`T` can be either
 `&str` or `String` or other string-like type)

# Examples
The following examples demonstrate both the `twas` text substitution syntax and typical useage of
the `twas` library.

## Simple text substitution
The following code initializes a new `Interpreter` and uses it to randomly choose a pet animal from the file `animal.txt`:
```rust
use twas;
let mut interpreter = twas::Interpreter::new();
interpreter.load_file("animal.txt").expect("Failed to load file");
let story = "I have a pet ${animal}.";
println!("{}", interpreter.eval(story).expect("Failed to eval"));
```

## Text substitution with references
The following code initializes a new `Interpreter` and uses it to randomly choose a pet animal from the file `animal.txt`, then re-use that same animal in the second sentence:
```rust
use twas;
let mut interpreter = twas::Interpreter::new();
interpreter.load_file("animal.txt").expect("Failed to load file");
let story = "I have a pet ${animal@pet}. I love my ${@pet}!";
println!("{}", interpreter.eval(story).expect("Failed to eval"));
```

## Text substitution with advanced options
Given the following `animal.txt` file:
```txt
aardvark
bird
cat
dog
elephant
```
and the following `pet-names.csv` file:
```txt
aardvark,bird,cat,dog,elephant
aaron,pip,paws,spot,nosey
aarnold,seed,mew,spike,jumbo
aaragon,lola,claws,rolf,tuba
```
this example generates a short story using a random animal and an appropriate name.
```rust
use twas;
let mut interpreter = twas::Interpreter::new();
interpreter.load_file("animal.txt").expect("Failed to load file");
interpreter.load_file("pet-names.csv").expect("Failed to load file");
let story = r#"I have a pet ${animal@pet}. His name is ${{id: "pet-names/$pet", case: title}}! ${{id: "@pet", aan: true, case: "first"}} is a girl's best friend."#;
println!("{}", interpreter.eval(story).expect("Failed to eval"));
```
 */
#[derive(Debug)]
pub struct Interpreter<R>
where
	R: RngExt,
{
	registry: HashMap<String, LookUpTable>,
	dice: DiceBag<StdRng>,
	rng: R,
	recursion_limit: usize,
}

impl<R> Interpreter<R>
where
	R: RngExt,
{
	/// Creates a new interpreter using the provided random number generator.
	/// # Arguments
	/// * rng: The random number generator to use.
	pub fn from_rng(mut rng: R) -> Interpreter<R> {
		let dice_seed: u64 = rng.random();
		Interpreter {
			registry: HashMap::new(),
			rng,
			dice: DiceBag::new(simple_rng(dice_seed)),
			recursion_limit: 1000,
		}
	}

	/// Sets the recursion limit to ensure that an infinite loop does not cause the program to
	/// run indefinitely (default is 1000)
	/// # Arguments
	/// * `limit`: The new recursion limit.
	pub fn set_recursion_limit(&mut self, limit: usize) {
		self.recursion_limit = limit;
	}

	/// Gets the recursion limit.
	pub fn get_recursion_limit(&mut self) -> usize {
		self.recursion_limit
	}

	/// Evaluates the given text to perform all text substitutions as per the `twas` text
	/// substitution syntax. See the [twas module](twas) description for more details on text
	/// substitution syntax.
	/// # Arguments
	/// * `text`: The target text to evaluate.
	/// # Returns
	/// The result of the evaluation, wrapped in a `Result`, or an error if the evaluation fails.
	///
	/// # Example
	/// ```rust
	/// use twas;
	/// let mut interpreter = twas::Interpreter::new();
	/// interpreter.load_file("animal.txt").expect("Failed to load file");
	/// let story = "My favorite animal is a ${animal}, and I have two pets, \
	/// ${{id: animal, aan: true}} and ${{id: animal, aan: true}}.";
	/// println!("{}", interpreter.eval(story).expect("Failed to eval"));
	/// ```
	pub fn eval<T>(&mut self, text: T) -> Result<String, ParsingError>
	where
		T: Into<String>,
	{
		do_eval(
			text.into(),
			0,
			&self.registry,
			&mut self.dice,
			&mut self.rng,
			self.recursion_limit,
			0,
		)
	}

	/// Loads a string containing a random look-up table in plain text (one line per item),
	/// comma-separated values (CSV), YAML, or JSON format. The parsed random look-up table is
	/// stored under the given look-up table ID. It is generally better to use the
	/// [load_file(...)](Interpreter::load_file) method instead of
	/// [load_str(...)](Interpreter::load_str).
	///
	/// See the [twas module](twas) description for more details on random look-up file formats.
	/// # Arguments
	/// * `id`: The identifier for the string.
	/// * `s`: The string to load.
	/// * `format`: The format of the string.
	/// # Returns
	/// A `Result` indicating success or failure.
	///
	/// # Example
	/// ```rust
	/// use twas;
	/// let mut interpreter = twas::Interpreter::new();
	/// interpreter.load_str("animal", include_str!("../animal.txt"), "txt").expect("failed to load animal.txt");
	/// let story = "I have a pet ${animal}.";
	/// println!("{}", interpreter.eval(story).expect("Failed to eval"));
	/// ```
	pub fn load_str<T>(&mut self, id: &str, s: T, format: &str) -> Result<(), errors::ParsingError>
	where
		T: Into<String>,
	{
		validate_id(id)?;
		let key = id;
		match format.to_lowercase().as_str() {
			"txt" => self.load_txt_str(key, s)?,
			"csv" => self.load_csv_str(key, s)?,
			"json" => self.load_json_str(key, s)?,
			"yml" => self.load_yaml_str(key, s)?,
			"yaml" => self.load_yaml_str(key, s)?,
			_ => {
				return Err(
					ParseError {
						msg: Some(format!(", format {} not supported", format)),
						line: None,
						col: None,
					}
					.into(),
				);
			},
		};
		Ok(())
	}

	/// Loads one (or more) random look-up table(s) from the given file. The base look-up table ID
	/// is just the filename without the file type suffix (eg "animal" for file "animal.txt"). In
	/// most cases, you should use [load_file(...)](Interpreter::load_file) with a directory or
	/// .zip file instead of this method.
	///
	/// Supported file formats:
	/// * .txt - each line is a look-up table item
	/// * .csv - each column is a look-up table, with optional `weight` column for specifying probability
	/// * .yaml|.yml - each list (unbiased table) and each map of string-number pairs (weighted table) is a look-up table
	/// * .json - each list (unbiased table) and each map of string-number pairs (weighted table) is a look-up table
	/// * directory - recursively load all supported files in directory
	/// * .zip - recursively load all supported files in the .zip archive
	///
	/// See the [twas module](twas) description for more details on random look-up file formats.
	///
	/// # Arguments
	/// * `filepath`: The path to the file to load.
	/// # Returns
	/// A `Result` indicating success or failure.
	///
	/// # Example
	/// `animal.txt`
	/// ```txt
	/// aardvark
	/// bird
	/// cat
	/// dog
	/// elephant
	/// ```
	/// `pet-names.csv`
	/// ```txt
	/// aardvark,bird,cat,dog,elephant
	/// aaron,pip,paws,spot,nosey
	/// aarnold,seed,mew,spike,jumbo
	/// aaragon,lola,claws,rolf,tuba
	/// ```
	/// Program code:
	/// ```rust
	/// use twas;
	/// let mut interpreter = twas::Interpreter::new();
	/// interpreter.load_file("animal.txt").expect("Failed to load file");
	/// interpreter.load_file("pet-names.csv").expect("Failed to load file");
	/// println!("Loaded IDs: {:?}", interpreter.list_ids());
	/// // prints "Loaded IDs: [animal, pet-names/aardvark, pet-names/bird, pet-names/cat,
	/// //     pet-names/dog, pet-names/elephant]"
	/// let story = r#"I have a pet ${animal@pet}. His name is ${{id: "pet-names/$pet", case: title}}! \
	/// ${{id: "@pet", aan: true, case: "first"}} is a girl's best friend."#;
	/// println!("{}", interpreter.eval(story).expect("Failed to eval"));
	/// // prints: "I have a pet elephant. His name is Tuba! An elephant is a girl's best friend."
	/// ```
	pub fn load_file<P>(&mut self, filepath: P) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		self.load_file_namespaced(filepath, "")
	}
	/// Loads one (or more) random look-up table(s) from the given file (just like
	/// [load_file(...)](Interpreter::load_file), but with the given "namespace" prefix added to
	/// the front of the look-up table ID(s). The base look-up table ID will be
	/// `namespace/filename` where `filename` is the file's base name without the file type
	/// suffix (eg "animal" for file "animal.txt"). You should prefer to use
	///
	/// Supported file formats:
	/// * .txt - each line is a look-up table item
	/// * .csv - each column is a look-up table, with optional `weight` column for specifying
	/// probability
	/// * .yaml - each list (unbiased table) and each map of string-number pairs (weighted table)
	/// is a look-up table
	///
	/// See the [twas module](twas) description for more details on random look-up file formats.
	///
	/// # Arguments
	/// * `filepath`: The path to the file to load.
	/// * `id_prefix`: ID prefix
	/// # Returns
	/// A `Result` indicating success or failure.
	///
	/// # Example
	/// `animal.txt`
	/// ```txt
	/// aardvark
	/// bird
	/// cat
	/// dog
	/// elephant
	/// ```
	/// `pet-names.csv`
	/// ```txt
	/// aardvark,bird,cat,dog,elephant
	/// aaron,pip,paws,spot,nosey
	/// aarnold,seed,mew,spike,jumbo
	/// aaragon,lola,claws,rolf,tuba
	/// ```
	/// Program code:
	/// ```rust
	/// use twas;
	/// let mut interpreter = twas::Interpreter::new();
	/// interpreter.load_file_namespaced("animal.txt", "my-story").unwrap();
	/// interpreter.load_file_namespaced("pet-names.csv", "my-story").unwrap();
	/// println!("Loaded IDs: {:?}", interpreter.list_ids());
	/// // prints "Loaded IDs: [my-story/animal, my-story/pet-names/aardvark,
	/// //     my-story/pet-names/bird, my-story/pet-names/cat,
	/// //     my-story/pet-names/dog, my-story/pet-names/elephant]"
	/// let story = r#"I have a pet ${my-story/animal@pet}. \
	/// His name is ${{id: "my-story/pet-names/$pet", case: title}}!"#;
	/// println!("{}", interpreter.eval(story).expect("Failed to eval"));
	/// // prints: "I have a pet dog. His name is Spot!"
	/// ```
	pub fn load_file_namespaced<P>(
		&mut self,
		filepath: P,
		id_prefix: &str,
	) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		validate_id(id_prefix)?;
		let id_prefix = id_prefix.trim();
		let filepath: PathBuf = filepath.into();
		if !filepath.exists() {
			return Err(io::Error::from(ErrorKind::NotFound).into());
		}
		if filepath.is_dir() {
			return self.load_dir_namespaced(filepath, id_prefix);
		}
		let path = filepath.as_path();
		let file_type = path
			.extension()
			.ok_or_else(|| ParseError {
				msg: Some(format!(
					"{:?} has unknown file type, please append .txt or other extension to it",
					filepath
				)),
				line: None,
				col: None,
			})?
			.to_str()
			.ok_or_else(|| {
				io::Error::new(
					ErrorKind::Unsupported,
					"Invalid characters in file extension",
				)
			})?;
		let filename = path
			.file_name()
			.ok_or_else(|| ParseError {
				msg: Some("Cannot get name of file".into()),
				line: None,
				col: None,
			})?
			.to_str()
			.ok_or_else(|| io::Error::new(ErrorKind::Unsupported, "Invalid characters in file name"))?;
		let mut id: String = id_prefix.into();
		if !id.is_empty() {
			id.push_str("/");
		}
		id.push_str(&filename[0..filename.rfind(".").unwrap_or(filename.len())]);
		match file_type.to_lowercase().as_str() {
			"txt" => {
				let input_file = File::open(path)?;
				let reader = io::BufReader::new(input_file);
				for line in reader.lines() {
					let entry = line?;
					self.get_or_create_lut(&id).add_item(entry, 1f64);
				}
			},
			"csv" => {
				let input_file = File::open(path)?;
				let reader = io::BufReader::new(input_file);
				self.load_csv(id.as_str(), reader)?;
			},
			"json" => {
				let input_file = File::open(path)?;
				let reader = io::BufReader::new(input_file);
				self.load_json(id.as_str(), reader)?;
			},
			"yml" | "yaml" => {
				let input_file = File::open(path)?;
				let reader = io::BufReader::new(input_file);
				self.load_yaml(id.as_str(), reader)?;
			},
			"zip" => return self.load_zip_namespaced(filepath, id_prefix),
			_ => {
				return Err(
					ParseError {
						msg: Some(format!("file type '{}' not supported", file_type)),
						line: None,
						col: None,
					}
					.into(),
				);
			},
		}
		Ok(())
	}

	/// Parses a YAML map object (recursive). If the map contains key:value pairs where the value
	/// is a number, then it is parsed as a weighted look-up table. If the map contains nested
	/// maps or lists, then it is recursively parsed.
	fn load_yaml_mapping(
		&mut self,
		map: serde_yaml_neo::mapping::Mapping,
		id_prefix: &str,
	) -> Result<(), ParsingError> {
		let id = String::from(id_prefix);
		for (k, v) in map {
			match k {
				serde_yaml_neo::Value::String(text) => match v {
					serde_yaml_neo::Value::Number(weight) => {
						let weight: f64 = weight.as_f64().ok_or_else(|| ParseError {
							msg: Some(format!("Could not convert {:?} to float", weight)),
							line: None,
							col: None,
						})?;
						self.get_or_create_lut(&id).add_item(text, weight);
					},
					serde_yaml_neo::Value::Mapping(nested_map) => {
						// sub-table
						let mut next_id = id.clone();
						if !id_prefix.is_empty() {
							next_id.push_str("/");
						}
						next_id.push_str(text.as_str());
						self.load_yaml_mapping(nested_map, next_id.as_str())?;
					},
					serde_yaml_neo::Value::Sequence(list) => {
						let mut next_id = id.clone();
						if !id_prefix.is_empty() {
							next_id.push_str("/");
						}
						next_id.push_str(text.as_str());
						self.load_yaml_sequence(list, next_id.as_str())?;
					},
					_ => {
						return Err(
							ParseError {
								msg: Some(format!(
									"Weight must be a number, but weight for '{}' was '{:?}' instead",
									text, v
								)),
								line: None,
								col: None,
							}
							.into(),
						);
					},
				},
				_ => {
					return Err(
						ParseError {
							msg: Some(format!("Invalid key format, key must be a string")),
							line: None,
							col: None,
						}
						.into(),
					);
				},
			}
		}
		Ok(())
	}

	/// Parses a YAML list object as an unbiased look-up table
	fn load_yaml_sequence(
		&mut self,
		list: serde_yaml_neo::Sequence,
		id_prefix: &str,
	) -> Result<(), ParsingError> {
		let id = String::from(id_prefix);
		for entry in list {
			match entry {
				// list of strings
				serde_yaml_neo::Value::String(text) => self.get_or_create_lut(&id).add_item(text, 1f64),
				_ => {
					return Err(
						ParseError {
							msg: Some(format!(
								"Only lists of strings are supported, found {:?}",
								entry
							)),
							line: None,
							col: None,
						}
						.into(),
					);
				},
			}
		}
		Ok(())
	}

	/// Recursively scans the provided directory for random look-up table(s) from all supported
	/// file formats found within the directory. The base look-up table ID for each table is the
	/// relative filepath of the look-up table files (eg "bar/animal" for file
	/// "foo/bar/animal.txt"). In most cases, you should use
	/// [load_file(...)](Interpreter::load_file) instead of this method.
	///
	/// See the [twas module](twas) description for more details on supported random look-up
	/// file formats.
	///
	/// # Arguments
	/// * `dirpath`: The path to the directory to load.
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_dir<P>(&mut self, dirpath: P) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		self.load_dir_namespaced(dirpath, "")
	}

	/// Recursively scans the provided directory for random look-up table(s) from all supported
	/// file formats found within the directory. The base look-up table ID for each table is the
	/// relative filepath of the look-up table files (eg "bar/animal" for file
	/// "foo/bar/animal.txt"). In most cases, you should use
	/// [load_file(...)](Interpreter::load_file) instead of this method.
	///
	/// See the [twas module](twas) description for more details on supported random look-up
	/// file formats.
	///
	/// # Arguments
	/// * `dirpath`: The path to the directory to load.
	/// * `id_prefix`: ID prefix path, use an empty String ("") if this directory is the root of
	/// the directory tree
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_dir_namespaced<P>(&mut self, dirpath: P, id_prefix: &str) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		validate_id(id_prefix)?;
		for file in fs::read_dir(dirpath.into())? {
			let file_path = file?.path();
			match file_path.is_dir() {
				true => {
					let dir_name = file_path
						.file_name()
						.ok_or_else(|| ParseError {
							msg: Some("Cannot get name of directory".into()),
							line: None,
							col: None,
						})?
						.to_str()
						.ok_or_else(|| {
							io::Error::new(
								ErrorKind::Unsupported,
								"Invalid characters in directory file name",
							)
						})?;
					let mut new_id: String = id_prefix.into();
					new_id.push_str(dir_name);
					self.load_dir_namespaced(&file_path, new_id.as_str())?;
				},
				false => {
					match file_path.extension() {
						None => {}, // ignore
						Some(suffix) => {
							let suffix = suffix.to_str().ok_or_else(|| {
								io::Error::new(
									ErrorKind::Unsupported,
									"Invalid characters in file extension",
								)
							})?;
							match suffix.to_lowercase().as_str() {
								"txt" | "csv" | "yml" | "yaml" | "json" => {
									self.load_file_namespaced(file_path.as_path(), id_prefix)?
								},
								_ => {}, // ignore
							}
						},
					}
				},
			}
		}
		Ok(())
	}

	/// Reads the provided zip archive for random look-up table(s) from all supported
	/// file formats found within the archive. The base look-up table ID for each table is the
	/// relative filepath of the look-up table files  within the zip archive (eg "bar/animal" for
	/// file "bar/animal.txt" in "foo.zip"). In most cases, you should use
	/// [load_file(...)](Interpreter::load_file) instead of this method.
	///
	/// See the [twas module](twas) description for more details on supported random look-up
	/// file formats.
	///
	/// # Arguments
	/// * `zippath`: The path to the zip file to load.
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_zip<P>(&mut self, zippath: P) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		self.load_zip_namespaced(zippath, "")
	}

	/// Reads the provided zip archive for random look-up table(s) from all supported
	/// file formats found within the archive. The base look-up table ID for each table is the
	/// relative filepath of the look-up table files  within the zip archive (eg "bar/animal" for
	/// file "bar/animal.txt" in "foo.zip"). In most cases, you should use
	/// [load_file(...)](Interpreter::load_file) instead of this method.
	///
	/// See the [twas module](twas) description for more details on supported random look-up
	/// file formats.
	///
	/// # Arguments
	/// * `zippath`: The path to the zip file to load.
	/// * `id_prefix`: ID prefix path, use an empty String ("") if not adding a prefix
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_zip_namespaced<P>(&mut self, zippath: P, id_prefix: &str) -> Result<(), ParsingError>
	where
		P: Into<PathBuf>,
	{
		// extract files and then parse the directory
		let tmp_dir = tempfile::tempdir()?;
		unzip_file(zippath.into().as_path(), tmp_dir.path())?;
		self.load_dir_namespaced(tmp_dir.path(), id_prefix)
	}

	/// Parses the provided string as a .txt file. Each line will be parsed as an entry in a
	/// look-up table, with all possible values having equal weight.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: look-up table ID to register this look-up table for text substitution
	/// * `txt`: the text to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_txt_str<T>(&mut self, id: &str, txt: T) -> Result<(), ParsingError>
	where
		T: Into<String>,
	{
		validate_id(id)?;
		let id = String::from(id);
		if !self.registry.contains_key(&id) {
			self.registry.insert(id.clone(), LookUpTable::new());
		}
		let lut = self.registry.get_mut(&id).unwrap();
		let txt: String = txt.into();
		for line in txt.split("\n") {
			lut.add_item(line.trim(), 1.);
		}
		Ok(())
	}

	/// Parses the provided string as a .csv file. The text will be interpreted as standard
	/// comma-separate value (CSV) file, where the first row is the header row containing column
	/// names and all subsequent rows are the possible values for each column. Each column is its
	/// own random look-up table. All rows have equal probability, unless there is a column
	/// named `weight`. If a `weight` column is present, then the probability of each row is
	/// weighted by the decimal value in the corresponding `weight` column.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: each column in the CSV text will be registered as a look-up table with ID `id/column-name`
	/// * `txt`: the text to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_csv_str<T>(&mut self, id: &str, txt: T) -> Result<(), ParsingError>
	where
		T: Into<String>,
	{
		let txt: String = txt.into();
		let reader = BufReader::new(txt.as_bytes());
		self.load_csv(id, reader)
	}

	/// Parses the provided string as JSON. A JSON object can contain one or multiple random
	/// look-up tables, with arbitrary levels of nested depth. Any lists encountered in the JSON
	/// will be parsed as look-up tables with equal probability for all items, while
	/// weighted-probabilities are specified using a string-number mapping
	/// (eg `rarity: {"common": 6, "uncommon": 3, "rare": 0.9, "very rare": 0.1}`). The tables can be
	/// organized by nesting map objects, with each nesting adding a level to the look-up table
	/// ID path.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: this id will be prefixed to the look-up tables nested in the provided JSON string
	/// * `txt`: the text to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_json_str<T>(&mut self, id: &str, txt: T) -> Result<(), ParsingError>
	where
		T: Into<String>,
	{
		let txt: String = txt.into();
		let reader = BufReader::new(txt.as_bytes());
		self.load_json(id, reader)
	}

	/// Parses the provided string as YAML. A YAML object can contain one or multiple random
	/// look-up tables, with arbitrary levels of nested depth. Any lists encountered in the YAML
	/// content will be parsed as look-up tables with equal probability for all items, while
	/// weighted-probabilities are specified using a string-number mapping
	/// (eg `rarity: {common: 6, uncommon: 3, rare: 0.9, "very rare": 0.1}`). The tables can be
	/// organized by nesting map objects, with each nesting adding a level to the look-up table
	/// ID path.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: this id will be prefixed to the look-up tables nested in the provided YAML string
	/// * `txt`: the text to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_yaml_str<T>(&mut self, id: &str, txt: T) -> Result<(), ParsingError>
	where
		T: Into<String>,
	{
		let txt: String = txt.into();
		let reader = BufReader::new(txt.as_bytes());
		self.load_yaml(id, reader)
	}

	/// Parses the provided stream as a .txt file. Each line will be parsed as an entry in a
	/// look-up table, with all possible values having equal weight.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: look-up table ID to register this look-up table for text substitution
	/// * `reader`: the text stream to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_txt<I: Read>(&mut self, id: &str, mut reader: I) -> Result<(), ParsingError> {
		let mut content = String::new();
		reader.read_to_string(&mut content)?;
		self.load_txt_str(id, content)?;
		Ok(())
	}

	/// Parses the provided stream as a .csv file. The text will be interpreted as standard
	/// comma-separate value (CSV) file, where the first row is the header row containing column
	/// names and all subsequent rows are the possible values for each column. Each column is its
	/// own random look-up table. All rows have equal probability, unless there is a column
	/// named `weight`. If a `weight` column is present, then the probability of each row is
	/// weighted by the decimal value in the corresponding `weight` column.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: each column in the CSV text will be registered as a look-up table with ID `id/column-name`
	/// * `reader`: the text stream to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_csv<I: Read>(&mut self, id_prefix: &str, reader: I) -> Result<(), ParsingError> {
		validate_id(id_prefix)?;
		let mut buffered_reader = BufReader::new(reader);
		let mut char_iter = buffered_reader.chars();
		let cols = match read_csv_row(&mut char_iter) {
			Some(row) => row,
			None => return Err(ParsingError::from(NoValuesError {})),
		};
		let mut weights_col: Option<usize> = None;
		for i in 0..cols.len() {
			let col = &cols[i];
			if col.as_str() == "weight" {
				weights_col = Some(i);
			}
		}
		while match read_csv_row(&mut char_iter) {
			None => false,
			Some(row) => {
				let w = match weights_col {
					None => 1f64,
					Some(c) => row[c].parse::<f64>()?,
				};
				for i in 0..row.len() {
					let col: &String = &cols[i];
					let cell: &String = &row[i];
					if cell.is_empty() {
						// empty cell, assume uneven table and do nothing
					} else {
						let mut id: String = id_prefix.into();
						if !id_prefix.is_empty() {
							id.push_str("/");
						}
						id.push_str(col.as_str());
						self.get_or_create_lut(&id).add_item(cell.clone(), w);
					}
				}
				true
			},
		} {}
		Ok(())
	}

	/// Parses the provided stream as JSON. A JSON object can contain one or multiple random
	/// look-up tables, with arbitrary levels of nested depth. Any lists encountered in the JSON
	/// will be parsed as look-up tables with equal probability for all items, while
	/// weighted-probabilities are specified using a string-number mapping
	/// (eg `rarity: {"common": 6, "uncommon": 3, "rare": 0.9, "very rare": 0.1}`). The tables can be
	/// organized by nesting map objects, with each nesting adding a level to the look-up table
	/// ID path.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: this id will be prefixed to the look-up tables nested in the provided JSON string
	/// * `reader`: the text stream to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_json<I: Read>(&mut self, id: &str, reader: I) -> Result<(), ParsingError> {
		self.load_yaml(id, reader)
	}

	/// Parses the provided stream as YAML. A YAML object can contain one or multiple random
	/// look-up tables, with arbitrary levels of nested depth. Any lists encountered in the YAML
	/// stream will be parsed as look-up tables with equal probability for all items, while
	/// weighted-probabilities are specified using a string-number mapping
	/// (eg `rarity: {common: 6, uncommon: 3, rare: 0.9, "very rare": 0.1}`). The tables can be
	/// organized by nesting map objects, with each nesting adding a level to the look-up table
	/// ID path.
	///
	/// See the [twas module](twas) description for more details on random look-up formats.
	///
	/// # Arguments
	/// * `id`: this id will be prefixed to the look-up tables nested in the provided YAML string
	/// * `reader`: the text stream to parse
	/// # Returns
	/// A `Result` indicating success or failure.
	pub fn load_yaml<I: Read>(&mut self, id: &str, reader: I) -> Result<(), ParsingError> {
		let parsed: serde_yaml_neo::Value = serde_yaml_neo::from_reader(reader)?;
		match parsed {
			serde_yaml_neo::Value::Sequence(list) => {
				self.load_yaml_sequence(list, id)?;
			},
			serde_yaml_neo::Value::Mapping(map) => {
				// map of items and weights or map of maps of items
				self.load_yaml_mapping(map, id)?
			},
			_ => {
				return Err(
					ParseError {
						msg: Some(format!(
							"Failed to parse {}, wrong structure (should be list or mapping object)",
							id
						)),
						line: None,
						col: None,
					}
					.into(),
				);
			},
		}
		Ok(())
	}

	/// Gets a random look-up table from the registry by it's registered ID
	/// (eg `animal` for `animal.txt`). If there is no look-up table for that ID, then a new empty
	/// look-up table will be created
	/// #Arguments
	/// * `id`: look-up table ID to fetch
	/// # Returns
	/// A [LookUpTable](twas::data::LookUpTable) object representing a random look-up table
	pub fn get_or_create_lut(&mut self, id: &str) -> &mut LookUpTable {
		let id = String::from(id);
		if self.registry.contains_key(&id) {
			return self.registry.get_mut(&id).unwrap();
		} else {
			let lut = LookUpTable::new();
			self.registry.insert(id.clone(), lut);
			return self.registry.get_mut(&id).unwrap();
		}
	}

	/// Gets a random look-up table from the registry by it's registered ID, if one exists.
	/// #Arguments
	/// * `id`: look-up table ID to fetch
	/// # Returns
	/// An `Option` containing a [LookUpTable](twas::data::LookUpTable) object representing a
	/// random look-up table, or None if no look-up table has been registered with the requested ID
	pub fn get_lut(&mut self, id: &str) -> Option<&mut LookUpTable> {
		let id = String::from(id);
		self.registry.get_mut(&id)
	}

	/// Gets a list of all currently registered look-up tables
	pub fn list_ids(&self) -> Vec<&String> {
		self.registry.keys().collect::<Vec<&String>>()
	}
}

impl Interpreter<rand::rngs::StdRng> {
	/// Creates a new interpreter
	pub fn new() -> Interpreter<rand::rngs::StdRng> {
		Interpreter::from_rng(new_simple_rng())
	}

	/// Creates a new interpreter with a seeded random number generator, such that identical usage
	/// with the same seed will result in identical results
	pub fn from_seed(seed: u64) -> Interpreter<rand::rngs::StdRng> {
		Interpreter::from_rng(simple_rng(seed))
	}
}

/// This is where all the action happens when evaluating a string for text substitution
fn do_eval<R: RngExt>(
	text: String,
	start_from: usize,
	reg: &HashMap<String, LookUpTable>,
	dice: &mut DiceBag<R>,
	rng: &mut impl RngExt,
	recursion_limit: usize,
	recursion: usize,
) -> Result<String, ParsingError> {
	if recursion > recursion_limit {
		return Err(RecursionLimitReached { limit: recursion_limit }.into());
	}
	//eprintln!("'{}'", text);
	let mut ref_map: HashMap<String, String> = HashMap::new();
	let mut text = text;
	let mut new_text;
	let mut pos = start_from;
	loop {
		match next_token(&text, pos, SUB_START) {
			None => break,
			Some((start, end)) => {
				let (front, tmp) = text.split_at(start);
				let (token, back) = tmp.split_at(end - start);
				let token = &token[SUB_START.len()..token.len() - 1];
				let substitution = do_sub(
					token.trim(),
					reg,
					dice,
					&mut ref_map,
					rng,
					recursion_limit,
					recursion,
				)?;
				//eprintln!("\tToken substitution: {} -> {}", token, substitution);
				new_text = String::from(front);
				new_text.push_str(substitution.as_str());
				new_text.push_str(back);
				pos = start;
			},
		}
		text = new_text;
	}
	loop {
		match next_token(&text, pos, DICE_START) {
			None => break,
			Some((start, end)) => {
				let (front, tmp) = text.split_at(start);
				let (token, back) = tmp.split_at(end - start);
				let dice_exp = &token[DICE_START.len()..token.len() - 1];
				let substitution = do_dice(dice_exp.trim(), dice)?;
				//eprintln!("\tDice substitution: {} -> {}", dice_exp, substitution);
				new_text = String::from(front);
				new_text.push_str(substitution.as_str());
				new_text.push_str(back);
				pos = start;
			},
		}
		text = new_text;
	}
	return Ok(text);
}

/// Generate a substitution from the provided substitution token, such as `${animal}` (note that the
/// `${` and `}` have already been stripped away).
fn do_sub<R: RngExt>(
	token: &str,
	reg: &HashMap<String, LookUpTable>,
	dice: &mut DiceBag<R>,
	ref_map: &mut HashMap<String, String>,
	rng: &mut impl RngExt,
	recursion_limit: usize,
	recursion: usize,
) -> Result<String, ParsingError> {
	// parse the token
	//eprintln!("Token: '{}'", token);
	let mut sub: SubstitutionOptions;
	// try YAML parsing in case user forgot to use double braces {{ }}
	if token.starts_with("{") && token.ends_with("}") {
		// JSON string with advanced options
		sub = serde_yaml_neo::from_str(token)?;
	} else {
		// simple token (but might have ref suffix)
		let token = token.trim();
		if token.starts_with("id:") || token.starts_with(r#""id":"#) {
			// looks like they forgot to use {{ double braces }} for JSON/YAML
			//eprintln!("WARNING: Substitution token '${{ {} }}' looks like JSON/YAML, but was not enclosed in double-braces. Treating it as JSON/YAML.", token);
			sub = serde_yaml_neo::from_str(format!("{{{}}}", token).as_str())?;
		} else {
			if token.starts_with("@") {
				// simple ref lookup: @ref
				sub = SubstitutionOptions::new(token);
			} else if token.contains("@") {
				// simple ref save: id@ref
				let i = token.find("@").unwrap();
				let (id, ref_token) = token.split_at(i);
				sub = SubstitutionOptions::new_with_ref(id, &ref_token[1..]);
			} else {
				// simple id lookup
				sub = SubstitutionOptions::new(token);
			}
		}
	}
	// apply references to id
	if sub.id.contains("$") {
		//eprintln!("ref_map: {:?}", ref_map);
		//eprintln!("sub.id: {}", sub.id);
		sub.id = do_ref_sub_in_id(sub.id.as_str(), ref_map)?;
		//eprintln!("new sub.id: {}", sub.id);
	}
	// generate substitution or recall a reference
	let mut text;
	if sub.id.starts_with("@") {
		// is a reference, return previously generated item
		let ref_id = String::from(&sub.id[1..]);
		match ref_map.get(&ref_id) {
			None => return Err(KeyNotFoundError { key: ref_id }.into()),
			Some(stored) => text = stored.clone(),
		}
		// prefix a/an if requested
		text = match &sub.aan {
			None => text,
			Some(aan) => {
				if *aan {
					// add a or an as appropriate
					let mut buffer = String::from(indefinite_article_prefix_for(text.as_str()));
					buffer.push_str(text.as_str());
					buffer
				} else {
					text
				}
			},
		};
		// change case if requested
		text = match &sub.case {
			None => text,
			Some(ch_case) => match ch_case.as_str() {
				"original" => text,
				"upper" => text.to_uppercase(),
				"lower" => text.to_lowercase(),
				"title" => title_case(text),
				"first" => {
					let s = text.as_str();
					let mut buffer = String::new();
					buffer.push_str(&s[0..1].to_uppercase().as_str());
					buffer.push_str(&s[1..]);
					buffer
				},
				_ => {
					return Err(ParsingError::ParseError(ParseError {
						msg: Some(ch_case.clone()),
						line: None,
						col: None,
					}));
				},
			},
		}
	} else {
		// draw the items
		let items: Vec<Item>;
		let lut = reg
			.get(sub.id.as_str())
			.ok_or_else(|| KeyNotFoundError { key: sub.id.into() })?;
		let num_to_draw: usize;
		match sub.count {
			None => num_to_draw = 1,
			Some(count_val) => match count_val {
				serde_yaml_neo::Value::Number(n) => {
					num_to_draw = n.as_u64().ok_or_else(|| ParseError {
						msg: Some(format!("{} as unsigned integer", n)),
						line: None,
						col: None,
					})? as usize
				},
				serde_yaml_neo::Value::String(dice_ex) => {
					let mut dice = DiceBag::new(simple_rng(rng.random()));
					let roll = dice.eval_total(dice_ex.as_str()).map_err(|_| ParseError {
						msg: Some(format!("'{}' is not a valid dice expression", dice_ex)),
						line: None,
						col: None,
					})?;
					if roll < 0 {
						num_to_draw = 0;
					} else {
						num_to_draw = roll as usize;
					}
				},
				_ => {
					return Err(ParsingError::ParseError(ParseError {
						msg: Some(String::from(token)),
						line: None,
						col: None,
					}));
				},
			},
		}
		match sub.method {
			None => items = lut.draw_n_random(rng, num_to_draw)?,
			Some(method) => match method.as_str() {
				"random" => items = lut.draw_n_random(rng, num_to_draw)?,
				"shuffle" => items = lut.shuffle_draw(rng, num_to_draw)?,
				_ => {
					return Err(ParsingError::ParseError(ParseError {
						msg: Some(method.clone()),
						line: None,
						col: None,
					}));
				},
			},
		}
		// format to text
		text = String::new();
		let mut loop_count = 0;
		let loop_total = items.len();
		for item in items {
			if loop_count > 0 {
				match &sub.sep {
					None => {},
					Some(sep) => {
						if loop_count == loop_total - 1 && (&sub.last_sep).is_some() {
							text.push_str(&unescape(sub.last_sep.clone().unwrap().as_str())?.as_str())
						} else {
							text.push_str(unescape(sep)?.as_str())
						}
					},
				}
			}
			match &sub.prefix {
				None => {},
				Some(prefix) => text.push_str(prefix.as_str()),
			}
			// do substitutions in randomly drawn text (if any)
			text = do_eval(text, 0, reg, dice, rng, recursion_limit, recursion + 1)?;
			// prefix a/an if requested
			let item_text: String = match &sub.aan {
				None => item.get_text().clone(),
				Some(aan) => {
					if *aan {
						// add a or an as appropriate
						let mut buffer = String::from(indefinite_article_prefix_for(item.get_text().as_str()));
						buffer.push_str(item.get_text().as_str());
						buffer
					} else {
						item.get_text().clone()
					}
				},
			};
			// change case if requested
			match &sub.case {
				None => text.push_str(item_text.as_str()),
				Some(ch_case) => match ch_case.as_str() {
					"original" => text.push_str(item_text.as_str()),
					"upper" => text.push_str(item_text.to_uppercase().as_str()),
					"lower" => text.push_str(item_text.to_lowercase().as_str()),
					"title" => text.push_str(title_case(item_text).as_str()),
					"first" => {
						let s = item_text.as_str();
						text.push_str(&s[0..1].to_uppercase().as_str());
						text.push_str(&s[1..]);
					},
					_ => {
						return Err(ParsingError::ParseError(ParseError {
							msg: Some(ch_case.clone()),
							line: None,
							col: None,
						}));
					},
				},
			}
			match &sub.suffix {
				None => {},
				Some(suffix) => text.push_str(suffix.as_str()),
			}
			loop_count += 1;
		}
	}
	// store items as ref if requested
	match &sub.reference {
		None => {},
		Some(ref_id) => {
			// eval the string to store in case it contains nested references
			text = do_eval(text, 0, reg, dice, rng, recursion_limit, recursion + 1)?;
			//eprintln!("Storing reference '{}' -> '{}'", ref_id, text);
			validate_ref(ref_id)?;
			let _ = ref_map.insert(ref_id.clone(), text.clone());
		},
	}
	// hide text if requested
	match sub.hidden {
		None => {},
		Some(hide_me) => {
			if hide_me {
				text = String::from("")
			}
		},
	}
	Ok(text)
}

/// When using `$` reference substitution in an ID string, this function is called to handle it.
/// Replaces `$ref-id` with the previously generated value that was saved under that ref ID
fn do_ref_sub_in_id(id: &str, ref_map: &HashMap<String, String>) -> Result<String, ParsingError> {
	let mut new_id = String::from(id);
	let mut tmp_id = String::from(id);
	let finder: Regex = Regex::new(r#"\$[\d\pL_\-+]+"#).unwrap();
	//eprintln!("do_ref_sub_in_id({}, {:?})", id, ref_map);
	loop {
		//eprintln!("loop...");
		match finder.find(new_id.as_str()) {
			None => break,
			Some(matched) => {
				let ref_id = String::from(&matched.as_str()[1..]); // srtip-off $ prefix
				//eprintln!("ref_id: {}", ref_id);
				match ref_map.get(&ref_id) {
					None => return Err(KeyNotFoundError { key: ref_id }.into()),
					Some(ref_value) => {
						let (front, _) = new_id.split_at(matched.start());
						let (_, back) = new_id.split_at(matched.end());
						tmp_id.clear();
						tmp_id.push_str(front);
						tmp_id.push_str(ref_value.as_str());
						tmp_id.push_str(back);
					},
				}
			},
		}
		new_id = tmp_id.clone();
	}
	Ok(new_id)
}

/// Returns an error result if the ID string is not valid, otherwise OK
fn validate_id<T>(id: T) -> Result<(), ParsingError>
where
	T: Into<String>,
{
	let id = id.into();
	let id_str = id.as_str();
	if id_str.contains("@") || id_str.contains("$") {
		return Err(
			InvalidIDError::new(format!(
				"'{}' is not a valid ID. IDs cannot contain '@' or '$'",
				id_str
			))
			.into(),
		);
	}
	Ok(())
}

/// Returns an error result if the ref ID string is not valid, otherwise OK
fn validate_ref<T>(id: T) -> Result<(), ParsingError>
where
	T: Into<String>,
{
	let checker: Regex = Regex::new(r#"^[\d\pL_\-+]+$"#).unwrap();
	let id = id.into();
	let id_str = id.as_str();
	if !checker.is_match(id_str) {
		return Err(InvalidIDError::new(format!("'{}' is not a valid reference ID. Reference IDs can only contain letters, numbers, _, -, and/or +", id_str)).into());
	}
	Ok(())
}

/// Return "a " or "an " depending on the first letter (or number) of the provided string
fn indefinite_article_prefix_for(text: &str) -> &'static str {
	let text = text.trim();
	if text.starts_with("a")
		|| text.starts_with("e")
		|| text.starts_with("i")
		|| text.starts_with("o")
		|| text.starts_with("u")
		|| text.starts_with("8")
	{
		"an "
	} else {
		"a "
	}
}

/// Handle `#{...}` number generation (eg "2d6+3")
fn do_dice<R>(dice_exp: &str, dice: &mut DiceBag<R>) -> Result<String, ParsingError>
where
	R: RngExt,
{
	let roll =
		dice
			.eval_total(dice_exp)
			.map_err(|e| ParseError { msg: e.msg, line: None, col: None })?;
	Ok(format!("{}", roll))
}

/// Converts a string to title case. This function is a little smarter than the standard
/// [String::to_title_case()](std::String::to_title_case) method, as it does not capitalize articles
/// and some prepositions
fn title_case(text: String) -> String {
	let mut output = String::new();
	let mut last_char: char = ' ';
	for (i, c) in text.char_indices() {
		if i == 0 {
			output.push_str(c.to_uppercase().to_string().as_str());
		} else if last_char.is_whitespace() {
			let (_, remainder) = text.split_at(i);
			let remainder = remainder.to_lowercase();
			if remainder.starts_with("the ")
				|| remainder.starts_with("of ")
				|| remainder.starts_with("a ")
				|| remainder.starts_with("an ")
				|| remainder.starts_with("and ")
				|| remainder.starts_with("in ")
				|| remainder.starts_with("on ")
			{
				output.push_str(c.to_lowercase().to_string().as_str());
			} else {
				output.push_str(c.to_uppercase().to_string().as_str());
			}
		} else {
			output.push_str(c.to_lowercase().to_string().as_str());
		}
		last_char = c;
	}
	return output;
}

/// Interprets JSON-style escapes such as `\n` as the intended characters
fn unescape<T>(s: T) -> Result<String, serde_json::Error>
where
	T: Into<String>,
{
	let txt = format!("\"{}\"", s.into());
	serde_json::from_str(txt.as_str())
}

/// FSM for in-house token parser (used to match {} curly braces in order to accurately extract
/// embedded YAML/JSON substitution tokens
#[derive(Debug)]
enum TokenParserFSM {
	Normal,
	EscapeInQuote,
	Escape,
	Quote,
}

/// Find next substituion token, if it exists, returning the start and end byte indices in the
/// provided UTF8 string
fn next_token(text: &String, pos: usize, token_start: &str) -> Option<(usize, usize)> {
	let (front, back) = text.split_at(pos);
	let next_token_start = back.find(token_start);
	match next_token_start {
		None => None,
		Some(start) => {
			let (mid, back) = back.split_at(start);
			let mut end: Option<usize> = None;
			let mut depth = 0;
			let mut state: TokenParserFSM = TokenParserFSM::Normal;
			for (i, c) in back.char_indices() {
				match state {
					TokenParserFSM::Normal => match c {
						'\\' => state = TokenParserFSM::Escape,
						'"' => state = TokenParserFSM::Quote,
						'{' => depth += 1,
						'}' => {
							if depth == 1 {
								end = Some(i);
								break;
							} else {
								depth -= 1;
							}
						},
						_ => {},
					},
					TokenParserFSM::Quote => match c {
						'\\' => state = TokenParserFSM::EscapeInQuote,
						'"' => state = TokenParserFSM::Normal,
						_ => {},
					},
					TokenParserFSM::Escape => state = TokenParserFSM::Normal,
					TokenParserFSM::EscapeInQuote => state = TokenParserFSM::Quote,
				}
			}
			match end {
				Some(len) => Some((front.len() + mid.len(), front.len() + mid.len() + len + 1)),
				None => None,
			}
		},
	}
}

/// In-house CSV parser implementation, following the
/// [RFC-4180 standard](https://www.rfc-editor.org/rfc/rfc4180)
fn read_csv_row<R: BufRead>(reader: &mut utf8_chars::Chars<R>) -> Option<Vec<String>> {
	let mut last_char = '\0';
	let mut in_quote = false;
	let mut cell_buffer = String::new();
	let mut cells: Vec<String> = Vec::new();
	let mut count = 0;
	loop {
		match reader.next() {
			None => {
				// end of file
				if count == 0 {
					return None;
				}
				break;
			},
			Some(cr) => {
				match cr {
					Ok(mut c) => {
						// successfully read a UTF-8 encodded char
						match in_quote {
							true => {
								// quoted text
								if c == '"' {
									in_quote = !in_quote;
									if last_char == '"' {
										cell_buffer.push('"');
										c = '\0';
									}
								} else {
									cell_buffer.push(c);
								}
							},
							false => {
								// unquoted text
								if c == '"' {
									in_quote = !in_quote;
									if last_char == '"' {
										cell_buffer.push('"');
										c = '\0';
									}
								} else if c == ',' {
									// cell delimiter
									cells.push(cell_buffer.clone());
									cell_buffer.clear();
								} else if c == '\r' {
									// csv files typically end with \r\n, but often end with just \n
									// do nothing
								} else if c == '\n' {
									// csv files typically end with \r\n, but often end with just \n
									// Note: skip empty lines
									if cell_buffer.is_empty() && cells.is_empty() {
										// empty line, reset to read next line
										count = 0;
										last_char = '\0';
										continue;
									} else {
										break;
									}
								} else {
									cell_buffer.push(c);
								}
							},
						}
						last_char = c;
					},
					Err(_) => {
						// invalid unicode
						cell_buffer.push_str("�");
					},
				}
			},
		}
		count += 1;
	}
	// push the last cell
	cells.push(cell_buffer.clone());
	return Some(cells);
}

/// Unzips the contents of a zip archive located at the specified `zip_path` and extracts them
/// to the destination directory specified by `dest_dir`.
/// # Arguments
/// * `zip_path` - A reference to the path of the zip archive file to be extracted.
/// * `dest_dir` - A reference to the directory where the contents of the zip archive will be extracted.
/// # Returns
/// Returns a `Result` with the unit type `()` if the operation is successful. If an error occurs
/// during the unzip process, a `ZipError` is returned, encapsulating the specific error information.
fn unzip_file(zip_path: &Path, dest_dir: &Path) -> Result<(), ZipError> {
	let file = File::open(zip_path)?;
	let reader = io::BufReader::new(file);
	let mut zip = zip::ZipArchive::new(reader)?;

	for i in 0..zip.len() {
		let mut entry = zip.by_index(i)?;
		let entry_path = entry.enclosed_name().to_owned();
		if entry_path.is_none() {
			continue;
		}
		let entry_dest = dest_dir.join(entry_path.unwrap());

		if (&*entry.name()).ends_with('/') {
			fs::create_dir_all(&entry_dest)?;
		} else {
			if let Some(p) = entry_dest.parent() {
				if !p.exists() {
					fs::create_dir_all(&p)?;
				}
			}
			let mut outfile = File::create(&entry_dest)?;
			std::io::copy(&mut entry, &mut outfile)?;
		}
	}
	Ok(())
}

#[cfg(test)]
mod unit_tests {
	use crate::{DICE_START, SUB_START, read_csv_row};
	use std::io::BufReader;
	use utf8_chars::BufReadCharsExt;

	#[test]
	fn test_next_token() {
		use crate::next_token;
		assert_eq!(
			next_token(&"one ${two} three".into(), 0, SUB_START),
			Some((4, 10))
		);
		assert_eq!(next_token(&"one ${two} three".into(), 10, SUB_START), None);
		assert_eq!(next_token(&"one} two three".into(), 0, SUB_START), None);
		assert_eq!(
			next_token(&"${one} ${two} ${three}".into(), 0, SUB_START),
			Some((0, 6))
		);
		assert_eq!(
			next_token(&"${one} ${two} ${three}".into(), 6, SUB_START),
			Some((7, 13))
		);
		assert_eq!(
			next_token(&"${one} ${two} ${three}".into(), 13, SUB_START),
			Some((14, 22))
		);
		assert_eq!(
			next_token(&"${one} ${two} ${three}".into(), 22, SUB_START),
			None
		);
		assert_eq!(
			next_token(
				&"one ${{\"name\": \"two\", \"count\": 1}} three".into(),
				0,
				SUB_START
			),
			Some((4, 34))
		);
		assert_eq!(
			next_token(&"#{1d4} five".into(), 0, DICE_START),
			Some((0, 6))
		);
		assert_eq!(
			next_token(&"one #{1d4} three".into(), 0, DICE_START),
			Some((4, 10))
		);
		assert_eq!(next_token(&"one #{1d4} three".into(), 10, DICE_START), None);
	}

	#[test]
	fn test_read_csv_row_01() {
		let mut src = BufReader::new("a,b,c".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
	}

	#[test]
	fn test_read_csv_row_02() {
		let mut src = BufReader::new("a,b,c\r\n".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
	}

	#[test]
	fn test_read_csv_row_03() {
		let mut src = BufReader::new("a,b without quotes,c".as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b without quotes", "c"]
		);
	}

	#[test]
	fn test_read_csv_row_04() {
		let mut src = BufReader::new(r#"a,"b with quotes",c"#.as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b with quotes", "c"]
		);
	}

	#[test]
	fn test_read_csv_row_05() {
		let mut src = BufReader::new(r#"a,b with ""quotes"",c"#.as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b with \"quotes\"", "c"]
		);
	}

	#[test]
	fn test_read_csv_row_06() {
		let mut src = BufReader::new(r#"a,"b with more ""quotes""",c"#.as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b with more \"quotes\"", "c"]
		);
	}

	#[test]
	fn test_read_csv_row_07() {
		let mut src = BufReader::new("a,b,c\r\n1,2,3".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["1", "2", "3"]);
	}

	#[test]
	fn test_read_csv_row_08() {
		let mut src = BufReader::new("a,b,c\r\n\r\n1,2,3".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["1", "2", "3"]);
	}

	#[test]
	fn test_read_csv_row_09() {
		let mut src = BufReader::new("a,b,c\n\n1,2,3\n".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["1", "2", "3"]);
	}

	#[test]
	fn test_read_csv_row_10() {
		let mut src = BufReader::new("a,b,c\n\n\n\n\n1,2,3\n".as_bytes());
		let mut iter = src.chars();
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["a", "b", "c"]);
		assert_eq!(read_csv_row(&mut iter).unwrap(), vec!["1", "2", "3"]);
	}

	#[test]
	fn test_read_csv_row_11() {
		let mut src = BufReader::new("a,\"b with\nnew-line\",c".as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b with\nnew-line", "c"]
		);
	}

	#[test]
	fn test_read_csv_row_12() {
		let mut src = BufReader::new(r#"a,"b with, comma",c"#.as_bytes());
		let mut iter = src.chars();
		assert_eq!(
			read_csv_row(&mut iter).unwrap(),
			vec!["a", "b with, comma", "c"]
		);
	}
}
