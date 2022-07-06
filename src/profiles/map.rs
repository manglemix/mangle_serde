use std::collections::hash_map::IntoIter;
use std::collections::HashMap;
use std::fmt::{Debug, Write};
use std::mem::{take};

use crate::{DeserializationError, ProfileFromData, TransformResult};
use crate::datum::{Datum};
use crate::profiles::SerdeData;

use super::DataProfile;

pub trait DatumMap: Debug {
	fn get_datum(&mut self, key: &Datum) -> Result<Datum, DeserializationError>;
}

type SerdeMap = SerdeData<HashMap<Datum, Datum>, dyn DatumMap>;


impl SerdeMap {
	fn set(&mut self, key: Datum, value: Datum) {
		match self {
			Self::Deserializing(_) => panic!("Attempted to set while deserializing! Please report this to the developer"),
			Self::Serializing(x) => x.insert(key, value)
		};
	}
	
	fn get(&mut self, key: &Datum) -> Result<Datum, DeserializationError> {
		match self {
			Self::Deserializing(x) => x.get_datum(key),
			Self::Serializing(_) => panic!("Attempted to get while serializing! Please report this to the developer")
		}
	}
}


/// A base data profile for data that is stored in a map, with keys and values.
/// Keys are always static strings.
/// You will not instantiate this directly, but you will make aliases of this using the make_data_profile macro
#[derive(Debug)]
pub struct MappedData {
	serializing: bool,
	data: SerdeMap,
}


impl DataProfile for MappedData {
	fn is_serial(&self) -> bool {
		self.serializing
	}
	fn serial_ready() -> Self {
		Self {
			serializing: true,
			data: SerdeMap::Serializing(HashMap::new()),
		}
	}
}


impl MappedData {
	/// Converts this data profile into an iterator over serialized entries
	///
	/// # panic
	/// Panics if this data profile is not in serialization mode
	#[must_use]
	pub fn into_serialized_entries(self) -> IntoIter<Datum, Datum> {
		match self.data {
			SerdeMap::Deserializing(_) => panic!("Attempted to iterate through entries while deserializing"),
			SerdeMap::Serializing(x) => x.into_iter()
		}
	}
	/// Serialize a named value as an entry.
	/// The value must be able to turn into a Datum by implementing Into<Datum>
	pub fn serialize_entry<K: Into<Datum>, V: Into<Datum>>(&mut self, name: K, value: V) {
		self.data.set(name.into(), value.into());
	}
	/// Deserialize a named entry
	/// The value must be able to come from a Datum by implementing TryFrom<Datum>
	pub fn deserialize_entry<K, V, E>(&mut self, name: K, into: &mut V) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: TryFrom<Datum, Error=E> + Default,
			DeserializationError: From<E>
	{
		let name_ref = name.into();
		*into = self.data.get(&name_ref)?.try_into().transform(name_ref.to_key_string())?;
		Ok(())
	}
	/// Deserialize a named entry that is one of the given matches
	/// The value will be taken from matches
	pub fn deserialize_matched_entry<K, V, I, Iter>(&mut self, name: K, into: &mut V, matches: I) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: PartialEq<Datum>,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=V>
	{
		let name_ref = name.into();
		let item = self.data.get(&name_ref)?;
		
		for maybe_match in matches {
			if maybe_match.eq(&item) {
				*into = maybe_match;
				return Ok(());
			}
		}
		
		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch { field: name_ref.to_key_string(), actual: debug_string })
	}
	/// Deserialize a named entry that is one of the given matches.
	/// The value will be cloned from matches
	pub fn deserialize_cloned_matched_entry<'a, K, V, I, Iter>(&mut self, name: K, into: &mut V, matches: I) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: Clone + PartialEq<Datum> + 'a,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=&'a V>
	{
		let name_ref = name.into();
		let item = self.data.get(&name_ref)?;
		
		for maybe_match in matches {
			if maybe_match.eq(&item) {
				*into = maybe_match.clone();
				return Ok(());
			}
		}
		
		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch { field: name_ref.to_key_string(), actual: debug_string })
	}
	/// Either serializes or deserializes a named entry.
	/// The data type of the field must be able to convert to or from a Datum
	pub fn serde_entry<K, V, E>(&mut self, name: K, value: &mut V) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: Into<Datum> + TryFrom<Datum, Error=E> + Default,
			DeserializationError: From<E>
	{
		if self.serializing {
			self.serialize_entry(name.into(), take(value));
			return Ok(());
		}
		self.deserialize_entry(name, value).map_err(Into::into)
	}
	/// Either serializes or deserializes a named entry that can only be an item in matches.
	/// Note that the value only needs to be present in matches during deserialization.
	/// The data type of the field must be able to turn into a Datum.
	/// The value is taken from matches
	pub fn serde_matched_entry<K, V, I, Iter>(&mut self, name: V, value: &mut V, matches: I) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: Into<Datum> + PartialEq<Datum> + Default,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=V>
	{
		if self.serializing {
			self.serialize_entry(name.into(), take(value));
			return Ok(());
		}
		self.deserialize_matched_entry(name, value, matches)
	}
	/// Either serializes or deserializes a named entry that can only be an item in matches.
	/// Note that the value only needs to be present in matches during deserialization.
	/// The data type of the field must be able to turn into a Datum.
	/// The value is cloned from matches
	pub fn serde_cloned_matched_entry<'a, K, V, I, Iter>(&mut self, name: K, value: &mut V, matches: I) -> Result<(), DeserializationError>
		where
			K: Into<Datum>,
			V: Into<Datum> + PartialEq<Datum> + Default + Clone + 'a,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=&'a V>
	{
		if self.serializing {
			self.serialize_entry(name, take(value));
			return Ok(());
		}
		self.deserialize_cloned_matched_entry(name, value, matches)
	}
}


impl<D: DatumMap + 'static> ProfileFromData<D> for MappedData {
	fn try_from(data: D) -> Result<Self, DeserializationError> {
		Ok(Self {
			serializing: false,
			data: SerdeMap::Deserializing(Box::new(data)),
		})
	}
}
