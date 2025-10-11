use super::{Format, QueryParams};
use crate::error::AppError;
use crate::rest::parse_user_header;
use crate::song::Model as SongModel;
use crate::song::Song;
use crate::user::Model as UserModel;
use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    HttpRequest, HttpResponse,
};
use fancy_surreal::Client;
use reqwest::multipart::{Form, Part};
use shared::song::wrap_html;
use std::io::{Cursor, Write};
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

#[get("/api/export")]
pub async fn get(
    req: HttpRequest,
    db: Data<Client<'_>>,
    q: Query<QueryParams>,
) -> Result<HttpResponse, AppError> {
    let db = db.into_inner();
    let filter = q.to_filter();
    let owners = UserModel::get_or_create(db.clone(), &parse_user_header(&req)?)
        .await?
        .read;
    let songs = SongModel::get(db.clone(), owners.clone(), &filter).await?;

    match q.format {
        Format::WorshipPro => export(songs, true),
        Format::ChordPro => export(songs, false),
        Format::Pdf => export_pdf(songs).await,
    }
}

fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn export(songs: Vec<Song>, worship_pro_features: bool) -> Result<HttpResponse, AppError> {
    let ending = if worship_pro_features { "wp" } else { "chopro" };

    if songs.len() == 1 {
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
        .map_err(AppError::from)?;
        zip.write_all(
            song.format_chord_pro(None, None, None, worship_pro_features)
                .as_bytes(),
        )
        .map_err(AppError::from)?;
    }

    zip.finish().map_err(AppError::from)?;
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

    let settings = crate::settings::get();
    let resp = http
        .post(format!(
            "http://{}:{}/print",
            settings.printer_host, settings.printer_port
        ))
        .multipart(form)
        .send()
        .await
        .map_err(AppError::from)?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_text = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!(
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
