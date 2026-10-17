use crate::{book_import::ImportedBook, chapters::Chapter, text_import};
use percent_encoding::percent_decode_str;
use roxmltree::Document;
use scraper::{ElementRef, Html, Selector};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::Path,
};
use zip::ZipArchive;

const MAX_RESOURCE: u64 = 20 * 1024 * 1024;
const MAX_TEXT: usize = 20 * 1024 * 1024;

fn resource(archive: &mut ZipArchive<File>, path: &str) -> Result<String, String> {
    let entry = archive
        .by_name(path)
        .map_err(|e| format!("无法读取 EPUB 内的 {path}：{e}"))?;
    let mut bytes = Vec::new();
    entry
        .take(MAX_RESOURCE + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_RESOURCE {
        return Err("EPUB 单个正文或目录文件超过 20 MiB，无法导入".into());
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let (text, _, errors) = encoding_rs::UTF_16LE.decode(&bytes);
        return if errors {
            Err("EPUB 内的 UTF-16 文本损坏".into())
        } else {
            Ok(text.into_owned())
        };
    }
    String::from_utf8(bytes)
        .map(|s| s.trim_start_matches('\u{feff}').to_owned())
        .map_err(|_| "EPUB 内的文字不是有效 UTF-8/UTF-16，可能已损坏或加密".into())
}

// Resolve only archive references; EPUB content never fetches URLs or extracts files to disk.
fn target(base: &str, href: &str) -> Option<(String, Option<String>)> {
    let root = url::Url::parse("https://epub.invalid/").unwrap();
    let url = root.join(base).ok()?.join(href).ok()?;
    if url.origin() != root.origin() {
        return None;
    }
    let path = percent_decode_str(url.path().trim_start_matches('/'))
        .decode_utf8()
        .ok()?
        .into_owned();
    let fragment = url
        .fragment()
        .map(|s| percent_decode_str(s).decode_utf8().map(|s| s.into_owned()))
        .transpose()
        .ok()?;
    Some((path, fragment))
}

#[derive(Default)]
struct Body {
    text: String,
    units: usize,
    anchors: HashMap<String, usize>,
    heading: String,
}
impl Body {
    fn newline(&mut self) {
        if self.text.ends_with(' ') {
            self.text.pop();
            self.units -= 1;
        }
        if !self.text.is_empty() && !self.text.ends_with('\n') {
            self.text.push('\n');
            self.units += 1;
        }
    }
    fn text(&mut self, value: &str) {
        for ch in value.chars() {
            if ch.is_whitespace() {
                if !self.text.is_empty() && !self.text.ends_with([' ', '\n']) {
                    self.text.push(' ');
                    self.units += 1;
                }
            } else {
                self.text.push(ch);
                self.units += ch.len_utf16();
            }
        }
    }
    fn element(&mut self, element: ElementRef<'_>, depth: usize) -> Result<(), String> {
        if depth > 256 {
            return Err("EPUB 正文嵌套过深，无法解析".into());
        }
        let name = element.value().name();
        if matches!(
            name,
            "head" | "script" | "style" | "nav" | "svg" | "iframe" | "object" | "rt" | "rp"
        ) || element.value().attr("hidden").is_some()
            || element.value().attr("aria-hidden") == Some("true")
        {
            return Ok(());
        }
        let block = matches!(
            name,
            "p" | "div"
                | "section"
                | "article"
                | "h1"
                | "h2"
                | "h3"
                | "h4"
                | "h5"
                | "h6"
                | "li"
                | "blockquote"
                | "pre"
                | "tr"
                | "br"
                | "hr"
        );
        if block {
            self.newline();
        }
        for attr in ["id", "name"] {
            if let Some(id) = element.value().attr(attr) {
                self.anchors.entry(id.to_owned()).or_insert(self.units);
            }
        }
        if self.heading.is_empty() && matches!(name, "h1" | "h2") {
            self.heading = element
                .text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
        }
        for child in element.children() {
            if let Some(text) = child.value().as_text() {
                self.text(text);
            } else if let Some(child) = ElementRef::wrap(child) {
                self.element(child, depth + 1)?;
            }
        }
        if block {
            self.newline();
        }
        Ok(())
    }
}

struct Link {
    title: String,
    path: String,
    fragment: Option<String>,
    level: usize,
}

fn navigation(source: &str, path: &str) -> Vec<Link> {
    let html = Html::parse_document(source);
    let nav_selector = Selector::parse("nav").unwrap();
    let link_selector = Selector::parse("a[href]").unwrap();
    let nav = html.select(&nav_selector).find(|n| {
        n.value()
            .attr("epub:type")
            .is_some_and(|t| t.split_whitespace().any(|t| t == "toc"))
            || n.value().attr("role") == Some("doc-toc")
    });
    nav.into_iter()
        .flat_map(|nav| nav.select(&link_selector))
        .filter_map(|link| {
            let (path, fragment) = target(path, link.value().attr("href")?)?;
            let title = link
                .text()
                .collect::<String>()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            let level = link
                .ancestors()
                .filter_map(ElementRef::wrap)
                .filter(|e| e.value().name() == "ol")
                .count()
                .saturating_sub(1);
            (!title.is_empty()).then_some(Link {
                title,
                path,
                fragment,
                level,
            })
        })
        .collect()
}

fn ncx(source: &str, path: &str) -> Result<Vec<Link>, String> {
    let xml = Document::parse(source).map_err(|e| format!("EPUB 旧版目录损坏：{e}"))?;
    Ok(xml
        .descendants()
        .filter(|n| n.has_tag_name("navPoint"))
        .filter_map(|point| {
            let href = point
                .children()
                .find(|n| n.has_tag_name("content"))?
                .attribute("src")?;
            let (path, fragment) = target(path, href)?;
            let label = point.children().find(|n| n.has_tag_name("navLabel"))?;
            let title = label
                .descendants()
                .filter(|n| n.is_text())
                .filter_map(|n| n.text())
                .collect::<String>()
                .trim()
                .to_owned();
            let level = point
                .ancestors()
                .skip(1)
                .filter(|n| n.has_tag_name("navPoint"))
                .count();
            (!title.is_empty()).then_some(Link {
                title,
                path,
                fragment,
                level,
            })
        })
        .collect())
}

pub fn read(path: &Path) -> Result<ImportedBook, String> {
    let file = File::open(path).map_err(|e| format!("无法读取 EPUB：{e}"))?;
    if file.metadata().map_err(|e| e.to_string())?.len() > 100 * 1024 * 1024 {
        return Err("EPUB 文件超过 100 MiB，无法导入".into());
    }
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("EPUB 容器损坏或格式不正确：{e}"))?;
    let mut encrypted = HashSet::new();
    if archive
        .file_names()
        .any(|name| name.is_ok_and(|name| name == "META-INF/encryption.xml"))
    {
        let source = resource(&mut archive, "META-INF/encryption.xml")?;
        let xml = Document::parse(&source).map_err(|e| format!("EPUB 加密描述损坏：{e}"))?;
        for item in xml
            .descendants()
            .filter(|n| n.has_tag_name("CipherReference"))
        {
            if let Some((path, _)) = item.attribute("URI").and_then(|uri| target("", uri)) {
                encrypted.insert(path);
            }
        }
    }
    let container = resource(&mut archive, "META-INF/container.xml")?;
    let xml = Document::parse(&container).map_err(|e| format!("EPUB 容器描述损坏：{e}"))?;
    let opf_path = xml
        .descendants()
        .find(|n| n.has_tag_name("rootfile"))
        .and_then(|n| n.attribute("full-path"))
        .ok_or("EPUB 缺少正文索引")?;
    let opf = resource(&mut archive, opf_path)?;
    let package = Document::parse(&opf).map_err(|e| format!("EPUB 正文索引损坏：{e}"))?;
    let title = package
        .descendants()
        .find(|n| n.has_tag_name("title"))
        .and_then(|n| n.text())
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| text_import::title(path));
    let manifest: HashMap<_, _> = package
        .descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|item| Some((item.attribute("id")?, item)))
        .collect();
    let spine = package
        .descendants()
        .find(|n| n.has_tag_name("spine"))
        .ok_or("EPUB 缺少阅读顺序")?;
    let mut warning = None;
    let nav = manifest.values().find(|n| {
        n.attribute("properties")
            .is_some_and(|s| s.split_whitespace().any(|v| v == "nav"))
    });
    let links = if let Some(nav) = nav {
        let (path, _) = target(opf_path, nav.attribute("href").ok_or("EPUB 目录缺少路径")?)
            .ok_or("EPUB 目录路径无效")?;
        navigation(&resource(&mut archive, &path)?, &path)
    } else if let Some(item) = spine
        .attribute("toc")
        .and_then(|id| manifest.get(id))
        .or_else(|| {
            manifest
                .values()
                .find(|n| n.attribute("media-type") == Some("application/x-dtbncx+xml"))
        })
    {
        let (path, _) = target(opf_path, item.attribute("href").ok_or("EPUB 目录缺少路径")?)
            .ok_or("EPUB 目录路径无效")?;
        ncx(&resource(&mut archive, &path)?, &path)?
    } else {
        Vec::new()
    };
    let mut text = String::new();
    let mut units = 0;
    let mut chapters = Vec::new();
    let mut locations = HashMap::new();
    for item in spine
        .children()
        .filter(|n| n.has_tag_name("itemref") && n.attribute("linear") != Some("no"))
    {
        let entry = item
            .attribute("idref")
            .and_then(|id| manifest.get(id))
            .ok_or("EPUB 阅读顺序引用了不存在的正文")?;
        if entry
            .attribute("properties")
            .is_some_and(|s| s.split_whitespace().any(|p| p == "nav"))
        {
            continue;
        }
        let (path, _) = target(
            opf_path,
            entry.attribute("href").ok_or("EPUB 正文缺少路径")?,
        )
        .ok_or("EPUB 正文路径无效")?;
        if encrypted.contains(&path) {
            return Err("EPUB 正文已加密，无法导入。请选择未加密的 EPUB。".into());
        }
        let source = resource(&mut archive, &path)?;
        let html = Html::parse_document(&source);
        let mut body = Body::default();
        if let Some(element) = html.select(&Selector::parse("body").unwrap()).next() {
            body.element(element, 0)?;
        }
        if body.text.trim().is_empty() {
            continue;
        }
        if text.len() + body.text.len() + 1 > MAX_TEXT {
            return Err("EPUB 提取的正文超过 20 MiB，无法导入".into());
        }
        if !text.is_empty() {
            text.push('\n');
            units += 1;
        }
        let base = units;
        let fallback = if body.heading.is_empty() {
            format!("第 {} 节", locations.len() + 1)
        } else {
            body.heading
        };
        chapters.push(Chapter {
            title: fallback,
            anchor: base,
            level: 0,
        });
        units += body.units;
        text.push_str(&body.text);
        locations.insert(path, (base, body.anchors));
    }
    if text.trim().is_empty() {
        return Err("EPUB 没有可提取的文字，可能是扫描图书或加密文件".into());
    }
    let mut resolved = Vec::new();
    for link in &links {
        if let Some((base, anchors)) = locations.get(&link.path) {
            let local = match &link.fragment {
                Some(id) if !id.is_empty() => anchors.get(id).copied(),
                _ => Some(0),
            };
            if let Some(local) = local {
                resolved.push(Chapter {
                    title: link.title.clone(),
                    anchor: base + local,
                    level: link.level,
                });
            } else {
                warning = Some("部分目录锚点不存在，已保留可定位的章节与正文分节。".into());
            }
        }
    }
    if !resolved.is_empty() {
        chapters.retain(|c| !resolved.iter().any(|r| r.anchor == c.anchor));
        chapters.extend(resolved);
    } else {
        warning = Some("未找到可用目录，已按正文文件顺序生成分节。".into());
    }
    chapters.sort_by_key(|c| c.anchor);
    chapters.dedup_by(|a, b| a.anchor == b.anchor && a.title == b.title);
    chapters.retain(|c| c.anchor < units);
    Ok(ImportedBook {
        title,
        text,
        encoding: "EPUB".into(),
        chapters,
        warning,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::io::Write;
    use zip::{ZipWriter, write::SimpleFileOptions};

    pub fn fixture(path: &Path, legacy: bool) {
        let navigation = if legacy {
            r#"<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/"><navMap><navPoint><navLabel><text>第一章 出发</text></navLabel><content src="text/one.xhtml#start"/><navPoint><navLabel><text>途中😀</text></navLabel><content src="text/one.xhtml#%E4%B8%AD"/></navPoint></navPoint><navPoint><navLabel><text>第二章 归来</text></navLabel><content src="text/two%20end.xhtml#end"/></navPoint></navMap></ncx>"#
        } else {
            r#"<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol><li><a href="text/one.xhtml#start">第一章 出发</a><ol><li><a href="text/one.xhtml#%E4%B8%AD">途中😀</a></li></ol></li><li><a href="text/two%20end.xhtml#end">第二章 归来</a></li></ol></nav></body></html>"#
        };
        let nav_item = if legacy {
            r#"id="toc" href="nav.xml" media-type="application/x-dtbncx+xml""#
        } else {
            r#"id="toc" href="nav.xml" media-type="application/xhtml+xml" properties="nav""#
        };
        let package = format!(
            r#"<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/"><metadata><dc:title>合成章节验收</dc:title></metadata><manifest><item id="second" href="text/two%20end.xhtml" media-type="application/xhtml+xml"/><item {nav_item}/><item id="first" href="text/one.xhtml" media-type="application/xhtml+xml"/></manifest><spine toc="toc"><itemref idref="first"/><itemref idref="second"/></spine></package>"#
        );
        write(
            path,
            &[
                ("mimetype", "application/epub+zip"),
                (
                    "META-INF/container.xml",
                    r#"<container><rootfiles><rootfile full-path="OPS/package.opf"/></rootfiles></container>"#,
                ),
                ("OPS/package.opf", &package),
                ("OPS/nav.xml", navigation),
                (
                    "OPS/text/one.xhtml",
                    r#"<html><head><title>NOT_BODY</title></head><body><h1 id="start">出发</h1><p>你好😀 &amp; 世界</p><h2 id="中">途中</h2><p>文字<strong>仍然连续</strong>。</p><script>BAD_SCRIPT</script><style>BAD_STYLE</style><p hidden="hidden">BAD_HIDDEN</p></body></html>"#,
                ),
                (
                    "OPS/text/two end.xhtml",
                    r#"<html><body><h1 id="end">归来</h1><p>第二个正文文件。</p></body></html>"#,
                ),
            ],
        );
    }
    fn write(path: &Path, entries: &[(&str, &str)]) {
        let mut zip = ZipWriter::new(File::create(path).unwrap());
        for (name, text) in entries {
            zip.start_file(
                name,
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
            zip.write_all(text.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
    }
    #[test]
    fn epub2_and_epub3_preserve_spine_nested_toc_and_fragment_offsets() {
        let temp = tempfile::tempdir().unwrap();
        for legacy in [false, true] {
            let path = temp.path().join("book.epub");
            fixture(&path, legacy);
            let book = read(&path).unwrap();
            assert_eq!(book.title, "合成章节验收");
            assert_eq!(book.chapters.len(), 3);
            assert_eq!(book.chapters[1].title, "途中😀");
            assert_eq!(book.chapters[1].level, 1);
            assert_eq!(book.chapters[0].anchor, 0);
            assert!(book.warning.is_none());
            let offset = book.text.find("途中").unwrap();
            assert_eq!(
                book.chapters[1].anchor,
                book.text[..offset].encode_utf16().count()
            );
            let offset = book.text.find("归来").unwrap();
            assert_eq!(
                book.chapters[2].anchor,
                book.text[..offset].encode_utf16().count()
            );
            assert!(book.text.contains("你好😀 & 世界\n"));
            assert!(book.text.contains("文字仍然连续。"));
            assert!(!book.text.contains("BAD_") && !book.text.contains("NOT_BODY"));
        }
    }
    #[test]
    fn missing_toc_uses_reading_order_and_bad_container_is_reported() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("plain.epub");
        write(
            &path,
            &[
                (
                    "META-INF/container.xml",
                    r#"<container><rootfile full-path="book.opf"/></container>"#,
                ),
                (
                    "book.opf",
                    r#"<package><manifest><item id="text" href="body.xhtml"/></manifest><spine><itemref idref="text"/></spine></package>"#,
                ),
                (
                    "body.xhtml",
                    "<html><body><h1>正文标题</h1><p>正文</p></body></html>",
                ),
            ],
        );
        let book = read(&path).unwrap();
        assert_eq!(book.title, "plain");
        assert_eq!(book.chapters[0].title, "正文标题");
        assert!(book.warning.unwrap().contains("按正文文件顺序"));
        std::fs::write(&path, b"not a zip").unwrap();
        assert!(read(&path).err().unwrap().contains("容器损坏"));
    }
    #[test]
    fn encrypted_body_and_oversized_expansion_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("encrypted.epub");
        write(
            &path,
            &[
                (
                    "META-INF/container.xml",
                    r#"<container><rootfile full-path="book.opf"/></container>"#,
                ),
                (
                    "META-INF/encryption.xml",
                    r#"<encryption><CipherReference URI="body.xhtml"/></encryption>"#,
                ),
                (
                    "book.opf",
                    r#"<package><manifest><item id="text" href="body.xhtml"/></manifest><spine><itemref idref="text"/></spine></package>"#,
                ),
                ("body.xhtml", "ENCRYPTED"),
            ],
        );
        assert!(read(&path).err().unwrap().contains("正文已加密"));
        write(
            &path,
            &[(
                "META-INF/container.xml",
                &"x".repeat(MAX_RESOURCE as usize + 1),
            )],
        );
        assert!(read(&path).err().unwrap().contains("超过 20 MiB"));
    }
    #[test]
    fn references_are_archive_relative_and_never_external() {
        assert_eq!(
            target("OPS/nav/toc.xhtml", "../text/a%20b.xhtml#%E7%AB%A0"),
            Some(("OPS/text/a b.xhtml".into(), Some("章".into())))
        );
        assert!(target("OPS/nav.xhtml", "https://example.com/book").is_none());
        assert!(target("OPS/nav.xhtml", "file:///private/file").is_none());
    }
}
