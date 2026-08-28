//! Temporary workspace directory lifecycle support.

use std::path::{Path, PathBuf};

/// Temporary workspace directory removed on drop.
#[derive(Debug)]
pub struct TempWorkDir {
    path: PathBuf,
}

impl TempWorkDir {
    pub fn new(parent: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let id = uuid::Uuid::new_v4().to_string();
        let path = parent.as_ref().join(id);
        std::fs::create_dir_all(&path)?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.path) {
            tracing::warn!(error = %err, "temp_work_dir.cleanup_failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temp_work_dir_removed_on_drop() {
        let parent =
            std::env::temp_dir().join(format!("worshipviewer_temp_work_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&parent).unwrap();
        let path = {
            let dir = TempWorkDir::new(&parent).unwrap();
            let p = dir.path().to_path_buf();
            assert!(p.exists());
            p
        };
        assert!(!path.exists());
        let _ = std::fs::remove_dir_all(&parent);
    }
}
