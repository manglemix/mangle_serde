use crate::{ArrayData, DataProfile, DeserializationError, ProfileFromData, ProfileToData, Serde};
use crate::datum::{Datum, DatumSize, DatumType};
use crate::profiles::{DatumArray};


fn split(arr: &mut Vec<u8>, size: usize) -> Option<Vec<u8>> {
	if arr.len() < size {
		return None
	}
	let (first, last) = arr.split_at(size);
	let first = first.to_vec();
	*arr = last.to_vec();
	Some(first)
}


fn split_arr<const SIZE: usize>(arr: &mut Vec<u8>) -> Option<[u8; SIZE]> {
	if arr.len() < SIZE {
		return None
	}
	let (first, last) = arr.split_at(SIZE);
	let first = first.try_into().unwrap();
	*arr = last.to_vec();
	Some(first)
}


fn get_size(arr: &mut Vec<u8>, datum_size: DatumSize) -> Option<usize> {
	if arr.is_empty() { return None }
	Some(match datum_size {
		DatumSize::U8 => arr.remove(0) as usize,
		DatumSize::U16 => u16::from_be_bytes(split_arr::<2>(arr)?) as usize,
		DatumSize::U32 => u32::from_be_bytes(split_arr::<4>(arr)?) as usize,
		DatumSize::U64 => u64::from_be_bytes(split_arr::<8>(arr)?) as usize,
	})
}


impl DatumArray for Vec<u8> {
	fn get_datum(&mut self, datum_type: DatumType, datum_size: DatumSize) -> Result<Datum, DeserializationError> {
		Ok(match datum_type {
			DatumType::String => {
				let size = get_size(self, datum_size).ok_or(DeserializationError::UnexpectedEOF)?;
				Datum::from(String::from_utf8(
					split(self, size).ok_or(DeserializationError::UnexpectedEOF)?
				)?)
			},
			DatumType::U64 => Datum::from(u64::from_be_bytes(split_arr::<8>(self).ok_or(DeserializationError::UnexpectedEOF)?)),
			_ => todo!()
		})
	}
}


impl From<Datum> for Vec<u8> {
	fn from(data: Datum) -> Self {
		match data {
			Datum::String(x) => x.into_bytes(),
			// Datum::Str(x) => String::from(x).into_bytes(),
			Datum::U64(x) => x.to_be_bytes().to_vec(),
			Datum::U32(x) => x.to_be_bytes().to_vec(),
			Datum::Map(_) => todo!(),
			Datum::Array(x) => ProfileToData::into(x)
		}
	}
}


impl ProfileToData<Vec<u8>> for ArrayData {
	fn into(self) -> Vec<u8> {
		let mut out = Vec::new();
		for (datum, datum_size) in self.into_serialized_items() {
			match datum.get_type() {
				DatumType::String => {
					let mut bytes: Vec<u8> = datum.into();
					out.append(&mut datum_size.serialize_usize(bytes.len()));
					out.append(&mut bytes);
				}
				_ => out.append(&mut datum.into()),
			}
		}
		out
	}
}


/// Adds explicit methods for converting to and from binary using a given data profile.
/// Can only be implemented on types that implement Serde with the same data profile
pub trait BinSerde<T: DataProfile + ProfileToData<Vec<u8>> + ProfileFromData<Vec<u8>>>: Serde<T> {
	/// Serializes self into binary
	fn serialize_bin(self) -> Vec<u8> {
		self.serialize()
	}
	/// Deserializes a binary vector into Self.
	/// Returns an error if the string could not be deserialized
	fn deserialize_bin(data: Vec<u8>) -> Result<Self, DeserializationError> {
		Self::deserialize(data)
	}
}
