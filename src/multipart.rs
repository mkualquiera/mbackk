use std::{
    io::{self, Read, Write},
    path::PathBuf,
};

use log::debug;

/// A writer with a size limit per file. If it goes over the limit, it
/// moves to the next file.
pub struct MultipartWriter {
    /// The base path of the files.
    base_path: PathBuf,
    /// The current file number.
    file_number: u32,
    /// The current file.
    file: std::fs::File,
    /// The current file size.
    file_size: u64,
    /// The maximum file size.
    max_file_size: u64,
}

impl MultipartWriter {
    /// Create a new `MultipartWriter` from a base path and a maximum file size.
    pub fn new(base_path: PathBuf, max_file_size: u64) -> Self {
        // Ensure the base path exists.
        std::fs::create_dir_all(&base_path).unwrap();
        let file_number = 0;
        let file =
            std::fs::File::create(base_path.join(file_number.to_string()))
                .unwrap();
        let file_size = 0;
        debug!("** Writing to file {}", file_number);
        Self {
            base_path,
            file_number,
            file,
            file_size,
            max_file_size,
        }
    }
}

impl Write for MultipartWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = buf.len();
        let can_write = self.max_file_size - self.file_size;
        if can_write == 0 {
            self.file_number += 1;
            self.file = std::fs::File::create(
                self.base_path.join(self.file_number.to_string()),
            )?;
            self.file_size = 0;
            debug!("** Writing to file {}", self.file_number);
            return self.write(buf);
        }
        let size = std::cmp::min(size, can_write as usize);
        let buf = &buf[..size];
        self.file.write_all(buf)?;
        self.file_size += size as u64;
        Ok(size)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

/// A reader that reads from multiple files.
pub struct MultipartReader {
    /// The base path of the files.
    base_path: PathBuf,
    /// The current file number.
    file_number: u32,
    /// The current file.
    file: std::fs::File,
    /// The current file size.
    file_size: u64,
}

impl MultipartReader {
    /// Create a new `MultipartReader` from a base path and a maximum file size.
    pub fn new(base_path: PathBuf) -> Self {
        let file_number = 0;
        let file = std::fs::File::open(base_path.join(file_number.to_string()))
            .unwrap();
        let file_size = 0;
        debug!("** Reading from file {}", file_number);
        Self {
            base_path,
            file_number,
            file,
            file_size,
        }
    }
}

impl Read for MultipartReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size = self.file.read(buf)?;
        if size == 0 {
            self.file_number += 1;
            // See if the next file exists.
            let next_file = self.base_path.join(self.file_number.to_string());
            if !next_file.exists() {
                return Ok(0);
            }
            debug!("** Reading from file {}", self.file_number);
            self.file = std::fs::File::open(next_file)?;
            self.file_size = 0;
            return self.read(buf);
        }
        self.file_size += size as u64;
        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipart() {
        let base_path = std::env::temp_dir().join("test_multipart");
        std::fs::create_dir_all(&base_path).unwrap();
        let mut writer = MultipartWriter::new(base_path.clone(), 5);
        writer.write_all(b"hello").unwrap();
        writer.write_all(b"world").unwrap();
        writer.write_all(b"!").unwrap();
        writer.flush().unwrap();

        // Check that the files were created and their contents are correct.
        let file_0 = base_path.join("0");
        let file_1 = base_path.join("1");
        let file_2 = base_path.join("2");
        assert!(file_0.exists());
        assert!(file_1.exists());
        assert!(file_2.exists());

        let mut buf = String::new();
        std::fs::File::open(&file_0)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(buf, "hello");
        buf.clear();
        std::fs::File::open(&file_1)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(buf, "world");
        buf.clear();
        std::fs::File::open(&file_2)
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        assert_eq!(buf, "!");

        // Check that the files contain the correct data.
        let mut reader = MultipartReader::new(base_path);
        let mut buf = String::new();
        reader.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "helloworld!");
    }
}
