#![deny(unused_must_use)]
#![deny(missing_docs)]
use crate::errors::NoValuesError;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// An item represents an entry in a random look-up table. It has a probability weight and a text
/// value
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Item {
	/// The look-up value (text)
	text: String,
	/// The probability weight for drawing this item from the look-up table
	weight: f64,
}

impl Item {
	/// Get a reference to the text value of the item.
	/// # Returns
	/// A reference to the text value stored in this `Item`.
	pub fn get_text(&self) -> &String {
		&self.text
	}

	/// Get the probability weight of the item.
	/// # Returns
	/// The probability weight associated with this `Item`.
	pub fn get_weight(&self) -> f64 {
		self.weight
	}
}

/// A random lookup table that holds items with associated weights for random selection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LookUpTable {
	items: Vec<Item>,
	total: f64,
	equal_weights: bool,
}

impl LookUpTable {
	/// Creates a new, empty `LookUpTable` with default settings.
	pub fn new() -> Self {
		LookUpTable { items: Vec::new(), total: 0., equal_weights: true }
	}

	/// Draws one item at random from the lookup table or returns a `NoValuesError` if there are
	/// no items to draw from.
	/// # Arguments
	/// * `rng` - A random number generator implementing the `Rng` trait.
	/// # Returns
	/// Returns a randomly selected `Item` or a `NoValuesError` if the table is empty.
	pub fn draw_random(&self, rng: &mut impl Rng) -> Result<Item, NoValuesError> {
		if self.items.len() == 0 {
			return Err(NoValuesError {});
		}
		if self.equal_weights {
			// simple integer draw
			let i = rng.random_range(0..self.items.len());
			Ok(self.items[i].clone())
		} else {
			let mut draw = self.total * rng.random_range(0f64..1f64);
			for item in &self.items {
				if draw <= item.weight {
					return Ok(item.clone());
				}
				draw -= item.weight;
			}
			assert!(
				false,
				"Logic violation. Output of random number generator exceeded range of 0-1"
			);
			return Ok(self.items.last().unwrap().clone());
		}
	}

	/// Draws a specified number of items at random from the lookup table (with possible duplicates)
	/// or returns a `NoValuesError` if there are no items to draw from.
	/// # Arguments
	/// * `rng` - A random number generator implementing the `Rng` trait.
	/// * `count` - The number of items to draw.
	/// # Returns
	/// Returns a vector of randomly selected `Item`s or a `NoValuesError` if the table is empty.
	pub fn draw_n_random(
		&self,
		rng: &mut impl Rng,
		count: usize,
	) -> Result<Vec<Item>, NoValuesError> {
		let mut result: Vec<Item> = Vec::with_capacity(count);
		for _ in 0..count {
			result.push(self.draw_random(rng)?);
		}
		Ok(result)
	}

	/// Shuffles all the items and returns the shuffled list or returns a NoValuesError is there are
	/// no items to draw from
	/// # Arguments
	/// * `rng` - A random number generator implementing the `Rng` trait.
	/// * `count` - The number of items to draw.
	/// # Returns
	/// Returns a vector of randomly selected `Item`s or a `NoValuesError` if the table is empty.
	pub fn shuffle(&self, rng: &mut impl Rng) -> Result<Vec<Item>, NoValuesError> {
		if self.items.len() == 0 {
			return Err(NoValuesError {});
		}
		let mut copy = self.items.clone();
		for i in copy.len() - 1..1 {
			let j = rng.random_range(0..=i);
			copy.swap(j, i);
		}
		Ok(copy)
	}

	/// Shuffles and draws the requested number or items or returns a NoValuesError is there are no
	/// items to draw from
	/// # Arguments
	/// * `rng` - A random number generator implementing the `Rng` trait.
	/// * `count` - The number of items to draw.
	/// # Returns
	/// Returns a vector of randomly selected `Item`s or a `NoValuesError` if the table is empty.
	pub fn shuffle_draw(&self, rng: &mut impl Rng, count: usize) -> Result<Vec<Item>, NoValuesError> {
		if self.items.len() == 0 {
			return Err(NoValuesError {});
		}
		let s = self.items.len();
		let mut buffer: Vec<Item> = Vec::with_capacity(s * (1 + (count % s)));
		while buffer.len() < count {
			buffer.extend(self.shuffle(rng)?);
		}
		buffer.truncate(count);
		Ok(buffer)
	}

	/// Adds an item to the lookup table.
	/// # Arguments
	/// * `item` - The `Item` to add to the table.
	/// # Panics
	/// Panics if the item's weight is negative or NaN.
	pub fn add(&mut self, item: Item) {
		if item.weight >= 0. {
			let w = item.weight;
			if self.items.len() > 0 {
				self.equal_weights = self.equal_weights && self.items.last().unwrap().weight == w;
			}
			self.total += w;
			self.items.push(item);
		} else {
			// do not add negative or NaN weighted items
			panic!("Invalid state: item weight must be a positive real number");
		}
	}

	/// Adds an item to the lookup table by specifying its text and weight.
	/// # Arguments
	/// * `text` - The text value for the new item (accepts both &str and String).
	/// * `weight` - The weight for the new item.
	/// # Panics
	/// Panics if the item's weight is negative or NaN.
	pub fn add_item<T>(&mut self, text: T, weight: f64)
	where
		T: Into<String>,
	{
		self.add(Item { text: text.into(), weight })
	}

	/// Removes an item from the lookup table based on its text value.
	/// # Arguments
	/// * `text` - The text value to search for and remove (accepts both &str and String).
	/// # Returns
	/// Returns `true` if an item matching the given text was found and removed, otherwise `false`.
	pub fn remove_item<T>(&mut self, text: T) -> bool
	where
		T: Into<String>,
	{
		let text = text.into();
		let mut removed = false;
		let mut i = self.items.len();
		while i > 0 {
			i -= 1;
			if &self.items[i].text == &text {
				removed = true;
				self.items.remove(i);
			}
		}
		self.recount();
		removed
	}

	/// Re-evaluates the sum of all weights
	fn recount(&mut self) {
		let mut sum = 0f64;
		for item in &self.items {
			sum += item.weight;
		}
		self.total = sum;
	}
}

#[cfg(test)]
mod unit_tests {
	use crate::data::{Item, LookUpTable};

	#[test]
	fn weight_check() {
		let w = 0.5f64;
		let text = "test";
		let i = Item { text: String::from(text), weight: w };
		assert_eq!(i.get_weight(), w);
		let mut lut = LookUpTable::new();
		assert_eq!(lut.total, 0f64);
		lut.add(i);
		assert_eq!(lut.total, w);
		lut.add_item("test2", w);
		assert_eq!(lut.total, (w + w));
		assert!(lut.remove_item(text));
		assert!(!lut.remove_item(text));
		assert_eq!(lut.total, w);
	}
}
