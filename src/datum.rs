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
			_ => Err(DeserializationError::InvalidType {field: "", expected: "u64", actual: "todo!"})
		}
	}
}


impl TryFrom<Datum> for String {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::String(x) => Ok(x),
			_ => Err(DeserializationError::InvalidType {field: "", expected: "String", actual: "todo!"})
		}
	}
}


impl TryFrom<Datum> for &'static str {
	type Error = DeserializationError;
	
	fn try_from(value: Datum) -> Result<Self, Self::Error> {
		match value {
			Datum::Str(x) => Ok(x),
			_ => Err(DeserializationError::InvalidType {field: "", expected: "static str", actual: "todo!"})
		}
	}
}