//! A convenience oriented crate for rapid implementation of serialization and deserialization
//! that can be tailored to a variety of applications, while working around strict limitations
//!
//! The three main features offered by this crate is:
//! * Agnostic data representation format that can be converted to different formats (such as TOML or binary) for free
//! * Symmetric coding for serialization and deserialization
//! * Custom serialization and deserialization profiles
//!
//! Often times, one would like to serialize their struct into readable TOML/JSON, while also being able to serialize to memory efficient binary
//! Both formats have different requirements, so it is often best to have different serialization strategies for both situations
//! This crate makes it easy to set up multiple different implementations of serialization and deserialization for the same struct
//! So that you can satisfy all the requirements you need the way that you need to
//!
//! On top of that, this crate allows you to write just one block of code that can do both serialization and deserialization
//! without sacrificing any flexibility; You can still make specific code branches for serialization and deserialization
//!
//! Best of all, the code you write is specific to the intended application, not the format
//! You'll be describing the process of moving data from structs to my agnostic data representation
//! This data representation will transform itself to toml, json, or binary, without any help
#![deny(missing_docs, missing_debug_implementations, missing_crate_level_docs, missing_fragment_specifier, missing_copy_implementations)]

#[cfg(feature = "toml")]
extern crate toml as extern_toml;

#[forbid(unsafe_code, unconditional_panic)]
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

#[cfg(feature = "toml")]
use extern_toml::{de::Error as TOMLError, Value as TOMLValue};

pub use crate::profiles::{ArrayData, DataProfile, MappedData, ProfileFromData, ProfileToData};

#[cfg(feature = "toml")]
mod toml;

mod datum;
mod profiles;

/// An error that can occur when trying to deserialize data
#[derive(Clone, Debug)]
pub enum DeserializationError {
	/// An expected field could not be found
	/// Contains the field name
	MissingField(&'static str),
	/// An expected field has an unexpected data type
	InvalidType {
		/// The name of the field
		/// The name "\<global\>" implies that the entire serialized data is in the wrong format
		field: &'static str,
		/// The expected type of the field
		expected: &'static str,
		/// The actual type of the field
		actual: &'static str,
	},
	/// An expected field does not contain any of the expected data
	NoMatch {
		/// The name of the field
		field: &'static str,
		/// The actual data contained in the field
		actual: String,
	},
	#[cfg(feature = "toml")]
	/// An error occurred while parsing TOML formatted data
	TOMLError(TOMLError),
}


impl DeserializationError {
	/// Sets the field parameter if possible.
	/// This is to help with debugging
	fn set_field(&mut self, new_field: &'static str) {
		*match self {
			Self::MissingField(_) => return,
			Self::InvalidType { field, .. } => field,
			Self::NoMatch { field, .. } => field,
			Self::TOMLError(_) => return
		} = new_field;
	}
}


/// Used to convert a result into a deserialization result
trait TransformResult<T> {
	/// Converts the result into a deserialization result.
	/// Sets some fields in deserialization error if possible
	fn transform(self, new_field: &'static str) -> Result<T, DeserializationError>;
}


impl<T, E: Into<DeserializationError>> TransformResult<T> for Result<T, E> {
	fn transform(self, new_field: &'static str) -> Result<T, DeserializationError> {
		match self {
			Ok(x) => Ok(x),
			Err(e) => {
				let mut e = e.into();
				e.set_field(new_field);
				Err(e)
			}
		}
	}
}


/// Serialization and Deserialization Trait.
/// Implement this on types that you wish to serialize or deserialize.
/// Takes in a DataRepresentation Alias as a type parameter
pub trait Serde<T: DataProfile>: Default {
	/// This function is called whenever serialization or deserialization is required.
	/// DataRepresentation offers multiple methods for symmetric coding to save you time here.
	/// However, DataRepresentation also allows you to check if the method is being called for serialization or deserialization
	fn serde(&mut self, data: &mut T) -> Result<(), DeserializationError>;
	
	/// Serialize to any type that can be constructed from a DataRepresentation.
	/// For now, that is only a toml::Value, json::Value, and Vec<u8>
	fn serialize<S>(mut self) -> S where T: ProfileToData<S> {
		let mut data = T::serial_ready();
		let _ = self.serde(&mut data);
		data.into()
	}
	
	/// Deserialize from an type that can transform into a DataRepresentation.
	/// For now, that is only a toml::Value, json::Value, and Vec<u8>
	fn deserialize<D>(data: D) -> Result<Self, DeserializationError> where T: ProfileFromData<D> {
		let mut deser = Self::default();
		let mut data = T::try_from(data)?;
		deser.serde(&mut data)?;
		Ok(deser)
	}
}


make_data_profile!(
	/// A data representation profile that should be used for dealing with serialized data that is human readable
	ReadableProfile use MappedData
);

// make_data_profile!(
// 	/// A data representation profile that should be used for dealing with serialized data that is space efficient
// 	EfficientProfile use ArrayData<Vec<u8>>
// );


#[cfg(test)]
mod tests {
	use super::*;
	
	#[derive(Default, Debug)]
	struct TestStruct {
		name: &'static str,
		age: usize,
		id: String,
	}
	
	impl Serde<ReadableProfile> for TestStruct {
		fn serde(&mut self, data: &mut ReadableProfile) -> Result<(), DeserializationError> {
			data.serde_cloned_matched_entry("name", &mut self.name, ["fergus", "ferus"].iter())?;
			data.serde_entry("age", &mut self.age)?;
			data.serde_entry("id", &mut self.id)
		}
	}
	
	impl_toml_serde!(TestStruct);
	
	#[test]
	fn test_serde() {
		let src = TestStruct {
			name: "ferus",
			age: 52,
			id: "gangnam".into(),
		};
		let ser = src.serialize_toml();
		println!("{}", ser);
		let deser = TestStruct::deserialize_toml(ser).unwrap();
		println!("{:?}", deser);
	}
}