use crate::song::Song;
use crate::AppError;
use awc::Client;

pub async fn import(identifier: &str) -> Result<Song, AppError> {
    if identifier.starts_with("tabs/ultimate-guitar/com/tab/") {
        Ok(Song::import_ultimate_guitar(&String::from_utf8(
            Client::default()
                .get("https://tabs.ultimate-guitar.com/tab/".to_string() + &identifier[29..])
                .send()
                .await?
                .body()
                .await?
                .to_vec(),
        )?)?)
    } else {
        Err(AppError::NotFound("import method not found".into()))
    }
}
