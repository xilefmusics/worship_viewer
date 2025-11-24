use actix_web::{http::header, HttpResponse};
use reqwest::multipart::{Form, Part};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::error::AppError;
use shared::song::{Song, wrap_html};
use std::io::{Cursor, Write};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};
use crate::settings::Settings;

#[derive(Debug, Deserialize, Clone, Default, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    #[serde(alias = "zip")]
    Zip,
    #[default]
    #[serde(alias = "wp")]
    WorshipPro,
    #[serde(alias = "cp")]
    ChordPro,
    #[serde(alias = "pdf")]
    Pdf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct QueryParams {
    #[serde(default)]
    pub format: Format,
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub async fn export(songs: Vec<Song>, format: Format) -> Result<HttpResponse, AppError> {
    match format {
        Format::Zip => export_chord_pro(songs, true, true),
        Format::WorshipPro => export_chord_pro(songs, true, false),
        Format::ChordPro => export_chord_pro(songs, false, false),
        Format::Pdf => export_pdf(songs).await,
    }
}

fn export_chord_pro(
    songs: Vec<Song>,
    worship_pro_features: bool,
    force_zip: bool,
) -> Result<HttpResponse, AppError> {
    let ending = if worship_pro_features { "wp" } else { "chopro" };

    if songs.len() == 1 && !force_zip {
        let content_type = if worship_pro_features {
            "text/x-worshippro"
        } else {
            "text/x-chordpro"
        };
        return Ok(HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, content_type))
            .insert_header((
                header::CONTENT_DISPOSITION,
                format!(
                    "attachment; filename=\"{0}.{ending}\"",
                    sanitize_filename(&songs[0].data.title),
                ),
            ))
            .body(songs[0].format_chord_pro(None, None, None, worship_pro_features)));
    }

    let mut buffer = Cursor::new(Vec::<u8>::new());
    let mut zip = ZipWriter::new(&mut buffer);

    let options: FileOptions<'_, ()> = FileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    for song in &songs {
        zip.start_file(
            &format!("{}.{}", sanitize_filename(&song.data.title), ending),
            options,
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
        zip.write_all(
            song.format_chord_pro(None, None, None, worship_pro_features)
                .as_bytes(),
        )
        .map_err(|err| AppError::Internal(err.to_string()))?;
    }

    zip.finish()
        .map_err(|err| AppError::Internal(err.to_string()))?;
    let bytes = buffer.into_inner();

    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/zip"))
        .insert_header((
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"songs.zip\""),
        ))
        .body(bytes))
}

async fn export_pdf(songs: Vec<Song>) -> Result<HttpResponse, AppError> {
    let css = songs
        .get(0)
        .map(|song| song.format_html(None, None, None, None).1)
        .unwrap_or_default();
    let pages = songs
        .into_iter()
        .map(|song| song.format_html(None, None, None, None).0)
        .collect::<String>();
    let html = wrap_html(&pages, &css, "title");

    let http = reqwest::Client::new();

    let part = Part::bytes(html.into_bytes())
        .file_name("song.html")
        .mime_str("text/html")
        .map_err(AppError::from)?;

    let form = Form::new().part("files", part).text("outline", "false");

    let settings = Settings::global();
    let resp = http
        .post(&settings.printer_address)
        .bearer_auth(&settings.printer_api_key)
        .multipart(form)
        .send()
        .await
        .map_err(AppError::from)?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_text = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "Couldn't print document ({}): {}",
            status, err_text
        )));
    }

    let bytes = resp.bytes().await.map_err(AppError::from)?;

    Ok(HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, "application/pdf"))
        .insert_header((
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"songs.pdf\"",
        ))
        .body(bytes))
}