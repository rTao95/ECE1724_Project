use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;
use crate::chunker::ChunkMetadata;

/// Save a chunk to a specified directory with its hash as the filename.
pub fn save_chunk(chunk_data: &[u8], chunk_metadata: &ChunkMetadata, output_dir: &str) -> io::Result<()> {
    let chunk_path = Path::new(output_dir).join(format!("{}.dat", chunk_metadata.hash));
    let mut file = File::create(chunk_path)?;
    file.write_all(chunk_data)?;
    Ok(())
}

/// Load a chunk by hash from the specified directory.
pub fn load_chunk(hash: &str, output_dir: &str) -> io::Result<Vec<u8>> {
    let chunk_path = Path::new(output_dir).join(format!("{}.dat", hash));
    fs::read(chunk_path)
}
