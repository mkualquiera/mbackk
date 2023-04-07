use log::debug;
use serde::{Deserialize, Serialize};

use crate::struct_stream::{StructReader, StructWriter};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
enum StorageOperation {
    /// Entered a directory. The argument is the name of the directory.
    EnterDirectory(String),
    /// Left a directory.
    LeaveDirectory,
    /// Created a file. The argument is the name of the file and the file size.
    CreateFile(String, u64),
}

/// Struct that writes directories or files to a Write implementation.
pub struct StorageWriter<W: Write>(StructWriter<W>);

impl<W: Write> StorageWriter<W> {
    /// Create a new `StorageWriter` from a writer.
    pub fn new(writer: W) -> Self {
        Self(StructWriter::new(writer))
    }

    /// Write a file to the writer.
    pub fn write_file(
        &mut self,
        file: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let StorageWriter(writer) = self;
        let file_name = file.file_name().unwrap().to_str().unwrap().to_string();
        let file = File::open(file)?;
        let file_size = file.metadata()?.len();
        debug!("** Writing file {} ({} bytes)", file_name, file_size);
        let operation = StorageOperation::CreateFile(file_name, file_size);
        writer.write(&operation)?;
        // Read by chunks of 1MB
        let mut buf = [0; 1024 * 1024];
        let mut file = file;
        loop {
            let bytes_read = file.read(&mut buf)?;
            if bytes_read == 0 {
                break;
            }
            writer.write_bytes(&buf[..bytes_read])?;
        }
        Ok(())
    }

    /// Write a directory to the writer.
    /// The directory is read recursively.
    pub fn write_directory(
        &mut self,
        dir: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let dir_name = dir.file_name().unwrap().to_str().unwrap().to_string();
        debug!("-> Enter directory {}", dir_name);
        let operation = StorageOperation::EnterDirectory(dir_name);
        self.0.write(&operation)?;
        for entry in dir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.write_directory(path)?
            } else {
                self.write_file(path)?
            }
        }
        let operation = StorageOperation::LeaveDirectory;
        debug!("<- Leave directory");
        self.0.write(&operation)?;
        Ok(())
    }
}

/// Struct that creates directories or files from a Read implementation.
pub struct StorageReader<R: Read>(StructReader<R>);

impl<R: Read> StorageReader<R> {
    pub fn new(reader: R) -> Self {
        Self(StructReader::new(reader))
    }

    /// Recursively nterpret the instructions in the reader
    pub fn interpret(
        &mut self,
        mut base_path: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // ensure base_path exists
        std::fs::create_dir_all(&base_path)?;

        let operation_or_error = self.0.next::<StorageOperation>();
        let operation = match operation_or_error {
            Ok(operation) => operation,
            Err(a) => {
                // Show the error
                debug!("Stopped reading: {}", a);
                return Ok(());
            }
        };
        match operation {
            StorageOperation::EnterDirectory(name) => {
                debug!("-> Enter directory {}", name);
                let path = base_path.join(name);
                std::fs::create_dir(&path)?;
                return self.interpret(path);
            }
            StorageOperation::LeaveDirectory => {
                debug!("<- Leave directory");
                // Pop the last directory from the path and call interpret
                // again
                base_path.pop();
                return self.interpret(base_path);
            }
            StorageOperation::CreateFile(name, size) => {
                debug!("** Create file {} ({} bytes)", name, size);
                let path = base_path.join(name);
                let mut file = File::create(&path)?;
                let mut bytes_left = size;
                while bytes_left > 0 {
                    let bytes_to_read = std::cmp::min(bytes_left, 1024 * 1024);
                    let bytes = self.0.next_bytes(bytes_to_read as usize)?;
                    file.write_all(&bytes)?;
                    bytes_left -= bytes_to_read;
                }
                return self.interpret(base_path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_storage() {
        // Test using assets/test written to a temporary file
        let file = tempfile::NamedTempFile::new().unwrap();
        let mut writer = StorageWriter::new(file.reopen().unwrap());

        let path = Path::new("assets/test").to_path_buf();
        writer.write_directory(path).unwrap();

        let mut reader = StorageReader::new(file.reopen().unwrap());
        let path = tempfile::tempdir().unwrap().into_path();
        reader.interpret(path).unwrap();
    }
}
