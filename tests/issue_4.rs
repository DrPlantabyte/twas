use rand::Rng;
use twas;

const CULTURE_BY_REGION_YAML: &str = r#"
any:
  - iltanno
"#;

const SPECIES_BY_CULTURE_YAML: &str = r#"
iltanno:
  human: 70
"#;

const GENDER_BY_SPECIES_YAML: &str = r#"
default:
  female: 1
human:
  - "${gender-by-species/default}"
"#;

const NAME_BY_CULTURE_YAML: &str = r#"
iltanno:
  female:
    - "${names/iltanno/female} ${names/iltanno/family}"
"#;

const NAMES_ILTANNO_CSV: &str = r#"family,female
Campos,Lisa
"#;

fn load_data(seed: u64) -> twas::Interpreter<impl Rng> {
	let mut interp = twas::Interpreter::from_seed(seed);
	interp
		.load_yaml_str("culture-by-region", CULTURE_BY_REGION_YAML)
		.unwrap();
	interp
		.load_yaml_str("species-by-culture", SPECIES_BY_CULTURE_YAML)
		.unwrap();
	interp
		.load_yaml_str("gender-by-species", GENDER_BY_SPECIES_YAML)
		.unwrap();
	interp
		.load_yaml_str("name-by-culture", NAME_BY_CULTURE_YAML)
		.unwrap();
	interp
		.load_csv_str("names/iltanno", NAMES_ILTANNO_CSV)
		.unwrap();
	interp
}

#[test]
fn test_nested_reference() {
	let mut interpreter = load_data(12345);
	let test_str = r#"${{id: culture-by-region/any, ref: culture, hidden: true}}${{id: "species-by-culture/$culture", ref: species, hidden: true}}${{id: "gender-by-species/$species", ref: gender, hidden: true}}A ${@gender} ${{id: "@culture", case: title}} ${{id: "@species", case: title}} named ${{id: "name-by-culture/$culture/$gender"}}."#;
	eprintln!("       test_str: {}", test_str);
	let expected_output = "A female Iltanno Human named Lisa Campos.";
	eprintln!("expected_output: {}", expected_output);
	eprint!("  actual_output: ");
	let actual_output = interpreter.eval(test_str).unwrap();
	eprintln!("{}", actual_output);
	assert_eq!(expected_output, actual_output);
}
