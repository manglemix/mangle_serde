use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use crate::profiles::{DatumMap, MappedData};

use super::*;

/// For checking equality against arbitrary types
/// Mainly used for comparing Datum to arbitrary types
pub trait Equals<T> {
	fn equals(&self, other: &T) -> bool;
}


/// An enum of common types that most structs are expected to contain
/// You will not find yourself needing to instantiate this yourself
#[derive(Debug)]
pub enum Datum {
	String(String),
	// Str(&'static str),
	U32(u32),
	U64(u64),
	Map(MappedData),
	Array(ArrayData)
}


impl Default for Datum {
	fn default() -> Self {
		Self::U32(0)
	}
}


impl PartialEq for Datum {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Datum::String(x) => if let Datum::String(o) = other {
											x == o
										} else { false }
			// Datum::Str(x) => 	if let Datum::Str(o) = other {
			// 								x == o
			// 							} else { false }
			Datum::U32(x) =>  if let Datum::U32(o) = other {
										x == o
									} else { false }
			Datum::U64(x) => 	if let Datum::U64(o) = other {
									x == o
								} else { false }
			Datum::Map(_) => unimplemented!("Cannot compare MappedData"),
			Datum::Array(_) => unimplemented!("Cannot compare ArrayData")
		}
	}
}


impl Eq for Datum {}


impl Clone for Datum {
	fn clone(&self) -> Self {
		match self {
			Datum::String(x) => Datum::String(x.clone()),
			// Datum::Str(x) => Datum::Str(x.clone()),
			Datum::U32(x) => Datum::U32(*x),
			Datum::U64(x) => Datum::U64(*x),
			Datum::Map(_) => unimplemented!("Cannot clone a MappedData"),
			Datum::Array(_) => unimplemented!("Cannot clone an ArrayData")
		}
	}
}


impl Hash for Datum {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Datum::String(x) => x.hash(state),
			// Datum::Str(x) => x.hash(state),
			Datum::U32(x) => x.hash(state),
			Datum::U64(x) => x.hash(state),
			Datum::Map(_) => unimplemented!("Cannot hash MappedData!"),
			Datum::Array(_) => unimplemented!("Cannot hash ArrayData!")
		}
	}
}


#[derive(Debug, Copy, Clone)]
pub enum DatumType {
	String,
	U32,
	U64,
	Map,
	Array
}


#[derive(Debug, Copy, Clone)]
pub enum DatumSize {
	U8,
	U16,
	U32,
	U64
}


impl DatumSize {
	pub fn into_byte_size(self) -> usize {
		match self {
			DatumSize::U8 => 1,
			DatumSize::U16 => 2,
			DatumSize::U32 => 4,
			DatumSize::U64 => 8
		}
	}

	pub fn serialize_usize(self, size: usize) -> Vec<u8> {
		match self {
			DatumSize::U8 => vec![size as u8],
			DatumSize::U16 => (size as u16).to_be_bytes().to_vec(),
			DatumSize::U32 => (size as u32).to_be_bytes().to_vec(),
			DatumSize::U64 => (size as u64).to_be_bytes().to_vec()
		}
	}
}


impl Datum {
	pub fn get_type(&self) -> DatumType {
		match self {
			Self::String(_) => DatumType::String,
			// Self::Str(_) => DatumType::Str,
			Self::U32(_) => DatumType::U32,
			Self::U64(_) => DatumType::U64,
			Self::Map(_) => DatumType::Map,
			Self::Array(_) => DatumType::Array
		}
	}
}


impl PartialEq<Datum> for &str {
	fn eq(&self, other: &Datum) -> bool {
		match other {
			// Datum::Str(s) => self == s,
			Datum::String(s) => self == s,
			_ => false
		}
	}
}


impl From<String> for Datum {
	fn from(s: String) -> Self {
		Self::String(s)
	}
}


impl From<&str> for Datum {
	fn from(s: &str) -> Self {
		Self::String(s.into())
	}
}


impl From<u64> for Datum {
	fn from(n: u64) -> Self {
		Self::U64(n)
	}
}


impl From<u32> for Datum {
	fn from(n: u32) -> Self {
		Self::U32(n)
	}
}


impl From<usize> for Datum {
	fn from(n: usize) -> Self {
		Self::U64(n as u64)
	}
}


impl<E, K, V> DatumMap for HashMap<K, V>
	where
		K: TryFrom<Datum, Error=E> + Debug + Eq + Hash,
		V: Into<Datum> + Debug,
		DeserializationError: From<E>
{
	fn get_datum(&mut self, key: &Datum) -> Result<Datum, DeserializationError> {
		match self.remove(&K::try_from(key.clone())?) {
			Some(x) => Ok(x.into()),
			None => Err(DeserializationError::MissingField(key.to_key_string()))
		}
	}
}


// impl<K: Into<Datum>, V: Into<Datum>> From<HashMap<K, V>> for Datum {
// 	fn from(map: HashMap<K, V>) -> Self {
// 		let data = MappedData::serial_ready();
// 		data.serialize_entry()
// 	}
// }


impl TryFrom<Datum> for usize {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::U64(n) => Ok(n as usize),
			_ => Err(DeserializationError::InvalidType { field: "".into(), expected: "u64", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for u32 {
	type Error = DeserializationError;

	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::U32(n) => Ok(n),
			_ => Err(DeserializationError::InvalidType { field: "".into(), expected: "u32", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for u64 {
	type Error = DeserializationError;

	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::U64(n) => Ok(n),
			_ => Err(DeserializationError::InvalidType { field: "".into(), expected: "u64", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for String {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::String(x) => Ok(x),
			_ => Err(DeserializationError::InvalidType { field: "".into(), expected: "String", actual: "todo!" })
		}
	}
}


pub trait GetDatumType {
	fn get_datum_type() -> DatumType;
}


impl GetDatumType for String {
	fn get_datum_type() -> DatumType {
		DatumType::String
	}
}


impl GetDatumType for &str {
	fn get_datum_type() -> DatumType {
		DatumType::String
	}
}


impl GetDatumType for u64 {
	fn get_datum_type() -> DatumType {
		DatumType::U64
	}
}


#[cfg(any(feature = "toml", feature = "json"))]
impl Datum {
	pub fn to_key_string(&self) -> String {
		match self {
			Datum::String(s) => s.clone(),
			// Datum::Str(s) => (*s).into(),
			Datum::U32(s) => s.to_string(),
			Datum::U64(s) => s.to_string(),
			Datum::Map(_) => unimplemented!("Cannot turn map into key string"),
			Datum::Array(_) => unimplemented!("Cannot turn array into key string")
		}
	}
}
