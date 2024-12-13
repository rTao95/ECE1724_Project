use std::fs::File;
use std::io::{self, Read};

const CHUNK_SIZE: usize =1024 * 15; // 15kb

/// Represents metadata about a file chunk, including its hash.
#[derive(Debug)]
pub struct ChunkMetadata {
    pub size: usize,
}

pub fn chunk_file(file_path: &str) -> io::Result<Vec<ChunkMetadata>> {
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; CHUNK_SIZE];
    let mut chunks_metadata = Vec::new();

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let metadata = ChunkMetadata {
            size: bytes_read,
        };

        chunks_metadata.push(metadata);
    }

    Ok(chunks_metadata)
}
