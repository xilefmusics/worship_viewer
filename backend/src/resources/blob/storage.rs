use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use actix_files::NamedFile;

use shared::blob::Blob;

use crate::error::AppError;

/// Abstracts blob file I/O. Enables mocking in tests.
pub trait BlobStorage: Send + Sync {
    fn write_blob_file(&self, blob: &Blob) -> Result<(), AppError>;
    fn write_blob_bytes(&self, blob: &Blob, data: &[u8]) -> Result<(), AppError>;
    fn delete_blob_file(&self, blob: &Blob);
    fn open_blob_data_file(&self, blob: &Blob) -> Result<NamedFile, AppError>;
}

/// Production filesystem-backed blob storage.
#[derive(Clone)]
pub struct FsBlobStorage {
    blob_dir: String,
}

impl FsBlobStorage {
    pub fn new(blob_dir: String) -> Self {
        Self { blob_dir }
    }
}

impl BlobStorage for FsBlobStorage {
    fn write_blob_file(&self, blob: &Blob) -> Result<(), AppError> {
        self.write_blob_bytes(blob, &[])
    }

    fn write_blob_bytes(&self, blob: &Blob, data: &[u8]) -> Result<(), AppError> {
        let file_name = blob
            .file_name()
            .ok_or_else(|| AppError::Internal("blob has no id".into()))?;
        let path = Path::new(&self.blob_dir).join(file_name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)
                .map_err(|e| AppError::internal_from_err("blob.storage.create_dir_all", e))?;
        }
        std::fs::write(&path, data).map_err(|e| AppError::internal_from_err("blob.storage.write", e))
    }

    fn delete_blob_file(&self, blob: &Blob) {
        if let Some(name) = blob.file_name() {
            let path = Path::new(&self.blob_dir).join(name);
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(err) if err.kind() == ErrorKind::NotFound => {
                    tracing::debug!(
                        blob_id = %blob.id,
                        path = %path.display(),
                        "blob file already absent during delete"
                    );
                }
                Err(err) => {
                    tracing::warn!(
                        blob_id = %blob.id,
                        path = %path.display(),
                        error = %err,
                        "failed to delete blob file"
                    );
                }
            }
        }
    }

    fn open_blob_data_file(&self, blob: &Blob) -> Result<NamedFile, AppError> {
        let file_name = blob
            .file_name()
            .ok_or_else(|| AppError::NotFound("blob has no id".into()))?;
        NamedFile::open(Path::new(&self.blob_dir).join(PathBuf::from(file_name)))
            .map_err(|err| AppError::internal_from_err("blob.storage.open", err))
    }
}
