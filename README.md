# TWAS - The random text substitution app and library
*Twas the night before **${holiday}** and all through the **${building}**, not a creature was stirring, not even **${{id: animal, aan: true}}**.*

*Twas the night before **Halloween** and all through the **barn**, not a creature was stirring, not even **an elephant**.*

TWAS (acronym for Text With Arbitrary Substitutions) is a text substitution tool for replacing identifiers such as `${animal}` with randomly selected items from lists of random word/phrase look-up tables.

# About
**twas** is a randomized text substitution tool for generating random stories, game events, and other miscellany. Given a text prompt and one or more look-up tables, **twas** will replace any `${...}` prompts with randomly selected text from the look-up tables. If the selected text also contains `${...}` prompts, then these will also be replaced (recursively). In this way, **twas** can generate complex stories or descriptions from a few simple tables.

# How to Install
## Installing the `twas` app
Assuming you have installed [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) on your computer, simply rung the following command to install (or update) the **twas** CLI app:
```bash
 cargo install twas --features=app
```

## Installing the `twas` library
In your `Cargo.toml` file, simply add the following to your dependencies section:
```toml
[dependencies]
twas="1"
```

# Usage
To use **twas**, you must first define one or more random look-up tables to reference in your substitution text. Look-up tables can be in any of the following format (format details described below under *Random Look-up Table Formats*): **plain text (.txt), comma-separated values (.csv), JSON (.json), and YAML (.yml or .yaml)**. Multiple files can be loaded together. You can also include a directory, which **twas** will recursively scan for supported file formats and load then, prefixing their IDs with the filepath relative to the provided directory. .zip files can also be included and will be treated similar to directories.

For example, here's a simple random look-up table consisting of a list of animals:
`animal.txt`
```text
antelope
bee
cat
dog
elephant
flying fish
```

Then if you run `twas -i animal.txt "I have a pet ${animal}."`, the text `${animal}` will be replaced with a line chosen randomly from `animal.txt` and printed back to the terminal. Text substitution syntax and options described below under *Text Substitution Syntax and Options*.

# Text Substitution Syntax and Options
Targets for text substitution are identified by a `$` dollar sign followed by `{}` curly braces which enclose the ID of the look-up table to use or a JSON object containing more advanced options. For example `${animal}` in the text string `"I have a pet ${animal}."` or `${ {id: animal, aan: true} }` in the text string `"My pet is ${ {id: animal, aan: true} }."`.

## Random Look-Up IDs
To reference a look-up table, you need to specify its ID. For **.txt** files, the ID is just the filename without the .txt suffix (for example, the id for `animal.txt` is `animal` and can be used for text substitution with `${animal}`). For **.csv** files, the ID is the filename (without the .csv file suffix) followed by a `/` backslash and the column name, for example the ID for column `bird` in CSV file `zoo-animals.csv` is `zoo-animals/bird` and can be used for text substitution with `${zoo-animals/plural}`). For JSON and YAML files, look-up tables can be nested, and similar to .csv files, the levels are delimited with `/` backslash using the filename (without the file suffix) as the base, for example the JSON file `plant.json` with content `{"trees": {"evergreen": ["pine", "cedar"]}}` contains ID `plant/trees/evergreen`. Look-up IDs are case-sensitive and are not allowed to contain either `$` or `@`. See the relevant random look-up table format subsection below for additional details on look-up IDs for each particular file format.

If you prefix the ID with `@` (eg `${@fav-pet}`), then the ID is treated as a reference ID, in which case it re-uses a previous substitution instead of drawing from a look-up table. For example: `I have a pet ${animal@fav-pet} and a pet ${animal}. The ${@fav-pet} is my favorite.` saves the first substitution as reference ID `fav-pet` in the first sentence and then re-uses it in the second sentence. See the *References* section below for a detailed description on the use of references.

## Basic Substitution Syntax
To perform a simple text substitution in your text, simply add a `$` dollar sign immediately followed by `{}` curly braces that enclose the look-up table ID you want to use for substitution, ie `${ look-up ID }`. Whitespace characters (eg spaces) before and after the ID are ignored, thus `${animal}` and `${ animal }` are treated the same.

To save the generated substitution as a reference for later re-use, append `@` followed by the reference ID (eg `${animal@fav-pet}`). To use a saved reference, use just `@` and the reference ID. For example:

`I have a pet ${animal@fav-pet} and a pet ${animal}. The ${@fav-pet} is my favorite.` => `I have a pet dog and a pet cat. The dog is my favorite.`

See the *References* section below for a detailed description on the use of references.

## Advanced Substitution Syntax
Text substitution can also be specified using JSON or YAML syntax using double-curly braces, where the look-up table ID is provided via the `id` field of the JSON/YAML object. For example, `${{id: animal}}` is exactly equivalent to `${animal}`. **twas** provides a number of additional options that can be applied to the text substitution via JSON syntax. The options are as follows:

### id (required)
The ID of the look-up table to draw from for this substitution. If the ID starts with a `@` then it is treated as a reference ID. The ID field can also contain ID substitutions using a `$` to substitute a portion of the ID with a saved reference. For example, in the text `I have a pet ${animal@pet}. Its name is ${{id: "names/$pet"}}`, if `${animal@pet}` substitutes to `dog`, then `${{id: "names/$pet"}}` becomes `${{id: "names/dog"}}` and then substitutes an entry from the `names/dog` look-up table. See the *References* section below for a detailed description on the use of references.

#### example:
`I have a pet ${{id: animal}}.` => `I have a pet dog.`

### count
The `count` option lets you pick multiple items from the look-up table. You can specify either a number or use RPG dice notation (eg "1d6+1") to draw a random number of items from the look-up table. The `count` option is typically used with `sep: ", "` and `last-sep: " and "` to make a comma-separated list. See also `method`, `prefix`, and `suffix`.
#### examples:
* `I have a pet ${{id: animal, count: 2}}.` => `I have a pet dog cat.`
* `My pets: ${{id: animal, count: 2, sep: ", ", last-sep: " and "}}.` => `My pets: dog and cat.`
* `My pets: ${{id: animal, count: 3, sep: ", ", last-sep: " and "}}.` => `My pets: dog, cat and cat.`
* `My pets: ${{id: animal, count: 3, sep: ", ", last-sep: " and ", method: shuffle}}.` => `My pets: dog, cat and bird.`

### method
The `method` option specifies which random look-up algorithm is used to draw from the look-up table when drawing multiple items with the `count` option. The supported methods are **"random"** and **shuffle**. With **"random"**, the same item may be drawn multiple times in a row. With **shuffle** the same item will not be drawn again unless `count` is higher than the total number of items in the look-up table. The default method is **"random"**.
#### examples:
* `My pets: ${{id: animal, count: 3, sep: ", ", last-sep: " and "}}.` => `My pets: dog, cat and cat.`
* `My pets: ${{id: animal, count: 3, sep: ", ", last-sep: " and ", method: shuffle}}.` => `My pets: dog, cat and bird.`

### sep
When using the `count` option, the provided `sep` string is placed between each item. If not specified, the default value is a space character. The `count` option is typically used with `sep: ", "` and `last-sep: " and "` to make a comma-separated list. See also `method`, `prefix`, and `suffix`.

#### example:
`List of pets: ${{id: animal, count: 2, sep: ", "}}.` => `List of pets: dog, cat.`

### last-sep
The `last-sep` like `sep` described above, but only placed between the last and second-to-last items
#### example:
* `My pets: ${{id: animal, count: 3, sep: ", ", last-sep: " and ", method: shuffle}}.` => `My pets: dog, cat and bird.`

### prefix
The `prefix` is added in front of each item drawn from the random look-up table. This is particularly useful when making random lists with the `count` option.
#### example:
`My pets:${{id: animal, count: 2, "prefix": "\n * "}}` =>
```text
My pets:
 * dog
 * cat
```

### suffix
The `suffix` is added at the end of each item drawn from the random look-up table. This is particularly useful when making random lists via the `count` option with output in a structured format like HTML.
#### example:
`My pets:<br><ul>${{id: animal, count: 2, "prefix": "\n<li>", "prefix": "</li>"}}</ul>` =>
```text
My pets:<br><ul>
<li>dog</li>
<li>cat</li></ul>
```

### case
The `case` option modifies the capitalization of the substituted text. The behavior generally assumes that the text in the random look-up tables is all lowercase. Supported `case` values are:

| case     | description                         | example             |
|----------|-------------------------------------|---------------------|
| original | No change (default)                 | big blue 3D glasses |
| upper    | All capital letters                 | BIG BLUE 3D GLASSES |
| lower    | All lowercase                       | big blue 3d glasses |
| title    | Capitalize first letter of ea. word | Big Blue 3D Glasses |
| first    | Capitalize first letter only        | Big blue 3D glasses |

### ref
If you use the `ref` option, the randomly selected item(s) from the random look-up table will be saved for re-use under the provided reference ID. See the *References* section below for a detailed description on the use of references.
#### example:
`I have a pet ${{id: animal, ref: "fav-pet"}} and a pet ${{id: animal}}. The ${@fav-pet} is my favorite.` => `I have a pet dog and a pet cat. The dog is my favorite.`

### hidden
If set to true, the `hidden` prevents the substitution from appearing in the text. This is only useful if combined with the `ref` option described above, in which case the hidden substitution is saved as a variable for use later in the text.
#### example:
`${{id: animal, ref: pet, "hidden": true}}I have a pet ${{id: "@pet"}}.` => `I have a pet dog.`

### aan
If `aan` is set to true, then the indefinite article `a` or `an` is added before the substitution text, as appropriate for the spelling of the randomly chosen item from the look-up table.
#### example:
`My favorite animal is ${{id: animal, aan: true}}.` => `My favorite animal is a dog.`

## References
When you want to use the same result in multiple places, you can use a reference to save a generated result and re-use it again. For example, suppose you are creating a story about a pet chosen randomly from the `animal` look-up table. Since the story refers to the same pet multiple times, you'd only want to draw from the `animal` random look-up table once. To achieve this, you would save the first `animal` usage as reference `pet`, and then everywhere you want to use the same reference, specify `@pet` as the ID instead of `animal`. Thus your story text might look like `"I have a pet ${animal@pet}. ${{id: "@pet", aan: true, "case": "first"}} is a good animal to have as a pet. I love my ${@pet}!"`, and if `${animal@pet}` resolves to `dog` then that becomes `"I have a pet dog. A dog is a good animal to have as a pet. I love my dog!"`.

### Creating References
If using basic substitution syntax, you create a reference by simply putting `@` after the look-up table ID followed by the reference ID for storing this substitution, for example `${animal@pet}` will draw a random entry from the `animal` lookup table and save the result as reference ID `pet`. If using JSON syntax, then you instead use the `ref` option to save the result to the given reference ID, for example `${{id: animal, ref: pet}}`.

### Using References for Text Substitution
References are stroed by the provided reference ID, and reference IDs are used like look-up table IDs, but with a `@` prefix. For example, `${@pet}` will be substituted with the saved `pet` reference, as will `${{id: "@pet"}}`.

If using JSON syntax, you can still apply additional options to the referenced text, such as `aan` or `case`. 

### Using References for ID Substitution
You can replace part of the ID string with the value of a saved reference. In this case, you use `$` followed by the reference ID as part of the look-up ID (eg `${pet-names/$pet}`). This allows you to use the result of one random look-up table to determine which other look-up table to use.

For example, suppose you load the following `animal.txt` file:
```text
dog
cat
bird
rat
```
And also load the following `pet-names.csv` file:
```text
bird,cat,dog,rat
pip,paws,spot,whiskers
seed,mew,spike,cheesy
lala,claws,rolf,nibbler
```
Then you could use a chosen animal to pick a specific name, like this:
`My pet ${animal@pet}'s name is ${{id: "pet-names/$pet", "case": "title"}}.` => `My pet dog's name is Spot.`

## Random Numbers with Dice Notation
You can also insert random numbers into your text with RPG dice notation. Number substitutions start with a `#` hash symbol followed by `{}` curly braces enclosing the dice expression, for example `#{1d6+2}` will be replaced with a random number from 3 to 8 (the expression "1d6+2" means "roll 1 die with 6 sides and add 2"). See [the dicexp crate](https://crates.io/crates/dicexp) for more details on supported dice expression syntax.

# Random Look-up Table Formats
Several different formats are supported for defining random look-up tables. The supported formats are described in detail here.

## .txt
Each line in a `.txt` file will be parsed as an entry in a look-up table, with all possible values having equal weight. 

### IDs
The ID for this look-up table is just the name of this file without the `.txt` ending (eg the ID for `animal.txt` is `animal`).

### Examples
The following is an example of a random look-up with ID `animal` that randomly resolved to any one of `dog`, `cat`, `bird`, or `rat` with equal probability:
`animal.txt`
```text
dog
cat
bird
rat
```

## .csv
`.csv` files are interpreted as standard comma-separate value (CSV) files (UTF-8 encoding), where the first row is the header row containing column names and all subsequent rows are the possible values for each column. Each column is its own random look-up table. All rows have equal probability, unless there is a column named `weight`. If a `weight` column is present, then the probability of each row is weighted by the decimal value in the corresponding `weight` column.

### IDs
The ID for each column in the CSV file is `filename/column` (eg `pet-names/dog` for column `dog` in file `pet-names.csv`).

### Examples
The following example create four different random look-up tables with IDs `pet-names/bird`, `pet-names/cat`, `pet-names/dog`, `pet-names/rat`:
`pet-names.csv`
```text
bird,cat,dog,rat
pip,paws,spot,whiskers
seed,mew,spike,cheesy
lala,claws,rolf,nibbler
```

The following example creates a look-up table with ID `rarity/rarity` that has a 60% probability of drawing "common", 30% probability of "uncommon", 9% probability of "rare", and 1% probability of "very rare":
`rarity.csv`
```text
weight,rarity
6,common
3,uncommon
0.9,rare
0.1,very rare
```

## .yaml (and .yml)
A YAML file can contain one or multiple random look-up tables, with arbitrary levels of nested depth. Any lists encountered in the YAML file will be parsed as look-up tables with equal probability for all items, while weighted-probabilities are specified using a string-number mapping (eg `rarity: {common: 6, uncommon: 3, rare: 0.9, "very rare": 0.1}`). The tables can be organized by nesting map objects, with each nesting adding a level to the look-up table ID path.

### IDs
The ID for each random look-up in a YAML file is equal to the filename (without the file suffix) followed by the object-map path in the file to the look-up table.

In the simple case of a file that only contains a list (see `animal.yaml` in the examples below), the ID is just the filename. For an object in the base of the file, the ID would be `filename/name` (see `treasure.yaml` in the examples below), while a nested object's ID would have ID where the nested path is represetned with `/` backslash delimiters (ie  `filename/level1/level2/.../name`, see `colors.yaml` example below).

### Examples
The following is an example of a random look-up with ID `animal` that randomly resolved to any one of `bird`, `cat`, `dog`, or `rat` with equal probability:
`animal.yaml`
```yaml
- bird
- cat
- dog
- rat
```

The following example creates a look-up table creates two tables with IDs `treasure/money` and `treasure/junk`, where the entries in `treasure/money` have different probabilities, but the entries in `treasure/junk` all have equal probability:
`treasure.yaml`
```yaml
money:
  "100 copper pennies": 4
  "10 silver dollars": 1.5
  "1 gold ingot": 0.5
junk:
  - old boot
  - pocket lint
  - broken toy boat
```

The following example creates three look-up tables with IDs `colors/light/primary`, `colors/paint/primary`, `colors/paint/secondary/pastel`:
`colors.yaml`
```yaml
light:
  primary:
    - red
    - green
    - blue
paint:
  primary:
    - red
    - yellow
    - blue
  secondary:
    pastel:
      - peach
      - light green
      - lavender
```

## .json
JSON files work exactly the same as YAML (see above).

### IDs
Same as for YAML parsing, described above.

### Examples
The following is an example of a random look-up with ID `animal` that randomly resolved to any one of `bird`, `cat`, `dog`, or `rat` with equal probability:
`animal.json`
```json
[
 "bird",
 "cat",
 "dog",
 "rat"
]
```

The following example creates a look-up table creates two tables with IDs `treasure/money` and `treasure/junk`, where the entries in `treasure/money` have different probabilities, but the entries in `treasure/junk` all have equal probability:
`treasure.json`
```json
{
 "money": {
  "100 copper pennies": 4,
  "10 silver dollars": 1.5,
  "1 gold ingot": 0.5
 },
 "junk": [
  "old boot",
  "pocket lint",
  "broken toy boat"
 ]
}
```

The following example creates three look-up tables with IDs `colors/light/primary`, `colors/paint/primary`, `colors/paint/secondary/pastel`:
`colors.json`
```json
{
 "light": {
  "primary": [
   "red",
   "green",
   "blue"
  ]
 },
 "paint": {
  "primary": [
   "red",
   "yellow",
   "blue"
  ],
  "secondary": {
   "pastel": [
    "peach",
    "light green",
    "lavender"
   ]
  }
 }
}
```

## directories
When you load a directory, **twas** will recursively scan the directory for all supported file formats and load all of those files. The IDs of al the loaded files will be prefixed by their relative directory filepaths within the loaded directory. Thus if you load directory `foo/bar`, file `foo/bar/animal.txt` will have ID `animal` but file `foo/bar/vehicles/cars.txt` will have ID `vehicles/cars`.

### IDs
The IDs for the files loaded in the directory will be prefixed with their relative subdirectory paths within the loaded directory.

### Examples:
Given the following directory structure and the files from the examples above:
```text
resources/
 ├─ adventure/
 │   ├─ rarity.csv
 │   └─ treasure.yaml
 ├─ colors.yaml
 └─ pets/
     ├─ animal.txt
     └─ pet-names.csv
```
Then loading the `resources` directory would register the following list of random look-up table IDs:
```text
adventure/rarity/rarity
adventure/treasure/money
adventure/treasure/junk
colors/light/primary
colors/paint/primary
colors/paint/secondary/pastel
pets/animal
pets/pet-names/bird
pets/pet-names/cat
pets/pet-names/dog
pets/pet-names/rat
```

## .zip
When **twas** loads a `.zip` file, it extracts it and treats its contents like a directory (see above).

### IDs
The IDs for the files loaded in the zip archive file will be prefixed with their relative subdirectory paths within the loaded zip archive.

### Examples:
Given the following zip archive structure and the files from the examples above:
```text
resources.zip
 ├─ adventure/
 │   ├─ rarity.csv
 │   └─ treasure.yaml
 ├─ colors.yaml
 └─ pets/
     ├─ animal.txt
     └─ pet-names.csv
```
Then loading the `resources` directory would register the following list of random look-up table IDs:
```text
adventure/rarity/rarity
adventure/treasure/money
adventure/treasure/junk
colors/light/primary
colors/paint/primary
colors/paint/secondary/pastel
pets/animal
pets/pet-names/bird
pets/pet-names/cat
pets/pet-names/dog
pets/pet-names/rat
```

# License and Redistribution
The **twas** source code is subject to the terms of the [Mozilla Public License, v. 2.0](https://mozilla.org/MPL/2.0/).