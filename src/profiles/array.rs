use std::fmt::Debug;
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
			Self::Deserializing(_) => panic!("Attempting to push string while deserializing!"),
			Self::Serializing(x) => x.push((item.into(), size))
		}
	}
	fn get_item<E, T>(&mut self) -> Result<T, DeserializationError>
		where
			E: Into<DeserializationError>,
			Datum: TryInto<T, Error=E>,
			T: GetDatumType
	{
		match self {
			Self::Serializing(_) => panic!("Attempting to push string while deserializing!"),
			Self::Deserializing(x) => x.get_datum(T::get_datum_type(), DatumSize::U32)?.try_into().map_err(Into::into)
		}
	}
}


pub trait DatumArray: Debug {
	fn get_datum(&mut self, datum_type: DatumType, datum_size: DatumSize) -> Result<Datum, DeserializationError>;
}


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
	pub fn into_serialized_items(self) -> IntoIter<(Datum, DatumSize)> {
		match self.data {
			SerdeArray::Serializing(x) => x.into_iter(),
			SerdeArray::Deserializing(_) => panic!("Attempted to iterate through items while deserializing")
		}
	}
	pub fn serialize_item<T: Into<Datum>>(&mut self, item: T) {
		self.data.push_item(item);
	}
	pub fn deserialize_item<T, E>(&mut self, into: &mut T) -> Result<(), DeserializationError>
		where
			E: Into<DeserializationError>,
			T: TryFrom<Datum, Error=E> + GetDatumType
	{
		*into = self.data.get_item()?;
		Ok(())
	}
	pub fn serde_item<T, E>(&mut self, into: &mut T) -> Result<(), DeserializationError>
		where
			E: Into<DeserializationError>,
			T: Into<Datum> + TryFrom<Datum, Error=E> + Default + GetDatumType
	{
		if self.is_serial() {
			self.serialize_item(take(into));
			return Ok(())
		}
		self.deserialize_item(into)
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
