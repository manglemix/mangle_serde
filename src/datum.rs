use crate::profiles::MappedData;

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
	Str(&'static str),
	U64(u64),
	Map(MappedData),
	Array(ArrayData)
}


#[derive(Debug, Copy, Clone)]
pub enum DatumType {
	String,
	Str,
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
			Self::Str(_) => DatumType::Str,
			Self::U64(_) => DatumType::U64,
			Self::Map(_) => DatumType::Map,
			Self::Array(_) => DatumType::Array
		}
	}
}


impl Equals<Datum> for &'static str {
	fn equals(&self, other: &Datum) -> bool {
		match other {
			Datum::Str(s) => self == s,
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


impl From<&'static str> for Datum {
	fn from(s: &'static str) -> Self {
		Self::Str(s)
	}
}


impl From<u64> for Datum {
	fn from(n: u64) -> Self {
		Self::U64(n)
	}
}


impl From<usize> for Datum {
	fn from(n: usize) -> Self {
		Self::U64(n as u64)
	}
}


impl TryFrom<Datum> for usize {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::U64(n) => Ok(n as usize),
			_ => Err(DeserializationError::InvalidType { field: "", expected: "u64", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for u64 {
	type Error = DeserializationError;

	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::U64(n) => Ok(n),
			_ => Err(DeserializationError::InvalidType { field: "", expected: "u64", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for String {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::String(x) => Ok(x),
			_ => Err(DeserializationError::InvalidType { field: "", expected: "String", actual: "todo!" })
		}
	}
}


impl TryFrom<Datum> for &'static str {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::Str(x) => Ok(x),
			_ => Err(DeserializationError::InvalidType { field: "", expected: "static str", actual: "todo!" })
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


impl GetDatumType for &'static str {
	fn get_datum_type() -> DatumType {
		DatumType::Str
	}
}


impl GetDatumType for u64 {
	fn get_datum_type() -> DatumType {
		DatumType::U64
	}
}
