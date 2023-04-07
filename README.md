# Backup Application

This Rust-based backup application allows you to back up a directory and restore it later using a set of files with a specified maximum size. The backup is created using the `multipart` and `storage` modules, which handle the creation of multiple files and format of the backup, respectively. The application also supports generating a report of the backup, which can be useful for tracking changes and understanding the backup's content.

## Installation of Rustup

Building this project requires a Rust build toolchain. follow the instructions on the official Rust website: https://rustup.rs/. This will guide you through installing Rustup, which is the recommended way to manage multiple Rust versions and associated tools.

## Building and Running the Application

To build the application, follow these steps:

1. Open a terminal/command prompt and navigate to the project root directory (where the `Cargo.toml` file is located).
2. Run the following command to build the application:

```bash
cargo build --release
```

This will create an executable file in the `target/release` directory.

To run the application, execute the binary created in the `target/release` directory. For example, on Unix-like systems:

```bash
./target/release/mbackk
```

On Windows:

```cmd
target\release\mbackk.exe
```

## Command Arguments

The application has two main operations: `Backup` and `Restore`. Each operation requires specific arguments to be provided. Below is a list of all command arguments and their usage:

### Backup

- `origin`: The origin directory path to back up.
- `destination`: The destination directory path to save the backup.
- `report`: (Optional) Write a report of the backup to the destination path. Default is `true`.
- `max_file_size`: (Optional) The maximum size of each file in the backup in bytes. Default is `536870912`, 512Mb.

### Restore

- `origin`: The origin directory path of the backup to restore.
- `destination`: The destination directory path to restore the backup.

## Example Usage

### Backup

To create a backup of the directory `/path/to/origin` to `/path/to/destination` with a maximum file size of 512 MB and generate a report:

```bash
./target/release/mbackk backup /path/to/origin /path/to/destination true 536870912
```

### Restore

To restore a backup from `/path/to/origin` to `/path/to/destination`:

```bash
./target/release/mbackk restore /path/to/origin /path/to/destination
```
