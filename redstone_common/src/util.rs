use std::{io::Read, path::PathBuf};

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

use crate::model::Result;

pub fn generate_sha256_digest(path: &PathBuf) -> Result<String> {
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
