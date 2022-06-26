use extern_toml::{de::Error, Value, value::Table};
pub(crate) use extern_toml::to_string as to_toml_string;

use crate::datum::Datum;
use crate::DeserializationError;
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
			Datum::Map(d) => ProfileToData::into(d),
			Datum::Str(s) => s.into()
		}
	}
}


/// Adds helper functions for serializing and deserializing TOML formatted strings
#[cfg(feature = "toml")]
#[macro_export]
macro_rules! impl_toml_serde {
    ($name: ident) => {
		impl $name {
			fn serialize_toml(self) -> String {
				toml::to_toml_string(&self.serialize::<TOMLValue>()).expect("An error occurred during TOML Serialization. Please report this to the developer")
			}
			fn deserialize_toml<T: Borrow<str>>(data: T) -> Result<Self, DeserializationError> {
				Self::deserialize::<TOMLValue>(data.borrow().parse()?)
			}
		}
	};
}
