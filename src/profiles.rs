use std::mem::swap;
use std::ops::DerefMut;
pub use array::{DatumArray, ArrayData};
pub use map::{DatumMap, MappedData};

use crate::DeserializationError;

mod map;
mod array;


/// An internal representation of the agnostic data format.
/// This format can only be in serialization or deserialization mode, not both
#[derive(Debug)]
enum SerdeData<S, D: ?Sized> {
	Deserializing(Box<D>),
	Serializing(S),
}


/// A Trait that all data profiles should implement
pub trait DataProfile {
	/// Returns true iff the profile is being used to serialize data.
	/// Should always be false if the profile is created directly from serialized data
	fn is_serial(&self) -> bool;
	/// Instantiates a profile that is ready for serialization
	fn serial_ready() -> Self;
}


/// Allows a data profile to be instantiated from another type.
/// The data profile must be ready for deserialization
pub trait ProfileFromData<D>: Sized {
	/// Tries to create a data profile from the given data
	fn try_from(data: D) -> Result<Self, DeserializationError>;
}


/// Allows a data profile to convert into another type.
/// The data profile must be ready for serialization
pub trait ProfileToData<D> {
	/// Converts the data profile into another type.
	/// Implementors must ensure that this conversion will not fail
	fn into(self) -> D;
}


/// Creates a new data profile, from an existing profile, that can be applied onto your types.
/// All required functionality is automatically implemented
#[macro_export]
macro_rules! make_data_profile {
    ($(#[$attr:meta])* $name: ident use $base: ty) => {

// Define new type
$(#[$attr])*
#[derive(Debug)]
pub struct $name($base);


impl Deref for $name {
	type Target = $base;
	
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for $name {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl DataProfile for $name {
	fn is_serial(&self) -> bool {
		self.deref().is_serial()
	}
	fn serial_ready() -> Self {
		Self(<$base>::serial_ready())
	}
}

// Inherit all ProfileFromData traits from base
impl<D> ProfileFromData<D> for $name where $base: ProfileFromData<D> {
	fn try_from(data: D) -> Result<Self, DeserializationError> {
		Ok(Self(ProfileFromData::try_from(data)?))
	}
}

// Inherit all ProfileToData traits from base
impl<D> ProfileToData<D> for $name where $base: ProfileToData<D> {
	fn into(self) -> D {
		ProfileToData::into(self.0)
	}
}
    
    };
}


/// Converts the given data profile into another data profile.
/// Both types must use the same base data profile
///
/// This is niche function that can come in handy if you are handling many data profiles and you want to
/// change them all into the same data profile.
/// This can also be used to convert between sub-formats.
///
/// If you have two different styles for toml that you have to use,
/// you can create two data profiles for serializing to toml
/// and use this function to convert profiles when necessary
pub fn convert_data_profile<B, P, T>(mut src: T) -> P
	where
		P: DerefMut<Target=B> + DataProfile,
		T: DerefMut<Target=B>,
{
	let mut new = P::serial_ready();
	swap(src.deref_mut(), new.deref_mut());
	new
}