#![deny(unused_must_use)]
#![deny(missing_docs)]
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::num::ParseFloatError;
use zip;

/// Represents an error that occurs during parsing of look-up tables or text substitution
#[derive(Debug)]
pub enum ParsingError {
	ParseError(ParseError),
	IOError(std::io::Error),
	InvalidIDError(InvalidIDError),
	ZipError(zip::result::ZipError),
	KeyNotFoundError(KeyNotFoundError),
	NoValuesError(NoValuesError),
	RecursionLimitReached(RecursionLimitReached),
	InvalidCombinationError(InvalidCombinationError),
	SerdeYAMLParserError(serde_yaml::Error)
}

impl From<ParseError> for ParsingError {
	fn from(value: ParseError) -> Self { ParsingError::ParseError(value) }
}

impl From<ParseFloatError> for ParsingError{
	fn from(value: ParseFloatError) -> Self { ParsingError::ParseError(ParseError{
		msg: Some(format!("{}", value)),
		line: None,
		col: None,
	}) }
}

impl From<std::io::Error> for ParsingError {
	fn from(value: std::io::Error) -> Self { ParsingError::IOError(value) }
}

impl From<InvalidIDError> for ParsingError {
	fn from(value: InvalidIDError) -> Self { ParsingError::InvalidIDError(value) }
}

impl From<zip::result::ZipError> for ParsingError {
	fn from(value: zip::result::ZipError) -> Self { ParsingError::ZipError(value) }
}

impl From<KeyNotFoundError> for ParsingError {
	fn from(value: KeyNotFoundError) -> Self { ParsingError::KeyNotFoundError(value) }
}

impl From<NoValuesError> for ParsingError {
	fn from(value: NoValuesError) -> Self { ParsingError::NoValuesError(value) }
}

impl From<RecursionLimitReached> for ParsingError {
	fn from(value: RecursionLimitReached) -> Self { ParsingError::RecursionLimitReached(value) }
}

impl From<InvalidCombinationError> for ParsingError {
	fn from(value: InvalidCombinationError) -> Self { ParsingError::InvalidCombinationError(value) }
}

impl From<serde_yaml::Error> for ParsingError {
	fn from(value: serde_yaml::Error) -> Self { ParsingError::SerdeYAMLParserError(value) }
}

/// Represents an error that occurs during parsing with additional information.
#[derive(Clone)]
pub struct ParseError {
	/// The error message.
	pub msg: Option<String>,
	/// The line where the error occurred, if known.
	pub line: Option<u64>,
	/// The column where the error occurred, if known.
	pub col: Option<u64>,
}

impl ParseError{
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match &self.msg {
			None => write!(f, "Could not parse")?,
			Some(s) => write!(f, "{}", s)?,
		}
		match &self.line {
			None => {}
			Some(s) => {
				write!(f, ", error on line {}", s)?;
				match &self.col {
					None => {}
					Some(c) => write!(f, ", column {}", c)?,
				}
			},
		}
		Ok(())
	}
}

impl Debug for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for ParseError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for ParseError {}


#[derive(Clone)]
pub struct KeyNotFoundError {
	pub key: String
}

impl KeyNotFoundError{
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Key '{}' not found in look-up table", self.key)
	}
}

impl Debug for KeyNotFoundError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for KeyNotFoundError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for KeyNotFoundError {}



#[derive(Clone)]
pub struct NoValuesError {}

impl NoValuesError{
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Empty data set; no values to choose from")
	}
}

impl Debug for NoValuesError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for NoValuesError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for NoValuesError {}


#[derive(Clone)]
pub struct RecursionLimitReached {
	pub limit: usize
}

impl RecursionLimitReached{
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Recursion limit {} exceeded. Substitution text contains circular reference or is too complex to operate upon.", self.limit)
	}
}

impl Debug for RecursionLimitReached {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for RecursionLimitReached {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for RecursionLimitReached {}


#[derive(Clone)]
pub struct InvalidIDError {
	msg: String
}

impl InvalidIDError{
	/// Creates a new `InvalidIDError` with a custom message.
	pub fn new<T>(msg: T) -> InvalidIDError where T: Into<String> {
		InvalidIDError{msg: msg.into()}
	}
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl Debug for InvalidIDError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for InvalidIDError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for InvalidIDError {}


#[derive(Clone)]
pub struct InvalidCombinationError {
	msg: String
}

impl InvalidCombinationError{
	/// Creates a new `InvalidCombinationError` with a custom message.
	pub fn new<T>(msg: T) -> InvalidCombinationError where T: Into<String> {
		InvalidCombinationError{msg: msg.into()}
	}
	/// Formats and prints the error message
	fn print(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.msg)
	}
}

impl Debug for InvalidCombinationError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl core::fmt::Display for InvalidCombinationError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		self.print(f)
	}
}

impl Error for InvalidCombinationError {}
