use std::borrow::Borrow;
use extern_json::{Error, JsonValue as Value, object::Object as Table, Array};

use crate::datum::Datum;
use crate::{ArrayData, DataProfile, DeserializationError, Serde};
use crate::profiles::{DatumMap, MappedData};
use crate::profiles::{ProfileFromData, ProfileToData};

impl DatumMap for Table {
	fn get_datum(&mut self, key: &Datum) -> Result<Datum, DeserializationError> {
		let key_string = key.to_key_string();
		match self.remove(key_string.as_str()) {
			None => Err(DeserializationError::MissingField(key_string)),
			Some(x) => Datum::try_from(x)
		}
	}
}


impl From<Error> for DeserializationError {
	fn from(e: Error) -> Self {
		DeserializationError::JSONError(e)
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
			table.insert(name.to_key_string().as_str(), value.into());
		}
		
		Value::Object(table)
	}
}


impl ProfileFromData<Value> for MappedData {
	fn try_from(data: Value) -> Result<Self, DeserializationError> {
		let table = match data {
			Value::Object(x) => x,
			_ => return Err(DeserializationError::InvalidType { field: "<global>".into(), expected: "table", actual: "todo!" })
		};
		
		ProfileFromData::try_from(table)
	}
}


impl TryFrom<Value> for Datum {
	type Error = DeserializationError;
	
	fn try_from(value: Value) -> Result<Self, Self::Error> {
		if value.is_object() {
			return Ok(Self::Map(ProfileFromData::try_from(value)?));
		}

		if value.is_number() {
			return Ok(Self::U64(
				value.as_u64().ok_or(DeserializationError::InvalidType {
					field: "".into(),
					expected: "integer",
					actual: "nan"
				})?
			));
		}

		Ok(match value {
			Value::String(s) => Self::String(s),
			Value::Short(s) => Self::String(s.into()),
			Value::Number(_) => unreachable!(),
			Value::Object(_) => unreachable!(),
			x => todo!("{:?}", x)
		})
	}
}


impl From<Datum> for Value {
	fn from(datum: Datum) -> Self {
		match datum {
			Datum::U64(n) => (n as i64).into(),
			Datum::U32(n) => (n as i64).into(),
			Datum::String(s) => s.into(),
			Datum::Map(map) => ProfileToData::into(map),
			// Datum::Str(s) => s.into(),
			Datum::Array(arr) => ProfileToData::into(arr)
		}
	}
}


pub trait JSONSerde<T: DataProfile + ProfileToData<Value> + ProfileFromData<Value>>: Serde<T> {
	const TAB_SIZE: u16;
	/// Serializes self into a JSON formatted string
	fn serialize_json(self) -> String {
		extern_json::stringify(self.serialize::<Value>())
	}
	fn serialize_json_pretty(self) -> String {
		extern_json::stringify_pretty(self.serialize::<Value>(), Self::TAB_SIZE)
	}
	/// Deserializes a string type into Self.
	/// Returns an error if the string could not be deserialized
	fn deserialize_json<S: Borrow<str>>(data: S) -> Result<Self, DeserializationError> {
		Self::deserialize::<Value>(extern_json::parse(data.borrow())?)
	}
}
