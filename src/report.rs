use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::Serialize;

use md5::Context;

#[derive(Serialize)]
struct FileReportInfo {
    path: PathBuf,
    size: u64,
    md5: String,
}

#[derive(Serialize)]
struct StorageReport {
    backed_up_files: Vec<FileReportInfo>,
    storage_files: Vec<FileReportInfo>,
}

/// Given an origin path for backup and a destination path for the backup,
/// saves a JSON report of the backup to the destination path.
pub fn backup_report(
    origin: PathBuf,
    destination: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let origin_files = collect_files_info(&origin)?;
    let destination_files = collect_files_info(&destination)?;

    let storage_report = StorageReport {
        backed_up_files: origin_files,
        storage_files: destination_files,
    };

    let report_json = serde_json::to_string_pretty(&storage_report)?;
    let report_path = destination.join("backup_report.json");
    let mut report_file = fs::File::create(report_path)?;
    report_file.write_all(report_json.as_bytes())?;

    Ok(())
}

fn collect_files_info(
    path: &PathBuf,
) -> Result<Vec<FileReportInfo>, io::Error> {
    let mut files_info = Vec::new();
    collect_files_info_recursive(path, &mut files_info)?;
    Ok(files_info)
}

fn collect_files_info_recursive(
    path: &PathBuf,
    files_info: &mut Vec<FileReportInfo>,
) -> Result<(), io::Error> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let entry_path = entry.path();
        if metadata.is_file() {
            let md5 = calculate_md5(&entry_path)?;
            files_info.push(FileReportInfo {
                path: entry_path,
                size: metadata.len(),
                md5,
            });
        } else if metadata.is_dir() {
            collect_files_info_recursive(&entry_path, files_info)?;
        }
    }
    Ok(())
}

fn calculate_md5(path: &PathBuf) -> Result<String, io::Error> {
    let mut buffer = Vec::new();
    let mut file = fs::File::open(path)?;
    file.read_to_end(&mut buffer)?;

    let mut hasher = Context::new();
    hasher.consume(buffer);

    let result = hasher.compute();
    Ok(format!("{result:x}"))
}
