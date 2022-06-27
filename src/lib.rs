//! A convenience oriented crate for rapid implementation of serialization and deserialization.
//! that can be tailored to a variety of applications, while working around strict limitations
//!
//! The three main features offered by this crate is:
//! * Agnostic data formats that can be converted to different formats (such as TOML or binary) for free
//! * Symmetric coding for serialization and deserialization
//! * Support for custom serialization and deserialization profiles
//!
//! Often times, one would like to serialize their struct into readable TOML/JSON, while also being able to serialize to memory efficient binary.
//! Both formats have different requirements, so it is often best to have different serialization strategies for both situations.
//! This crate makes it easy to set up multiple different implementations of serialization and deserialization for the same struct.
//! So that you can satisfy all the requirements you need the way that you need to.
//!
//! On top of that, this crate allows you to write just one block of code that can do both serialization and deserialization.
//! without sacrificing any flexibility; You can still make specific code branches for serialization and deserialization.
//!
//! Best of all, the code you write is specific to the intended application, not the format.
//! You'll be describing the process of moving data from structs to agnostic data profiles.
//! All data profiles can convert to and from toml, json, or binary, for free.
#![deny(missing_docs, missing_debug_implementations, missing_crate_level_docs, missing_fragment_specifier, missing_copy_implementations)]
#![forbid(unsafe_code, unconditional_panic)]

#[cfg(feature = "toml")]
extern crate toml as extern_toml;

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::string::FromUtf8Error;

#[cfg(feature = "toml")]
use extern_toml::{de::Error as TOMLError};

use crate::profiles::DataProfile;
pub use crate::profiles::{ArrayData, MappedData, ProfileFromData, ProfileToData};

#[cfg(feature = "toml")]
mod toml;

mod datum;
mod profiles;

#[cfg(feature = "bin")]
mod binary;

/// An error that can occur when trying to deserialize data
#[derive(Clone, Debug)]
pub enum DeserializationError {
	/// An expected field could not be found
	/// Contains the field name
	MissingField(&'static str),
	/// An expected field has an unexpected data type
	InvalidType {
		/// The name of the field.
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
	/// An error creating a utf-8 string from binary
	FromUTF8Error(FromUtf8Error),
	/// The data we are deserializing from is too short
	UnexpectedEOF,
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
			Self::TOMLError(_) => return,
			DeserializationError::FromUTF8Error(_) => return,
			DeserializationError::UnexpectedEOF => return
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


impl From<FromUtf8Error> for DeserializationError {
	fn from(e: FromUtf8Error) -> Self {
		Self::FromUTF8Error(e)
	}
}


/// Serialization and Deserialization Trait.
/// Implement this on types that you wish to serialize or deserialize.
/// Takes in a DataProfile Alias as a type parameter
pub trait Serde<T: DataProfile>: Default {
	/// This function is called whenever serialization or deserialization is required.
	/// DataProfile offers multiple methods for symmetric coding to save you time here.
	/// However, DataProfile also allows you to check if the method is being called for serialization or deserialization
	fn serde(&mut self, data: &mut T) -> Result<(), DeserializationError>;

	/// Serializes self into a data profile.
	/// You will not need to call this directly.
	/// Consider using serialize
	fn into_data_profile(mut self) -> T {
		let mut data = T::serial_ready();
		let _ = self.serde(&mut data);
		data
	}

	/// Deserializes the given data profile into Self
	/// You will not need to call this directly.
	/// Consider using deserialize
	fn from_data_profile(mut data: T) -> Result<Self, DeserializationError> {
		let mut deser = Self::default();
		deser.serde(&mut data)?;
		Ok(deser)
	}

	/// Serialize to any type that can be constructed from a DataProfile.
	/// For now, that is only a toml::Value, json::Value, and Vec<u8>
	fn serialize<S>(self) -> S where T: ProfileToData<S> {
		self.into_data_profile().into()
	}
	
	/// Deserialize from an type that can transform into a DataProfile.
	/// For now, that is only a toml::Value, json::Value, and Vec\<u8\>
	fn deserialize<D>(data: D) -> Result<Self, DeserializationError> where T: ProfileFromData<D> {
		Self::from_data_profile(T::try_from(data)?)
	}
}


make_data_profile!(
	/// A data representation profile that should be used for dealing with serialized data that is human readable
	ReadableProfile use MappedData
);

make_data_profile!(
	/// A data representation profile that should be used for dealing with serialized data that is space efficient
	EfficientProfile use ArrayData
);


#[cfg(test)]
mod tests {
	use crate::toml::TOMLSerde;
	use crate::binary::BinSerde;
	use crate::profiles::{convert_data_profile};
	use super::*;

	make_data_profile!(
		///
		TestProfile use MappedData
	);

	#[derive(Default, Debug)]
	struct TestStruct {
		name: &'static str,
		age: u64,
		id: String,
	}
	
	impl Serde<ReadableProfile> for TestStruct {
		fn serde(&mut self, data: &mut ReadableProfile) -> Result<(), DeserializationError> {
			data.serde_cloned_matched_entry("name", &mut self.name, ["fergus", "ferus"].iter())?;
			data.serde_entry("age", &mut self.age)?;
			data.serde_entry("id", &mut self.id)
		}
	}

	impl Serde<EfficientProfile> for TestStruct {
		fn serde(&mut self, data: &mut EfficientProfile) -> Result<(), DeserializationError> {
			data.serde_item(&mut self.age)?;
			data.serde_item(&mut self.id)
		}
	}
	
	impl TOMLSerde<ReadableProfile> for TestStruct {}
	impl BinSerde<EfficientProfile> for TestStruct {}
	
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

	#[test]
	fn test_serde2() {
		let src = TestStruct {
			name: "ferus",
			age: 52,
			id: "gangnam".into(),
		};
		let ser = src.serialize_bin();
		println!("{:?}", ser);
		let deser = TestStruct::deserialize_bin(ser).unwrap();
		println!("{:?}", deser);
	}

	#[test]
	fn test_serde3() {
		let src = TestStruct {
			name: "ferus",
			age: 52,
			id: "gangnam".into(),
		};
		let test: TestProfile = convert_data_profile(Serde::<ReadableProfile>::into_data_profile(src));
		let ser = ProfileToData::into(test);
		println!("{}", ser);
		let deser: TestStruct = Serde::<ReadableProfile>::from_data_profile(ProfileFromData::try_from(ser).unwrap()).unwrap();
		println!("{:?}", deser);
	}
}