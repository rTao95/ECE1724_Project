mod chunker;
mod storage_manager;
mod utils;

use std::env;
use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_file> <output_dir>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_dir = &args[2];

    // Ensure output directory exists
    std::fs::create_dir_all(output_dir)?;

    // Chunk the file and store each chunk
    let chunks_metadata = chunker::chunk_file(input_file)?;
    let mut file = File::open(input_file)?;

    // Read each chunk and save it
    for metadata in &chunks_metadata {
        let mut buffer = vec![0; metadata.size];
        file.read_exact(&mut buffer)?;

        storage_manager::save_chunk(&buffer, metadata, output_dir)?;
        println!("Saved chunk with hash: {}", metadata.hash);
    }

    Ok(())
}
