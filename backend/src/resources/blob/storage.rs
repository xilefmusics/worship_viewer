use std::path::{Path, PathBuf};

use actix_files::NamedFile;

use shared::blob::Blob;

use crate::error::AppError;

/// Abstracts blob file I/O. Enables mocking in tests.
pub trait BlobStorage: Send + Sync {
    fn write_blob_file(&self, blob: &Blob) -> Result<(), AppError>;
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
        let file_name = blob
            .file_name()
            .ok_or_else(|| AppError::Internal("blob has no id".into()))?;
        let path = Path::new(&self.blob_dir).join(file_name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| AppError::Internal(e.to_string()))?;
        }
        std::fs::write(&path, []).map_err(|e| AppError::Internal(e.to_string()))
    }

    fn delete_blob_file(&self, blob: &Blob) {
        if let Some(name) = blob.file_name() {
            let path = Path::new(&self.blob_dir).join(name);
            let _ = std::fs::remove_file(path);
        }
    }

    fn open_blob_data_file(&self, blob: &Blob) -> Result<NamedFile, AppError> {
        let file_name = blob
            .file_name()
            .ok_or_else(|| AppError::NotFound("blob has no id".into()))?;
        NamedFile::open(Path::new(&self.blob_dir).join(PathBuf::from(file_name)))
            .map_err(|err| AppError::Internal(format!("{}", err)))
    }
}
