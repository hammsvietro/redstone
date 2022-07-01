use std::{io::Read, path::PathBuf};

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

use crate::constants::BLOCK_SIZE;

pub fn generate_sha256_digest(path: &PathBuf) -> std::io::Result<String> {
    let input = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(input);

    let digest = {
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1024];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };
    Ok(HEXLOWER.encode(digest.as_ref()))
}

pub fn generate_sha256_digest_chunks(path: &PathBuf) -> std::io::Result<Vec<String>> {
    let input = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(input);
    let mut chunks: Vec<String> = Vec::new();

    let mut buffer = [0; BLOCK_SIZE];
    let mut last_count = 0;
    loop {
        let mut hasher = Sha256::new();
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[last_count..count]);
        let chunk_digest = HEXLOWER.encode(hasher.finalize().as_ref());
        chunks.push(chunk_digest);
        last_count = count;
    }
    Ok(chunks)
}
