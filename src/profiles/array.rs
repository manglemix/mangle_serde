use std::io::{Read, Write};
use crate::DataProfile;


#[derive(Debug)]
pub(crate) enum DataStream<RW: Read + Write + Default> {
	Reader(RW),
	Writer(RW),
}


impl<RW: Read + Write + Default> Read for DataStream<RW> {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
		match self {
			Self::Reader(reader) => reader.read(buf),
			Self::Writer(_) => panic!("Attempted to read from a writer!")
		}
	}
}


impl<RW: Read + Write + Default> Write for DataStream<RW> {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		match self {
			Self::Writer(writer) => writer.write(buf),
			Self::Reader(_) => panic!("Attempted to write to a reader!")
		}
	}
	
	fn flush(&mut self) -> std::io::Result<()> {
		match self {
			Self::Writer(writer) => writer.flush(),
			Self::Reader(_) => panic!("Attempted to flush a reader!")
		}
	}
}


#[derive(Debug)]
pub struct ArrayData<RW: Read + Write + Default>  {
	pub(crate) serializing: bool,
	pub(crate) data: DataStream<RW>
}


impl<RW: Read + Write + Default> DataProfile for ArrayData<RW> {
	fn is_serial(&self) -> bool {
		self.serializing
	}
	fn serial_ready() -> Self {
		Self {
			serializing: true,
			data: DataStream::Writer(RW::default()),
		}
	}
}