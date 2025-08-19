#![deny(unused_must_use)]
#![deny(missing_docs)]
use serde::{Deserialize, Serialize};
use serde_yaml;

/// Struct to hold all the possible substitution options for a substitution token
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SubstitutionOptions {
	/// ID of the lookup table to apply
	pub id: String,
	/// Option to specify number of items to draw from the lookup table. Can be either a number or
	/// a dice expression (eg "2d6+3" meaning 'roll two 6-sided dice and then add 3 to the total')
	pub count: Option<serde_yaml::Value>,
	/// If drawing more than one, what method to use. Either "random" for unbiased random draw or
	/// "shuffle" to avoid drawing the same item twice (until all items are used)
	pub method: Option<String>,
	/// If drawing more than one item, separate them with this string
	pub sep: Option<String>,
	/// If drawing more than one item, separate the last two with this string instead of `sep`
	#[serde(rename = "last-sep")]
	pub last_sep: Option<String>,
	/// Prefix this string before each item
	pub prefix: Option<String>,
	/// Postfix this string after each item
	pub suffix: Option<String>,
	/// Specify text capitalization. Must be one of: "upper", "lower", "title", "first", "original"
	pub case: Option<String>,
	/// References allow for re-use of the same substitution with the @ref syntax
	#[serde(rename = "ref")]
	pub reference: Option<String>,
	/// If set to true, do not render this item (useful for references)
	pub hidden: Option<bool>,
	/// If set to true, prefix with correct english indefinite article (a/an)
	pub aan: Option<bool>,
}

impl SubstitutionOptions {
	/// Constructs a new `SubstitutionOptions` with default values
	pub fn new(id: &str) -> Self {
		SubstitutionOptions {
			id: String::from(id),
			count: None,
			method: None,
			sep: None,
			last_sep: None,
			prefix: None,
			suffix: None,
			case: None,
			reference: None,
			hidden: None,
			aan: None,
		}
	}
	/// Constructs a new `SubstitutionOptions` with default values plus a reference ID
	pub fn new_with_ref(id: &str, ref_name: &str) -> Self {
		SubstitutionOptions {
			id: String::from(id),
			count: None,
			method: None,
			sep: None,
			last_sep: None,
			prefix: None,
			suffix: None,
			case: None,
			reference: Some(ref_name.to_string()),
			hidden: None,
			aan: None,
		}
	}
}

#[cfg(test)]
mod unit_tests {
	use crate::subspec::SubstitutionOptions;

	#[test]
	fn test_serde_parse_1() {
		let sub_spec: SubstitutionOptions = serde_yaml::from_str(
			r#"{"id": "animals.plural", "count": 3, "method": "shuffle", "sep": ", ", "last-sep": ", and "}"#
		).expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animals.plural");
		assert_eq!(
			sub_spec.count,
			Some(serde_yaml::Value::Number(serde_yaml::Number::from(3)))
		);
		assert_eq!(sub_spec.method, Some(String::from("shuffle")));
		assert_eq!(sub_spec.sep, Some(String::from(", ")));
		assert_eq!(sub_spec.last_sep, Some(String::from(", and ")));
		assert!(sub_spec.prefix.is_none());
		assert!(sub_spec.suffix.is_none());
		assert!(sub_spec.case.is_none());
		assert!(sub_spec.reference.is_none());
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
	#[test]
	fn test_serde_parse_2() {
		let sub_spec: SubstitutionOptions = serde_yaml::from_str(
			r#"{"id": "animals.plural", "count": "1d4+1", "method": "random", "sep": ", ", "last-sep": ", and "}"#
		).expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animals.plural");
		assert_eq!(
			sub_spec.count,
			Some(serde_yaml::Value::String(String::from("1d4+1")))
		);
		assert_eq!(sub_spec.method, Some(String::from("random")));
		assert_eq!(sub_spec.sep, Some(String::from(", ")));
		assert_eq!(sub_spec.last_sep, Some(String::from(", and ")));
		assert!(sub_spec.prefix.is_none());
		assert!(sub_spec.suffix.is_none());
		assert!(sub_spec.case.is_none());
		assert!(sub_spec.reference.is_none());
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
	#[test]
	fn test_serde_parse_2b() {
		let sub_spec: SubstitutionOptions = serde_yaml::from_str(
			r#"{id: animals.plural, count: 1d4+1, method: random, sep: ", ", last-sep: ", and "}"#,
		)
		.expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animals.plural");
		assert_eq!(
			sub_spec.count,
			Some(serde_yaml::Value::String(String::from("1d4+1")))
		);
		assert_eq!(sub_spec.method, Some(String::from("random")));
		assert_eq!(sub_spec.sep, Some(String::from(", ")));
		assert_eq!(sub_spec.last_sep, Some(String::from(", and ")));
		assert!(sub_spec.prefix.is_none());
		assert!(sub_spec.suffix.is_none());
		assert!(sub_spec.case.is_none());
		assert!(sub_spec.reference.is_none());
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
	#[test]
	fn test_serde_parse_3() {
		let sub_spec: SubstitutionOptions =
			serde_yaml::from_str(r#"{"id": "animals.plural"}"#).expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animals.plural");
		assert!(sub_spec.count.is_none());
		assert!(sub_spec.method.is_none());
		assert!(sub_spec.sep.is_none());
		assert!(sub_spec.last_sep.is_none());
		assert!(sub_spec.prefix.is_none());
		assert!(sub_spec.suffix.is_none());
		assert!(sub_spec.case.is_none());
		assert!(sub_spec.reference.is_none());
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
	#[test]
	fn test_serde_parse_4() {
		let sub_spec: SubstitutionOptions = serde_yaml::from_str(
			r#"{"id": "animals.plural", "count": 3, "prefix": " * ", "suffix": "\n", "case": "first"}"#,
		)
		.expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animals.plural");
		assert_eq!(
			sub_spec.count,
			Some(serde_yaml::Value::Number(serde_yaml::Number::from(3)))
		);
		assert!(sub_spec.method.is_none());
		assert!(sub_spec.sep.is_none());
		assert!(sub_spec.last_sep.is_none());
		assert_eq!(sub_spec.prefix, Some(String::from(" * ")));
		assert_eq!(sub_spec.suffix, Some(String::from("\n")));
		assert_eq!(sub_spec.case, Some(String::from("first")));
		assert!(sub_spec.reference.is_none());
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
	#[test]
	fn test_serde_parse_5() {
		let sub_spec: SubstitutionOptions =
			serde_yaml::from_str(r#"{"id": "animal", "ref": "pet"}"#).expect("Failed to parse");
		assert_eq!(sub_spec.id.as_str(), "animal");
		assert!(sub_spec.count.is_none());
		assert!(sub_spec.method.is_none());
		assert!(sub_spec.sep.is_none());
		assert!(sub_spec.last_sep.is_none());
		assert!(sub_spec.prefix.is_none());
		assert!(sub_spec.suffix.is_none());
		assert!(sub_spec.case.is_none());
		assert_eq!(sub_spec.reference, Some(String::from("pet")));
		assert!(sub_spec.hidden.is_none());
		assert!(sub_spec.aan.is_none());
	}
}
