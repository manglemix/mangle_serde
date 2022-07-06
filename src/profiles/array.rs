use std::fmt::{Debug, Write};
use std::mem::{take};
use std::vec::IntoIter;

use crate::{DataProfile, DeserializationError, ProfileFromData};
use crate::datum::{Datum, DatumSize, DatumType, GetDatumType};
use super::SerdeData;

type SerdeArray = SerdeData<Vec<(Datum, DatumSize)>, dyn DatumArray>;


impl SerdeArray {
	fn push_item<T: Into<Datum>>(&mut self, item: T) {
		self.push_item_sized(item, DatumSize::U32)
	}
	fn push_item_sized<T: Into<Datum>>(&mut self, item: T, size: DatumSize) {
		match self {
			Self::Deserializing(_) => panic!("Attempting to push item while deserializing!"),
			Self::Serializing(x) => x.push((item.into(), size))
		}
	}
	fn get_item<T: GetDatumType>(&mut self) -> Result<Datum, DeserializationError> {
		self.get_item_sized::<T>(DatumSize::U32)
	}
	fn get_item_sized<T: GetDatumType>(&mut self, size: DatumSize) -> Result<Datum, DeserializationError> {
		match self {
			Self::Serializing(_) => panic!("Attempting to get item while serializing!"),
			Self::Deserializing(x) => x.get_datum(T::get_datum_type(), size).map_err(Into::into)
		}
	}
}


pub trait DatumArray: Debug {
	fn get_datum(&mut self, datum_type: DatumType, datum_size: DatumSize) -> Result<Datum, DeserializationError>;
}


/// A base data profile for data that is stored in an array.
/// You will not instantiate this directly, but you will make aliases of this using the make_data_profile macro
#[derive(Debug)]
pub struct ArrayData {
	serializing: bool,
	data: SerdeArray,
}


impl DataProfile for ArrayData {
	fn is_serial(&self) -> bool {
		self.serializing
	}
	fn serial_ready() -> Self {
		Self {
			serializing: true,
			data: SerdeArray::Serializing(Vec::new()),
		}
	}
}


impl ArrayData {
	/// Consumes self and returns an iterator over data and their sizes
	#[must_use]
	pub fn into_serialized_items(self) -> IntoIter<(Datum, DatumSize)> {
		match self.data {
			SerdeArray::Serializing(x) => x.into_iter(),
			SerdeArray::Deserializing(_) => panic!("Attempted to iterate through items while deserializing")
		}
	}
	/// Serializes the given item that can turn into a Datum
	pub fn serialize_item<T: Into<Datum>>(&mut self, item: T) {
		self.data.push_item(item);
	}
	/// Deserializes some data and places it in into
	pub fn deserialize_item<T, E>(&mut self, into: &mut T) -> Result<(), DeserializationError>
		where
			T: TryFrom<Datum, Error=E> + GetDatumType,
			DeserializationError: From<E>
	{
		*into = TryFrom::try_from(self.data.get_item::<T>()?)?;
		Ok(())
	}
	pub fn deserialize_matched_item<T, I, Iter>(&mut self, into: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: GetDatumType + PartialEq<Datum>,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=T>
	{
		let item = self.data.get_item::<T>()?;

		for maybe_match in matches {
			if maybe_match.eq(&item) {
				*into = maybe_match;
				return Ok(());
			}
		}

		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch { field: "".into(), actual: debug_string })
	}
	pub fn deserialize_cloned_matched_item<'a, T, I, Iter>(&mut self, into: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: GetDatumType + PartialEq<Datum> + Clone + 'a,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=&'a T>
	{
		let item = self.data.get_item::<T>()?;

		for maybe_match in matches {
			if maybe_match.eq(&item) {
				*into = maybe_match.clone();
				return Ok(());
			}
		}

		let mut debug_string = String::new();
		let _ = writeln!(&mut debug_string, "{:?}", item);
		Err(DeserializationError::NoMatch { field: "".into(), actual: debug_string })
	}
	/// Serializes or deserializes, based on the current state
	pub fn serde_item<T, E>(&mut self, into: &mut T) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + TryFrom<Datum, Error=E> + Default + GetDatumType,
			DeserializationError: From<E>
	{
		if self.is_serial() {
			self.serialize_item(take(into));
			return Ok(())
		}
		self.deserialize_item(into)
	}
	pub fn serde_matched_item<T, I, Iter>(&mut self, into: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + GetDatumType + PartialEq<Datum>,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=T>
	{
		if self.is_serial() {
			self.serialize_item(take(into));
			return Ok(())
		}
		self.deserialize_matched_item(into, matches)
	}
	pub fn serde_cloned_matched_item<'a, T, I, Iter>(&mut self, into: &mut T, matches: I) -> Result<(), DeserializationError>
		where
			T: Into<Datum> + Default + GetDatumType + PartialEq<Datum> + Clone + 'a,
			I: IntoIterator<IntoIter=Iter>,
			Iter: Iterator<Item=&'a T>
	{
		if self.is_serial() {
			self.serialize_item(take(into));
			return Ok(())
		}
		self.deserialize_cloned_matched_item(into, matches)
	}
}


impl<D: DatumArray + 'static> ProfileFromData<D> for ArrayData {
	fn try_from(data: D) -> Result<Self, DeserializationError> {
		Ok(Self {
			serializing: false,
			data: SerdeArray::Deserializing(Box::new(data)),
		})
	}
}
