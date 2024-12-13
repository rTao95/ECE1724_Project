use rfd::FileDialog;
use std::path::PathBuf;

/// Opens a file dialog for the user to select a file.
/// Returns the path to the selected file.
pub fn select_file() -> Option<PathBuf> {
    FileDialog::new().pick_file()
}