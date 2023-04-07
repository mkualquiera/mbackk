use std::io::{BufReader, Read, Write};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};

/// A wrapped reader that can either read a struct or a number of bytes.
pub struct StructReader<T: Read>(BufReader<T>);

impl<T: Read> StructReader<T> {
    /// Create a new `StructReader` from a reader.
    pub fn new(reader: T) -> Self {
        Self(BufReader::new(reader))
    }

    /// Read a struct from the reader.
    pub fn next<S: for<'a> Deserialize<'a>>(
        &mut self,
    ) -> Result<S, Box<dyn std::error::Error>> {
        let StructReader(reader) = self;
        let s_size = reader.read_u32::<LittleEndian>()?;
        let mut s_bytes = vec![0; s_size as usize];

        reader.read_exact(&mut s_bytes)?;

        let s: S = bincode::deserialize(&s_bytes)?;
        Ok(s)
    }

    /// Read some bytes from the reader.
    pub fn next_bytes(
        &mut self,
        size: usize,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let StructReader(reader) = self;
        let mut bytes = vec![0; size];

        reader.read_exact(&mut bytes)?;
        Ok(bytes)
    }
}

/// A wrapped writer that can either write a struct or a number of bytes.
pub struct StructWriter<T: Write + WriteBytesExt>(T);

impl<T: Write + WriteBytesExt> StructWriter<T> {
    /// Create a new `StructWriter` from a writer.
    pub fn new(writer: T) -> Self {
        Self(writer)
    }

    /// Write a struct to the writer.
    pub fn write<S: Serialize>(
        &mut self,
        s: &S,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let StructWriter(writer) = self;
        let s_bytes = bincode::serialize(s)?;
        let s_size = s_bytes.len() as u32;

        writer.write_u32::<LittleEndian>(s_size)?;
        writer.write_all(&s_bytes)?;
        Ok(())
    }

    /// Write some bytes to the writer.
    pub fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let StructWriter(writer) = self;
        writer.write_all(bytes)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Seek;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        name: String,
        a: u32,
        b: u32,
    }

    #[test]
    fn test_struct() {
        // write to a temp file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = StructWriter::new(file.as_file_mut());

        let s1 = TestStruct {
            name: "s1".to_string(),
            a: 1,
            b: 2,
        };

        writer.write(&s1).unwrap();

        file.rewind().unwrap();

        // read from the temp file
        let mut reader = StructReader::new(file.as_file_mut());

        let s2 = reader.next::<TestStruct>().unwrap();

        assert_eq!(s1, s2);
    }

    #[test]
    fn test_bytes() {
        // write to a temp file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = StructWriter::new(file.as_file_mut());

        let bytes = vec![1, 2, 3, 4, 5];

        writer.write_bytes(&bytes).unwrap();

        file.rewind().unwrap();

        // read from the temp file
        let mut reader = StructReader::new(file.as_file_mut());

        let bytes2 = reader.next_bytes(5).unwrap();

        assert_eq!(bytes, bytes2);
    }

    #[test]
    fn test_mix() {
        // write to a temp file
        let mut file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = StructWriter::new(file.as_file_mut());

        let s1 = TestStruct {
            name: "s1".to_string(),
            a: 1,
            b: 2,
        };

        writer.write(&s1).unwrap();

        let bytes = vec![1, 2, 3, 4, 5];

        writer.write_bytes(&bytes).unwrap();

        file.rewind().unwrap();

        // read from the temp file
        let mut reader = StructReader::new(file.as_file_mut());

        let s2 = reader.next::<TestStruct>().unwrap();

        assert_eq!(s1, s2);

        let bytes2 = reader.next_bytes(5).unwrap();

        assert_eq!(bytes, bytes2);
    }
}
