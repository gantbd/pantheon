//! Shared owner-private document loading for effect and evidence adapters.

use std::{
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
};

#[derive(Debug)]
pub enum PrivateDocumentError {
    Unavailable(std::io::Error),
    InvalidShape,
    Changed,
}

impl std::fmt::Display for PrivateDocumentError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable(error) => write!(formatter, "unavailable: {error}"),
            Self::InvalidShape => {
                formatter.write_str("not one nonempty owner-private regular file")
            }
            Self::Changed => formatter.write_str("changed while being read"),
        }
    }
}

impl std::error::Error for PrivateDocumentError {}

pub fn read(path: &Path, maximum_bytes: u64) -> Result<Vec<u8>, PrivateDocumentError> {
    if maximum_bytes == 0 {
        return Err(PrivateDocumentError::InvalidShape);
    }
    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(nix::libc::O_NOFOLLOW)
        .open(path)
        .map_err(PrivateDocumentError::Unavailable)?;
    let before = file.metadata().map_err(PrivateDocumentError::Unavailable)?;
    if !before.file_type().is_file()
        || before.permissions().mode() & 0o077 != 0
        || before.nlink() != 1
        || before.len() == 0
        || before.len() > maximum_bytes
    {
        return Err(PrivateDocumentError::InvalidShape);
    }
    let mut bytes = Vec::with_capacity(before.len() as usize);
    file.by_ref()
        .take(maximum_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(PrivateDocumentError::Unavailable)?;
    let after = file.metadata().map_err(PrivateDocumentError::Unavailable)?;
    if bytes.len() as u64 != before.len()
        || before.len() != after.len()
        || before.dev() != after.dev()
        || before.ino() != after.ino()
    {
        return Err(PrivateDocumentError::Changed);
    }
    Ok(bytes)
}
