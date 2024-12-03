use rfd::FileDialog;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

/// Opens a file dialog for the user to select a file.
/// Returns the path to the selected file.
pub fn select_file() -> Option<PathBuf> {
    FileDialog::new().pick_file()
}

/// Reads the contents of the file at the given path into a byte vector.
pub fn read_file_to_bytes(file_path: &PathBuf) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}