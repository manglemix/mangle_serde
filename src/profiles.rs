pub use array::ArrayData;
pub use map::{DatumMap, MappedData};

use crate::DeserializationError;

mod map;
mod array;


#[derive(Debug)]
enum SerdeData<S: 'static, D: ?Sized> {
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
	fn try_from(data: D) -> Result<Self, DeserializationError>;
}


pub trait ProfileToData<D> {
	fn into(self) -> D;
}


/// Creates a new data profile, from an existing profile, that can be applied onto your types.
/// All required functionality is automatically implemented
#[macro_export]
macro_rules! make_data_profile {
	($name: ident use $base: ty) => {
		make_data_profile!(
			///
			=> $name use $base
		)
	};
    ($(#[$attr:meta])* $name: ident use $base: ty) => {

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

impl<D> ProfileFromData<D> for $name where $base: ProfileFromData<D> {
	fn try_from(data: D) -> Result<Self, DeserializationError> {
		Ok(Self(ProfileFromData::try_from(data)?))
	}
}

impl<D> ProfileToData<D> for $name where $base: ProfileToData<D> {
	fn into(self) -> D {
		ProfileToData::into(self.0)
	}
}
    
    };
}