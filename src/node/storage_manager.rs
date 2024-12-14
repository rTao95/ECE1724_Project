use crate::node::chunker::ChunkMetadata;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Save a chunk to a specified directory with a custom chunk filename.
pub fn save_chunk(
    chunk_data: &[u8],
    _chunk_metadata: &ChunkMetadata,
    output_dir: &str,
    chunk_filename: &str,
) -> io::Result<()> {
    let chunk_path = Path::new(output_dir).join(chunk_filename);
    let mut file = File::create(chunk_path)?;
    file.write_all(chunk_data)?;
    Ok(())
}