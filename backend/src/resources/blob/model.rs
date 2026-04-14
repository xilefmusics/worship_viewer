use std::path::{Path, PathBuf};

use actix_files::NamedFile;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

use shared::api::ListQuery;
use shared::blob::{Blob, CreateBlob};

use crate::database::Database;
use crate::error::AppError;
use crate::resources::User;
use crate::resources::common::{belongs_to, resource_id};
use crate::resources::team::{content_read_team_things, content_write_team_things};
use crate::settings::Settings;

pub trait Model {
    async fn get_blobs(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError>;
    async fn get_blob(&self, read_teams: Vec<Thing>, id: &str) -> Result<Blob, AppError>;
    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError>;
    async fn update_blob(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError>;
    async fn delete_blob(&self, write_teams: Vec<Thing>, id: &str) -> Result<Blob, AppError>;
}

impl Model for Database {
    async fn get_blobs(
        &self,
        read_teams: Vec<Thing>,
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError> {
        let mut query = String::from("SELECT * FROM blob WHERE owner IN $teams");
        if pagination.to_offset_limit().is_some() {
            query.push_str(" LIMIT $limit START $start");
        }

        let mut request = self.db.query(query).bind(("teams", read_teams));
        if let Some((offset, limit)) = pagination.to_offset_limit() {
            request = request.bind(("limit", limit)).bind(("start", offset));
        }

        let mut response = request.await.map_err(AppError::database)?;

        Ok(response
            .take::<Vec<BlobRecord>>(0)?
            .into_iter()
            .map(BlobRecord::into_blob)
            .collect())
    }

    async fn get_blob(&self, read_teams: Vec<Thing>, id: &str) -> Result<Blob, AppError> {
        let record: Option<BlobRecord> = self.db.select(resource_id("blob", id)?).await?;
        match record {
            Some(r) if belongs_to(&r.owner, &read_teams) => Ok(r.into_blob()),
            _ => Err(AppError::NotFound("blob not found".into())),
        }
    }

    async fn create_blob(&self, owner: &str, blob: CreateBlob) -> Result<Blob, AppError> {
        let owner_team = self.personal_team_thing_for_user(owner).await?;
        let created = self
            .db
            .create("blob")
            .content(BlobRecord::from_payload(
                None,
                Some(owner_team),
                Some(Utc::now().into()),
                blob,
            ))
            .await?
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::database("failed to create blob"))?;
        write_blob_file(&created)?;
        Ok(created)
    }

    async fn update_blob(
        &self,
        write_teams: Vec<Thing>,
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError> {
        let (tb, sid) = resource_id("blob", id)?;

        let mut response = self
            .db
            .query(
                "UPDATE type::thing($tb, $sid) SET file_type = $file_type, width = $width, \
                 height = $height, ocr = $ocr WHERE owner IN $teams RETURN AFTER",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("file_type", blob.file_type))
            .bind(("width", blob.width))
            .bind(("height", blob.height))
            .bind(("ocr", blob.ocr.clone()))
            .bind(("teams", write_teams))
            .await?;

        let rows: Vec<BlobRecord> = response.take(0)?;
        let updated = rows
            .into_iter()
            .next()
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))?;
        write_blob_file(&updated)?;
        Ok(updated)
    }

    async fn delete_blob(&self, write_teams: Vec<Thing>, id: &str) -> Result<Blob, AppError> {
        let (tb, sid) = resource_id("blob", id)?;
        let mut response = self
            .db
            .query(
                "DELETE FROM type::thing($tb, $sid) WHERE owner IN $teams RETURN BEFORE",
            )
            .bind(("tb", tb))
            .bind(("sid", sid))
            .bind(("teams", write_teams))
            .await?;

        let rows: Vec<BlobRecord> = response.take(0)?;
        let deleted = rows
            .into_iter()
            .next()
            .map(BlobRecord::into_blob)
            .ok_or_else(|| AppError::NotFound("blob not found".into()))?;
        if let Some(name) = deleted.file_name() {
            let path = Path::new(&Settings::global().blob_dir).join(name);
            let _ = std::fs::remove_file(path);
        }
        Ok(deleted)
    }
}

impl Database {
    pub async fn list_blobs_for_user(
        &self,
        user: &User,
        pagination: ListQuery,
    ) -> Result<Vec<Blob>, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_blobs(read_teams, pagination).await
    }

    pub async fn get_blob_for_user(&self, user: &User, id: &str) -> Result<Blob, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        self.get_blob(read_teams, id).await
    }

    pub async fn create_blob_for_user(
        &self,
        user: &User,
        blob: CreateBlob,
    ) -> Result<Blob, AppError> {
        self.create_blob(&user.id, blob).await
    }

    pub async fn update_blob_for_user(
        &self,
        user: &User,
        id: &str,
        blob: CreateBlob,
    ) -> Result<Blob, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.update_blob(write_teams, id, blob).await
    }

    pub async fn delete_blob_for_user(&self, user: &User, id: &str) -> Result<Blob, AppError> {
        let write_teams = content_write_team_things(self, user).await?;
        self.delete_blob(write_teams, id).await
    }

    pub async fn open_blob_data_file_for_user(
        &self,
        user: &User,
        id: &str,
    ) -> Result<NamedFile, AppError> {
        let read_teams = content_read_team_things(self, user).await?;
        let blob = self.get_blob(read_teams, id).await?;
        let file_name = blob
            .file_name()
            .ok_or_else(|| AppError::NotFound("blob has no id".into()))?;
        NamedFile::open(Path::new(&Settings::global().blob_dir).join(PathBuf::from(file_name)))
            .map_err(|err| AppError::Internal(format!("{}", err)))
    }
}

fn write_blob_file(blob: &Blob) -> Result<(), AppError> {
    let file_name = blob
        .file_name()
        .ok_or_else(|| AppError::Internal("blob has no id".into()))?;
    let path = Path::new(&Settings::global().blob_dir).join(file_name);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| AppError::Internal(e.to_string()))?;
    }
    std::fs::write(&path, []).map_err(|e| AppError::Internal(e.to_string()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BlobRecord {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    id: Option<Thing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    owner: Option<Thing>,
    file_type: shared::blob::FileType,
    width: u32,
    height: u32,
    #[serde(default)]
    ocr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    created_at: Option<Datetime>,
}

impl BlobRecord {
    fn into_blob(self) -> Blob {
        Blob {
            id: self
                .id
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            owner: self
                .owner
                .map(|thing| thing.id.to_string())
                .unwrap_or_default(),
            file_type: self.file_type,
            width: self.width,
            height: self.height,
            ocr: self.ocr,
        }
    }

    fn from_payload(
        id: Option<Thing>,
        owner: Option<Thing>,
        created_at: Option<Datetime>,
        blob: CreateBlob,
    ) -> Self {
        Self {
            id,
            owner,
            file_type: blob.file_type,
            width: blob.width,
            height: blob.height,
            ocr: blob.ocr,
            created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::blob::FileType;

    #[test]
    fn blob_record_from_payload_into_blob() {
        let id = Thing::from(("blob".to_owned(), "b99".to_owned()));
        let owner = Thing::from(("team".to_owned(), "tm".to_owned()));
        let record = BlobRecord::from_payload(
            Some(id.clone()),
            Some(owner.clone()),
            None,
            CreateBlob {
                file_type: FileType::SVG,
                width: 640,
                height: 480,
                ocr: "text".into(),
            },
        );
        let b = record.into_blob();
        assert_eq!(b.id, "b99");
        assert_eq!(b.owner, "tm");
        assert_eq!(b.file_type, FileType::SVG);
        assert_eq!(b.width, 640);
        assert_eq!(b.height, 480);
        assert_eq!(b.ocr, "text");
    }
}
