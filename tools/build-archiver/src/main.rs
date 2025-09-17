use anyhow::Result;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use zip::write::{FileOptions, SimpleFileOptions};
use zip::ZipWriter;

fn main() -> Result<()> {
    pretty_env_logger::env_logger::builder().filter_level(log::LevelFilter::Debug).format_timestamp(None).init();
    package_windows()?;
    package_linux()?;

    Ok(())
}

fn package_windows() -> Result<()> {
    info!("Packaging for Windows...");
    // Step 1: Build the project
    let status = Command::new("cargo").args(&["build", "--release", "--bin", "minipx"]).status().expect("Failed to run cargo build");
    if !status.success() {
        panic!("cargo build failed");
    }

    // Step 2: Locate the binary
    let binary_path = Path::new("target/release/minipx.exe");
    if !binary_path.exists() {
        panic!("binary not found at {:?}", binary_path);
    }

    // Step 3: Create zip archive
    let zip_path = Path::new("minipx-windows-x64.zip");
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options: SimpleFileOptions = FileOptions::default().unix_permissions(0o755);
    zip.start_file(binary_path.file_name().unwrap().to_str().unwrap(), options)?;
    let bin_data = std::fs::read(binary_path)?;
    zip.write_all(&bin_data)?;

    zip.finish()?;

    info!("Archived binary to {:?}", zip_path);
    Ok(())
}
fn package_linux() -> Result<()> {
    info!("Packaging for Linux...");
    // Step 1: Build the project
    let status = Command::new("wsl").args(&["bash", "-ic", "cargo build --release --bin minipx"]).status().expect("Failed to run cargo build");

    if !status.success() {
        panic!("cargo build failed");
    }

    // Step 2: Locate the binary
    let binary_path = Path::new("target/release/minipx");
    if !binary_path.exists() {
        panic!("binary not found at {:?}", binary_path);
    }

    // Step 3: Create zip archive
    let zip_path = Path::new("minipx-linux-x64.zip");
    let file = File::create(&zip_path)?;
    let mut zip = ZipWriter::new(file);

    let options: SimpleFileOptions = FileOptions::default().unix_permissions(0o755);
    zip.start_file(binary_path.file_name().unwrap().to_str().unwrap(), options)?;
    let bin_data = std::fs::read(binary_path)?;
    zip.write_all(&bin_data)?;

    zip.finish()?;

    info!("Archived binary to {:?}", zip_path);
    Ok(())
}
