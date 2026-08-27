//! Slide-deck source probing, SVG sanitization, and PDF splitting.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use lopdf::{Document, LoadOptions};
use shared::MediaAssetKind;

use crate::process_runner::TempWorkDir;
use crate::resources::media::av_processor::AvProcessFailure;

const MAX_PDF_DECOMPRESSED_STREAM_BYTES: usize = 256 * 1024 * 1024;

#[derive(Debug)]
pub struct DeckPageOutput {
    pub path: PathBuf,
    pub content_type: &'static str,
    pub kind: MediaAssetKind,
}

#[derive(Debug)]
pub struct DeckExpandResult {
    pub pages: Vec<DeckPageOutput>,
    _work: Option<TempWorkDir>,
}

impl DeckExpandResult {
    pub fn new(pages: Vec<DeckPageOutput>, work: Option<TempWorkDir>) -> Self {
        Self { pages, _work: work }
    }
}

#[async_trait]
pub trait DeckProcessor: Send + Sync {
    async fn expand_source(
        &self,
        input: &Path,
        declared_kind: MediaAssetKind,
        work_parent: &Path,
        remaining_page_budget: usize,
    ) -> Result<DeckExpandResult, AvProcessFailure>;
}

pub struct UnsupportedDeckProcessor;

#[async_trait]
impl DeckProcessor for UnsupportedDeckProcessor {
    async fn expand_source(
        &self,
        _input: &Path,
        _declared_kind: MediaAssetKind,
        _work_parent: &Path,
        _remaining_page_budget: usize,
    ) -> Result<DeckExpandResult, AvProcessFailure> {
        Err(AvProcessFailure::InputUnsupported)
    }
}

#[derive(Clone)]
pub struct LopdfDeckProcessor {
    pub timeout: Duration,
}

#[async_trait]
impl DeckProcessor for LopdfDeckProcessor {
    async fn expand_source(
        &self,
        input: &Path,
        declared_kind: MediaAssetKind,
        work_parent: &Path,
        remaining_page_budget: usize,
    ) -> Result<DeckExpandResult, AvProcessFailure> {
        let input = input.to_path_buf();
        let work_parent = work_parent.to_path_buf();
        let task = tokio::task::spawn_blocking(move || {
            expand_source_blocking(&input, declared_kind, &work_parent, remaining_page_budget)
        });
        tokio::time::timeout(self.timeout, task)
            .await
            .map_err(|_| AvProcessFailure::TimedOut)?
            .map_err(|_| AvProcessFailure::Failed)?
    }
}

fn expand_source_blocking(
    input: &Path,
    declared_kind: MediaAssetKind,
    work_parent: &Path,
    remaining_page_budget: usize,
) -> Result<DeckExpandResult, AvProcessFailure> {
    let detected = detect_deck_source_path(input)?;
    if !declared_kind_matches(declared_kind, detected) {
        return Err(AvProcessFailure::InputUnsupported);
    }
    match detected {
        DetectedDeckSource::Png | DetectedDeckSource::Jpeg => {
            if remaining_page_budget < 1 {
                return Err(AvProcessFailure::InputUnsupported);
            }
            Ok(DeckExpandResult::new(
                vec![DeckPageOutput {
                    path: input.to_path_buf(),
                    content_type: detected.content_type(),
                    kind: MediaAssetKind::Image,
                }],
                None,
            ))
        }
        DetectedDeckSource::Svg => {
            if remaining_page_budget < 1 {
                return Err(AvProcessFailure::InputUnsupported);
            }
            let text =
                std::fs::read_to_string(input).map_err(|_| AvProcessFailure::InputInvalid)?;
            let sanitized = sanitize_svg(&text)?;
            let work = TempWorkDir::new(work_parent).map_err(|_| AvProcessFailure::Failed)?;
            let output = work.path().join("page.svg");
            std::fs::write(&output, sanitized.as_bytes()).map_err(|_| AvProcessFailure::Failed)?;
            Ok(DeckExpandResult::new(
                vec![DeckPageOutput {
                    path: output,
                    content_type: "image/svg+xml",
                    kind: MediaAssetKind::Svg,
                }],
                Some(work),
            ))
        }
        DetectedDeckSource::Pdf => expand_pdf(input, work_parent, remaining_page_budget),
    }
}

fn expand_pdf(
    input: &Path,
    work_parent: &Path,
    remaining_page_budget: usize,
) -> Result<DeckExpandResult, AvProcessFailure> {
    let document = Document::load_with_options(
        input,
        LoadOptions {
            max_decompressed_size: Some(MAX_PDF_DECOMPRESSED_STREAM_BYTES),
            ..LoadOptions::default()
        },
    )
    .map_err(|_| AvProcessFailure::InputInvalid)?;
    if document.was_encrypted() {
        return Err(AvProcessFailure::InputInvalid);
    }
    let page_count = document.get_pages().len();
    if page_count == 0 {
        return Err(AvProcessFailure::InputInvalid);
    }
    if page_count > remaining_page_budget {
        return Err(AvProcessFailure::InputUnsupported);
    }
    if page_count == 1 {
        return Ok(DeckExpandResult::new(
            vec![DeckPageOutput {
                path: input.to_path_buf(),
                content_type: "application/pdf",
                kind: MediaAssetKind::Pdf,
            }],
            None,
        ));
    }

    let work = TempWorkDir::new(work_parent).map_err(|_| AvProcessFailure::Failed)?;
    let all_pages: Vec<u32> = document.get_pages().keys().copied().collect();
    let mut pages = Vec::with_capacity(page_count);
    for page_number in &all_pages {
        let mut page_document = document.clone();
        let pages_to_delete: Vec<u32> = all_pages
            .iter()
            .copied()
            .filter(|candidate| candidate != page_number)
            .collect();
        page_document.delete_pages(&pages_to_delete);
        page_document.prune_objects();
        page_document.renumber_objects();
        if page_document.get_pages().len() != 1 {
            return Err(AvProcessFailure::Failed);
        }
        let path = work.path().join(format!("page-{page_number}.pdf"));
        page_document
            .save(&path)
            .map_err(|_| AvProcessFailure::Failed)?;
        pages.push(DeckPageOutput {
            path,
            content_type: "application/pdf",
            kind: MediaAssetKind::Pdf,
        });
    }
    Ok(DeckExpandResult::new(pages, Some(work)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DetectedDeckSource {
    Png,
    Jpeg,
    Svg,
    Pdf,
}

impl DetectedDeckSource {
    fn content_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Svg => "image/svg+xml",
            Self::Pdf => "application/pdf",
        }
    }
}

fn declared_kind_matches(declared: MediaAssetKind, detected: DetectedDeckSource) -> bool {
    matches!(
        (declared, detected),
        (
            MediaAssetKind::Image,
            DetectedDeckSource::Png | DetectedDeckSource::Jpeg
        ) | (MediaAssetKind::Svg, DetectedDeckSource::Svg)
            | (MediaAssetKind::Pdf, DetectedDeckSource::Pdf)
    )
}

fn detect_deck_source(bytes: &[u8]) -> Result<DetectedDeckSource, AvProcessFailure> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        if png_is_animated(bytes) {
            return Err(AvProcessFailure::InputUnsupported);
        }
        return Ok(DetectedDeckSource::Png);
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Ok(DetectedDeckSource::Jpeg);
    }
    if bytes.starts_with(b"%PDF-") {
        return Ok(DetectedDeckSource::Pdf);
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Err(AvProcessFailure::InputUnsupported);
    }
    if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP") {
        return Err(AvProcessFailure::InputUnsupported);
    }
    if looks_like_svg(bytes) {
        return Ok(DetectedDeckSource::Svg);
    }
    Err(AvProcessFailure::InputUnsupported)
}

pub fn detect_deck_source_kind(path: &Path) -> Result<MediaAssetKind, AvProcessFailure> {
    match detect_deck_source_path(path)? {
        DetectedDeckSource::Png | DetectedDeckSource::Jpeg => Ok(MediaAssetKind::Image),
        DetectedDeckSource::Svg => Ok(MediaAssetKind::Svg),
        DetectedDeckSource::Pdf => Ok(MediaAssetKind::Pdf),
    }
}

fn detect_deck_source_path(path: &Path) -> Result<DetectedDeckSource, AvProcessFailure> {
    const SNIFF_BYTES: usize = 64 * 1024;
    let mut file = std::fs::File::open(path).map_err(|_| AvProcessFailure::InputInvalid)?;
    let mut bytes = Vec::with_capacity(SNIFF_BYTES);
    file.by_ref()
        .take(SNIFF_BYTES as u64)
        .read_to_end(&mut bytes)
        .map_err(|_| AvProcessFailure::InputInvalid)?;
    detect_deck_source(&bytes)
}

fn png_is_animated(bytes: &[u8]) -> bool {
    let mut offset = 8usize;
    while offset + 12 <= bytes.len() {
        let length =
            u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap_or([0; 4])) as usize;
        let ty = &bytes[offset + 4..offset + 8];
        if ty == b"acTL" {
            return true;
        }
        if ty == b"IDAT" || ty == b"IEND" {
            return false;
        }
        offset = offset.saturating_add(12).saturating_add(length);
    }
    false
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let text = std::str::from_utf8(bytes).unwrap_or("");
    let trimmed = text.trim_start_matches(['\u{feff}', ' ', '\n', '\r', '\t']);
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("<?xml") && lower.contains("<svg") || lower.starts_with("<svg")
}

pub fn sanitize_svg(input: &str) -> Result<String, AvProcessFailure> {
    if !looks_like_svg(input.as_bytes()) {
        return Err(AvProcessFailure::InputInvalid);
    }
    let lower = input.to_ascii_lowercase();
    const FORBIDDEN_TAGS: &[&str] = &[
        "<script",
        "<foreignobject",
        "<iframe",
        "<embed",
        "<object",
        "<applet",
        "<animate",
        "<set",
        "<handler",
        "<listener",
        "<video",
        "<audio",
        "<use",
    ];
    if FORBIDDEN_TAGS.iter().any(|tag| lower.contains(tag)) {
        return Err(AvProcessFailure::InputInvalid);
    }
    if lower.contains("javascript:") || lower.contains("data:text/html") {
        return Err(AvProcessFailure::InputInvalid);
    }
    let mut sanitized = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(idx) = rest.find('<') {
        sanitized.push_str(&rest[..idx]);
        let after = &rest[idx..];
        let end = after.find('>').map(|n| n + 1).unwrap_or(after.len());
        let tag = &after[..end];
        sanitized.push_str(&strip_unsafe_attributes(tag)?);
        rest = &after[end..];
    }
    sanitized.push_str(rest);
    Ok(sanitized)
}

fn strip_unsafe_attributes(tag: &str) -> Result<String, AvProcessFailure> {
    let mut out = String::with_capacity(tag.len());
    let mut chars = tag.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_whitespace() {
            out.push(ch);
            let mut name = String::new();
            while let Some(next) = chars.peek() {
                if next.is_ascii_alphanumeric() || *next == ':' || *next == '-' || *next == '_' {
                    name.push(*next);
                    chars.next();
                } else {
                    break;
                }
            }
            if name.is_empty() {
                continue;
            }
            let lowered = name.to_ascii_lowercase();
            let mut value = String::new();
            let mut had_value = false;
            while matches!(chars.peek(), Some(c) if c.is_ascii_whitespace()) {
                chars.next();
            }
            if chars.peek() == Some(&'=') {
                had_value = true;
                chars.next();
                while matches!(chars.peek(), Some(c) if c.is_ascii_whitespace()) {
                    chars.next();
                }
                let quote = match chars.peek() {
                    Some(&c) if c == '"' || c == '\'' => {
                        let q = c;
                        chars.next();
                        Some(q)
                    }
                    _ => None,
                };
                if let Some(q) = quote {
                    for c in chars.by_ref() {
                        if c == q {
                            break;
                        }
                        value.push(c);
                    }
                } else {
                    while let Some(c) = chars.peek() {
                        if c.is_ascii_whitespace() || *c == '>' || *c == '/' {
                            break;
                        }
                        value.push(*c);
                        chars.next();
                    }
                }
            }
            if lowered.starts_with("on") {
                continue;
            }
            if matches!(lowered.as_str(), "href" | "xlink:href" | "src") {
                let v = value.trim().to_ascii_lowercase();
                if v.starts_with("javascript:")
                    || v.starts_with("http:")
                    || v.starts_with("https:")
                    || v.starts_with("data:text/html")
                    || v.starts_with("//")
                {
                    return Err(AvProcessFailure::InputInvalid);
                }
            }
            out.push_str(&name);
            if had_value {
                out.push_str("=\"");
                out.push_str(&value.replace('"', "&quot;"));
                out.push('"');
            }
            continue;
        }
        out.push(ch);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{Object, Stream, dictionary};

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn detects_png_jpeg_pdf_and_rejects_gif_webp() {
        assert_eq!(
            detect_deck_source(TINY_PNG).unwrap(),
            DetectedDeckSource::Png
        );
        assert_eq!(
            detect_deck_source(&[0xFF, 0xD8, 0xFF, 0xE0]).unwrap(),
            DetectedDeckSource::Jpeg
        );
        assert_eq!(
            detect_deck_source(b"%PDF-1.4\n").unwrap(),
            DetectedDeckSource::Pdf
        );
        assert!(matches!(
            detect_deck_source(b"GIF89a"),
            Err(AvProcessFailure::InputUnsupported)
        ));
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&[0, 0, 0, 0]);
        webp.extend_from_slice(b"WEBP");
        assert!(matches!(
            detect_deck_source(&webp),
            Err(AvProcessFailure::InputUnsupported)
        ));
    }

    #[test]
    fn rejects_apng_actl_chunk() {
        let mut apng = TINY_PNG.to_vec();
        apng.splice(
            8..8,
            [
                0x00, 0x00, 0x00, 0x08, b'a', b'c', b'T', b'L', 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        );
        assert!(matches!(
            detect_deck_source(&apng),
            Err(AvProcessFailure::InputUnsupported)
        ));
    }

    #[test]
    fn sanitizes_safe_svg_and_rejects_attacks() {
        let safe = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="1" height="1"/></svg>"#;
        assert!(sanitize_svg(safe).unwrap().contains("<rect"));
        assert!(sanitize_svg(r#"<svg><script>alert(1)</script></svg>"#).is_err());
        assert!(
            sanitize_svg(r#"<svg onload="alert(1)"></svg>"#)
                .unwrap()
                .contains("<svg")
        );
        assert!(
            !sanitize_svg(r#"<svg onload="alert(1)"></svg>"#)
                .unwrap()
                .to_ascii_lowercase()
                .contains("onload")
        );
        assert!(sanitize_svg(r#"<svg><a href="javascript:alert(1)">x</a></svg>"#).is_err());
        assert!(sanitize_svg(r#"<svg><image href="https://evil.example/x.png"/></svg>"#).is_err());
        assert!(sanitize_svg(r#"<svg><foreignObject></foreignObject></svg>"#).is_err());
    }

    #[test]
    fn splits_pdf_into_valid_single_page_documents() {
        let temp = tempfile::tempdir().unwrap();
        let input = temp.path().join("deck.pdf");
        let mut document = Document::with_version("1.5");
        let pages_id = document.new_object_id();
        let mut page_ids = Vec::new();
        for _ in 0..3 {
            let content_id = document.add_object(Stream::new(dictionary! {}, Vec::new()));
            let page_id = document.add_object(dictionary! {
                "Type" => "Page",
                "Parent" => pages_id,
                "MediaBox" => vec![0.into(), 0.into(), 100.into(), 100.into()],
                "Contents" => content_id,
                "Resources" => dictionary! {},
            });
            page_ids.push(page_id);
        }
        document.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
                "Count" => page_ids.len() as i64,
            }),
        );
        let catalog_id = document.add_object(dictionary! {
            "Type" => "Catalog",
            "Pages" => pages_id,
        });
        document.trailer.set("Root", catalog_id);
        document.save(&input).unwrap();
        assert!(matches!(
            expand_pdf(&input, temp.path(), 2),
            Err(AvProcessFailure::InputUnsupported)
        ));
        let expanded = expand_pdf(&input, temp.path(), 3).unwrap();

        assert_eq!(expanded.pages.len(), 3);
        for page in expanded.pages {
            let split = Document::load(&page.path).unwrap();
            assert_eq!(split.get_pages().len(), 1);
        }
    }
}
