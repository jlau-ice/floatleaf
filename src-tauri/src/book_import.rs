use crate::{
    chapters::{self, Chapter},
    epub_import, text_import,
};
use serde::Serialize;
use std::path::Path;

pub struct ImportedBook {
    pub title: String,
    pub text: String,
    pub encoding: String,
    pub chapters: Vec<Chapter>,
    pub warning: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub title: String,
    pub encoding: String,
    pub preview: String,
    pub warning: Option<String>,
    pub utf16_length: usize,
    pub chapter_count: usize,
}

pub fn read(path: &Path, encoding: &str) -> Result<ImportedBook, String> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("txt") => {
            let (text, encoding, warning) =
                text_import::decode(&text_import::read_source(path)?, encoding)?;
            Ok(ImportedBook {
                title: text_import::title(path),
                chapters: chapters::detect(&text),
                text,
                encoding,
                warning,
            })
        }
        Some("epub") => epub_import::read(path),
        _ => Err("支持 TXT 和 EPUB，请选择对应文件。其他格式请先转换为 EPUB。".into()),
    }
}

pub fn preview(path: &Path, encoding: &str) -> Result<ImportPreview, String> {
    let book = read(path, encoding)?;
    Ok(ImportPreview {
        title: book.title,
        encoding: book.encoding,
        preview: book.text.chars().take(1800).collect(),
        warning: book.warning,
        utf16_length: book.text.encode_utf16().count(),
        chapter_count: book.chapters.len(),
    })
}
