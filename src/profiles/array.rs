use std::fmt::Debug;
use std::vec::IntoIter;

use crate::{DataProfile, DeserializationError};
use crate::datum::Datum;
use super::SerdeData;

type SerdeArray = SerdeData<Vec<Datum>, dyn DatumArray>;


pub trait DatumArray: Debug {
	fn get_datum(&mut self, size: usize) -> Result<Datum, DeserializationError>;
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
	pub fn into_serialized_items(self) -> IntoIter<Datum> {
		match self.data {
			SerdeArray::Serializing(x) => x.into_iter(),
			SerdeArray::Deserializing(_) => panic!("Attempted to iterate through items while deserializing")
		}
	}
}
