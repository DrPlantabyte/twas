use rand::{SeedableRng, RngCore, Error};
use bytemuck;
use twas;

// TESTS //

#[test]
fn load_basic_text() {
	let mut gen = twas::Interpreter::from_seed(12345);
	let input = include_str!("test-data/basic_text.txt");
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(output.as_str(), input, "Text should not have been modified");
}

#[test]
fn single_sub_test_1() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	let input = include_str!("test-data/single_sub.txt");
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(input.replace("${animal}", "dog"), output.as_str(), "Incorrect evaluation");
}

#[test]
fn single_sub_test_2() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/animal.txt").expect("IO Error");
	let input = include_str!("test-data/single_sub.txt");
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(input.replace("${animal}", "dog"), output.as_str(), "Incorrect evaluation");
}

#[test]
fn csv_test() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/words.csv").expect("IO Error");
	println!("{:#?}", gen);
	let input = r#"${{"id": "words/single", "case": "first"}} and three ${words/plural}"."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(
		input.replace(r#"${{"id": "words/single", "case": "first"}}"#, "Goose")
			.replace(r#"${words/plural}"#, "geese"),
		output, "Incorrect evaluation"
	);
}
#[test]
fn weighted_csv_test() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/quality.csv").expect("IO Error");
	let input = r#"A ${quality/adj} ${quality/noun}."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(
		input.replace(r#"${quality/adj}"#, "good")
			.replace(r#"${quality/noun}"#, "thing"),
		output, "Incorrect evaluation"
	);
}

#[test]
fn json_test_1() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/loot.json").expect("IO Error");
	println!("Registry: {:?}", gen.list_ids());
	let input = r#"${loot/coins} and a(n) ${loot/junk}."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_ne!(input, output.as_str(), "Failed to apply substitutions");
}

#[test]
fn json_test_2() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/color.json").expect("IO Error");
	let input = r#"I like ${color}."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(
		input.replace(r#"${color}"#, "black"),
		output, "Incorrect evaluation"
	);
}

#[test]
fn json_test_3() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/rarity.json").expect("IO Error");
	let input = r#"A ${rarity} item."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(
		input.replace(r#"${rarity}"#, "common"),
		output, "Incorrect evaluation"
	);
}

#[test]
fn dice_test_1() {
	use regex::Regex;
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	let input = "#{2d4} items.";
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert!(Regex::new("^[2-8] items.$").unwrap().is_match(output.as_str()), "Incorrect evaluation");
}

#[test]
fn dice_test_2() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/loot.json").expect("IO Error");
	let input = r#"You found:
${{"id": "loot/junk", "count": "2d4", "prefix": " * ", "suffix": "\n"}}"#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!(r#"You found:
 * old boot
 * old boot
 * old boot
 * old boot
 * old boot
 * old boot
"#, output.as_str(), "Incorrect evaluation");
}

#[test]
fn ref_test_1() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	let input = "I like a good ${animal@fav}. A ${@fav} is my favorite animal.";
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "I like a good dog. A dog is my favorite animal.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn ref_test_2() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	let input = r#"I like a good ${{"id": "animal", "ref": "fav"}}. A ${{"id": "@fav"}} is my favorite animal."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "I like a good dog. A dog is my favorite animal.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn ref_test_3() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	gen.load_str("animal_names", include_str!("test-data/animal_names.csv"), "csv").expect("Failure");
	let input = r#"I have a pet ${animal@pet} named ${{"id": "animal_names/$pet", "case": "title", "ref": "petname"}}. ${{"id": "@petname", "case": "title"}} is a good boy."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "I have a pet dog named Spike. Spike is a good boy.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn hidden_test_1() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	let input = r#"${{"id": "animal", "ref": "pet", "hidden": true}}I like a good ${@pet}."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "I like a good dog.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn indefinite_article_test_1() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal.txt"), "txt").expect("Failure");
	let input = r#"${{"id": "animal", "aan": true, "case": "first"}} is a man's best friend."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "A dog is a man's best friend.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn indefinite_article_test_2() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal2.txt"), "txt").expect("Failure");
	let input = r#"${{"id": "animal", "aan": true, "case": "first"}} is a man's best friend."#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "An elephant is a man's best friend.", output.as_str(), "Incorrect evaluation");
}

#[test]
fn indefinite_article_test_3() {
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_str("animal", include_str!("test-data/animal2.txt"), "txt").expect("Failure");
	let input = r#"${{id: animal, ref: pet, hidden: true}}${{id: "@pet", aan: true, case: first}} is a man's best friend. I like the ${@pet}!"#;
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	assert_eq!( "An elephant is a man's best friend. I like the elephant!", output.as_str(), "Incorrect evaluation");
}



#[test]
fn dir_test_1() {
	use regex;
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/testdir").expect("Failed to load dir");
	let mut loaded_ids = gen.list_ids();
	loaded_ids.sort();
	println!("loaded_ids = {:?}", loaded_ids);
	assert_eq!(&loaded_ids[..], &["elf/names/female", "elf/names/male", "elf/names/nonbinary",
		"elf/names/surname", "gender", "human/names/female", "human/names/male",
		"human/names/nonbinary", "human/names/surname", "kind/species", "kind/weight"]);
	let input = "${{id: kind/species, ref: kind, hidden: true}}${{id: gender, ref: gender, hidden: true}}\
	A ${@gender} ${@kind} named ${$kind/names/$gender}.";
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	let matcher = regex::Regex::new("A (male|female|nonbinary) (human|elf) named (\\p{L}+).").unwrap();
	assert!( matcher.is_match(output.as_str()), "Incorrect evaluation");
}

#[test]
fn zip_test_1() {
	use regex;
	let mut gen = twas::Interpreter::from_rng(NotRandom::seed_from_u64(0));
	gen.load_file("tests/test-data/testzip.zip").expect("Failed to load zip archive");
	let mut loaded_ids = gen.list_ids();
	loaded_ids.sort();
	println!("loaded_ids = {:?}", loaded_ids);
	assert_eq!(&loaded_ids[..], &["elf/names/female", "elf/names/male", "elf/names/nonbinary",
		"elf/names/surname", "gender", "human/names/female", "human/names/male",
		"human/names/nonbinary", "human/names/surname", "kind/species", "kind/weight"]);
	let input = "${{id: kind/species, ref: kind, hidden: true}}${{id: gender, ref: gender, hidden: true}}\
	A ${@gender} ${@kind} named ${$kind/names/$gender}.";
	print!("\ninput = '{}'\n", input);
	let output = gen.eval(input).unwrap();
	println!("output = '{}'", output);
	let matcher = regex::Regex::new("A (male|female|nonbinary) (human|elf) named (\\p{L}+).").unwrap();
	assert!( matcher.is_match(output.as_str()), "Incorrect evaluation");
}

#[test]
#[allow(unused_imports)]
fn example01(){
	use twas;
	let mut interpreter = twas::Interpreter::new();
	interpreter.load_file("tests/test-data/animal.txt").expect("Failed to load file");
	let story = "I have a pet ${animal@pet}. I love my ${@pet}!";
	println!("{}", interpreter.eval(story).expect("Failed to eval"));
}
#[test]
#[allow(unused_imports)]
fn example02(){
	use twas;
	let mut interpreter = twas::Interpreter::new();
	interpreter.load_file("tests/test-data/animal.txt").expect("Failed to load file");
	interpreter.load_file("tests/test-data/animal_names.csv").expect("Failed to load file");
	println!("Registry: {:?}", interpreter.list_ids());
	let story = r#"I have a pet ${animal@pet}. His name is ${{id: "animal_names/$pet", case: title}}! ${{id: "@pet", aan: true, case: "first"}} is a girl's best friend."#;
	println!("{}", interpreter.eval(story).expect("Failed to eval"));
}

// UTILS //

#[derive(Debug)]
struct NotRandom {
	seed: u64
}

impl SeedableRng for NotRandom {
	type Seed = [u8; 8];

	fn from_seed(seed: Self::Seed) -> Self {
		NotRandom{seed: bytemuck::cast(seed)}
	}

	fn seed_from_u64(state: u64) -> Self {
		NotRandom{seed: state}
	}
}

impl RngCore for NotRandom {
	fn next_u32(&mut self) -> u32 {
		self.seed as u32
	}

	fn next_u64(&mut self) -> u64 {
		self.seed
	}

	fn fill_bytes(&mut self, dest: &mut [u8]) {
		for i in 0..dest.len(){
			dest[i] = self.seed as u8;
		}
	}

	fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
		self.fill_bytes(dest);
		Ok(())
	}
}
