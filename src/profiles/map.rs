use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::mem::{replace, take};
use super::DataProfile;
use crate::datum::{Datum, Equals};
use crate::{DeserializationError, TransformResult};


/// A base data profile for data that is stored in a map, with keys and values
/// Keys are always static strings
/// You will not instantiate this directly, but you will make aliases to this using the make_data_profile macro
#[derive(Debug)]
pub struct MappedData {
	pub(crate) serializing: bool,
	pub(crate) map: HashMap<String, Datum>
}


impl DataProfile for MappedData {
	fn is_serial(&self) -> bool {
		self.serializing
	}
	fn serial_ready() -> Self {
		Self {
			serializing: true,
			map: HashMap::new(),
		}
	}
}


impl MappedData {
	/// Serialize a named value as an entry
	/// The value must be able to turn into a Datum by implementing Into<Datum>
	pub fn serialize_entry<T: Into<Datum>>(&mut self, name: &'static str, value: T) {
		self.map.insert(name.into(), value.into());
	}
	/// Deserialize a named entry
	/// The value must be able to come from a Datum by implementing TryFrom<Datum>
	pub fn deserialize_entry<T, E>(&mut self, name: &'static str, into: &mut T) -> Result<(), DeserializationError>
		where E: Into<DeserializationError>,
			  T: TryFrom<Datum, Error=E> + Default
	{
		let _ = replace(
			into,
			self.map.remove(name).ok_or(DeserializationError::MissingField(name))?.try_into().transform(name)?
		);
		Ok(())
	}
	/// Deserialize a named entry that is one of the given matches
	/// The value will be taken from matches
	pub fn deserialize_matched_entry<T, I>(&mut self, name: &'static str, value: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + Equals<Datum> + Debug,
			I: Iterator<Item=T>
	{
		let item = self.map.remove(name).ok_or(DeserializationError::MissingField(name))?;
		
		for maybe_match in matches {
			if maybe_match.equals(&item) {
				let _ = replace(value, maybe_match);
				return Ok(())
			}
		}
		
		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch {field: name, actual: debug_string})
	}
	/// Deserialize a named entry that is one of the given matches
	/// The value will be cloned from matches
	pub fn deserialize_cloned_matched_entry<'a, T, I>(&mut self, name: &'static str, value: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + Equals<Datum> + Debug + Clone + 'a,
			I: Iterator<Item=&'a T>
	{
		let item = self.map.remove(name).ok_or(DeserializationError::MissingField(name))?;
		
		for maybe_match in matches {
			if maybe_match.equals(&item) {
				let _ = replace(value, maybe_match.clone());
				return Ok(())
			}
		}
		
		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch {field: name, actual: debug_string})
	}
	/// Either serializes or deserializes a named entry
	/// The data type of the field must be able to turn into or come from a Datum
	pub fn serde_entry<T>(&mut self, name: &'static str, value: &mut T) -> Result<(), DeserializationError>
		where T: Into<Datum> + TryFrom<Datum, Error=DeserializationError> + Default
	{
		if self.serializing {
			self.serialize_entry(name, take(value));
			return Ok(());
		}
		self.deserialize_entry(name, value)
	}
	/// Either serializes or deserializes a named entry that can only be an item in matches
	/// Note that the value only needs to be present in matches during deserialization
	/// The data type of the field must be able to turn into a Datum
	/// The value is taken from matches
	pub fn serde_matched_entry<T, I>(&mut self, name: &'static str, value: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + Equals<Datum> + Debug,
			I: Iterator<Item=T>
	{
		if self.serializing {
			self.serialize_entry(name, take(value));
			return Ok(());
		}
		self.deserialize_matched_entry(name, value, matches)
	}
	/// Either serializes or deserializes a named entry that can only be an item in matches
	/// Note that the value only needs to be present in matches during deserialization
	/// The data type of the field must be able to turn into a Datum
	/// The value is cloned from matches
	pub fn serde_cloned_matched_entry<'a, T, I>(&mut self, name: &'static str, value: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + Equals<Datum> + Debug + Clone + 'a,
			I: Iterator<Item=&'a T>
	{
		if self.serializing {
			self.serialize_entry(name, take(value));
			return Ok(());
		}
		self.deserialize_cloned_matched_entry(name, value, matches)
	}
}
