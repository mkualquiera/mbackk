use std::{env, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use log::{error, info};

mod multipart;
mod report;
mod storage;
mod struct_stream;

/// Take an origin path and back it up to a destination path composed of files
/// with a specified maximum size.
pub fn backup(
    origin: PathBuf,
    destination: PathBuf,
    max_file_size: u64,
    write_report: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Writing origin to backup with max file size {} bytes...",
        max_file_size
    );
    // Create the multipart writer to save the backup.
    let mut writer =
        multipart::MultipartWriter::new(destination.clone(), max_file_size);
    // Create the storage writer to read the origin.
    let mut storage_writer = storage::StorageWriter::new(&mut writer);
    // Write the origin to the storage writer.
    storage_writer.write_directory(origin.clone())?;

    if write_report {
        info!("Writing backup report...");
        report::backup_report(origin, destination)?;
    }

    Ok(())
}

/// Restore a backup from a path composed of files to a destination path.
/// The backup must have been created with the `backup` function.
pub fn restore(
    origin: PathBuf,
    destination: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Restoring backup to destination...");
    // Create the multipart reader to read the backup.
    let mut reader = multipart::MultipartReader::new(origin);
    // Create the storage reader to restore the backup.
    let mut storage_reader = storage::StorageReader::new(&mut reader);
    // Read the storage reader to the destination.
    storage_reader.interpret(destination)
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// The log level to use.
    #[arg(default_value = "info")]
    log_level: String,
    #[command(subcommand)]
    operation: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Backup the origin path to the destination path.
    Backup(Backup),
    /// Restore the origin path to the destination path.
    Restore(Restore),
}

#[derive(Args)]
struct Backup {
    /// The origin path to backup.
    origin: PathBuf,
    /// The destination path to save the backup.
    destination: PathBuf,
    /// Write a report of the backup to the destination path.
    #[arg(default_value = "true")]
    report: Option<bool>,
    /// The maximum size of each file in the backup.
    #[arg(default_value = "512")]
    max_file_size: Option<u64>,
}

#[derive(Args)]
struct Restore {
    /// The origin path to restore.
    origin: PathBuf,
    /// The destination path to restore the backup.
    destination: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", cli.log_level);
    }
    env_logger::init();

    let result = match cli.operation {
        Commands::Backup(args) => backup(
            args.origin,
            args.destination,
            args.max_file_size.unwrap() * 1024 * 1024,
            args.report.unwrap(),
        ),
        Commands::Restore(args) => restore(args.origin, args.destination),
    };

    if let Err(err) = result {
        error!("Failed to execute operation: {}", err);
        std::process::exit(1);
    }
    info!("Operation completed successfully.");
}
