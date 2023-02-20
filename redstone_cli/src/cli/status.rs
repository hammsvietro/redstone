use std::env::current_dir;

use redstone_common::model::{
    backup::{get_index_file_for_path, IndexFile},
    fs_tree::FSTree,
    DomainError, RedstoneError, Result,
};

pub fn run_status_cmd() -> Result<()> {
    let path = current_dir()?;
    let index_file_path = get_index_file_for_path(&path);
    if !index_file_path.exists() {
        let path = path.to_str().unwrap().into();
        return Err(RedstoneError::DomainError(DomainError::BackupDoesntExist(
            path,
        )));
    }
    let index_file = IndexFile::from_file(&index_file_path)?;
    let current_fs_tree = FSTree::build(path, None)?;
    let diff = current_fs_tree.diff(&index_file.last_fs_tree)?;

    println!("{}", diff.get_changes_message());

    Ok(())
}
