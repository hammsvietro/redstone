use std::{io::Read, path::Path};

use data_encoding::HEXLOWER;
use sha2::{Digest, Sha256};

use crate::model::Result;

pub fn generate_sha256_digest(path: &Path) -> Result<String> {
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

pub fn bytes_to_human_readable(bytes: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
    let mut bytes = bytes as f64;
    let mut unit = 0;
    while bytes >= 1024.0 {
        bytes /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", bytes, units[unit])
}
