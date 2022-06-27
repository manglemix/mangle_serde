use std::borrow::Borrow;
use extern_toml::{de::Error, Value, value::Table};
use extern_toml::value::Array;

use crate::datum::Datum;
use crate::{ArrayData, DataProfile, DeserializationError, Serde};
use crate::profiles::{DatumMap, MappedData};
use crate::profiles::{ProfileFromData, ProfileToData};

impl DatumMap for Table {
	fn get_datum(&mut self, key: &'static str) -> Result<Datum, DeserializationError> {
		match self.remove(key) {
			None => Err(DeserializationError::MissingField(key)),
			Some(x) => Datum::try_from(x)
		}
	}
}


impl From<Error> for DeserializationError {
	fn from(e: Error) -> Self {
		DeserializationError::TOMLError(e)
	}
}


impl ProfileToData<Value> for ArrayData {
	fn into(self) -> Value {
		let mut array = Array::new();
		
		for (item, _) in self.into_serialized_items() {
			array.push(item.into());
		}
		
		Value::Array(array)
	}
}


impl ProfileToData<Value> for MappedData {
	fn into(self) -> Value {
		let mut table = Table::new();
		
		for (name, value) in self.into_serialized_entries() {
			table.insert(name.into(), value.into());
		}
		
		Value::Table(table)
	}
}


impl ProfileFromData<Value> for MappedData {
	fn try_from(data: Value) -> Result<Self, DeserializationError> {
		let table = match data {
			Value::Table(x) => x,
			_ => return Err(DeserializationError::InvalidType { field: "<global>", expected: "table", actual: "todo!" })
		};
		
		ProfileFromData::try_from(table)
	}
}


impl TryFrom<Value> for Datum {
	type Error = DeserializationError;
	
	fn try_from(value: Value) -> Result<Self, Self::Error> {
		if value.is_table() {
			return Ok(Self::Map(ProfileFromData::try_from(value)?));
		}
		
		Ok(match value {
			Value::String(s) => Self::String(s),
			Value::Integer(n) => Self::U64(n as u64),
			Value::Table(_) => unreachable!(),
			_ => todo!()
		})
	}
}


impl From<Datum> for Value {
	fn from(datum: Datum) -> Self {
		match datum {
			Datum::U64(n) => (n as i64).into(),
			Datum::String(s) => s.into(),
			Datum::Map(map) => ProfileToData::into(map),
			Datum::Str(s) => s.into(),
			Datum::Array(arr) => ProfileToData::into(arr)
		}
	}
}


pub trait TOMLSerde<T: DataProfile + ProfileToData<Value> + ProfileFromData<Value>>: Serde<T> {
	/// Serializes self into a TOML formatted string
	///
	/// # Panics
	/// This method should not panic. If it does, please report the error to the developer
	fn serialize_toml(self) -> String {
		extern_toml::to_string(&self.serialize::<Value>()).expect("An error occurred during TOML Serialization. Please report this to the developer")
	}
	/// Deserializes a string type into Self.
	/// Returns an error if the string could not be deserialized
	fn deserialize_toml<S: Borrow<str>>(data: S) -> Result<Self, DeserializationError> {
		Self::deserialize::<Value>(data.borrow().parse()?)
	}
}
