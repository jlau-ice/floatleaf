use std::{fs, io::Read, path::Path};
const MAX_BOOK_BYTES: u64 = 20 * 1024 * 1024;

pub fn title(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .chars()
        .filter(|c| !c.is_control())
        .take(160)
        .collect()
}

pub fn read_source(path: &Path) -> Result<Vec<u8>, String> {
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("txt"))
    {
        return Err("首版仅支持 .txt 文件".into());
    }
    let mut file = fs::File::open(path).map_err(|e| format!("无法读取文件：{e}"))?;
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Err("请选择普通 TXT 文件".into());
    }
    if metadata.len() > MAX_BOOK_BYTES {
        return Err("文件超过 20 MiB，请先拆分后导入".into());
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_BOOK_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BOOK_BYTES {
        return Err("文件在读取期间变大，已取消导入".into());
    }
    Ok(bytes)
}

pub fn decode(bytes: &[u8], requested: &str) -> Result<(String, String, Option<String>), String> {
    if bytes.is_empty() {
        return Err("文件为空，没有可阅读的内容".into());
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        return Err("检测到 UTF-16，请先另存为 UTF-8 TXT 后导入".into());
    }
    let bytes = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
    let (text, encoding, mut warning) = match requested {
        "auto" | "utf8" => match std::str::from_utf8(bytes) {
            Ok(text) => (text.to_owned(), "UTF-8", None),
            Err(_) if requested == "utf8" => {
                return Err("文件不是有效 UTF-8，请改选 GBK 并核对预览".into());
            }
            Err(_) => {
                let (text, errors) = encoding_rs::GBK.decode_without_bom_handling(bytes);
                if errors {
                    return Err("UTF-8 和 GBK 都不能完整解码，请检查文件编码或另存为 UTF-8".into());
                }
                (
                    text.into_owned(),
                    "GBK",
                    Some("已按 GBK 解码。自动检测不能保证文字正确，请核对预览后再导入。".into()),
                )
            }
        },
        "gbk" => {
            let (text, errors) = encoding_rs::GBK.decode_without_bom_handling(bytes);
            if errors {
                return Err("GBK 解码存在无效字节，请检查源文件或改选 UTF-8".into());
            }
            (
                text.into_owned(),
                "GBK",
                Some("已手动选择 GBK，请核对预览中的文字。".into()),
            )
        }
        _ => return Err("不支持的编码选项".into()),
    };
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    if text.trim().is_empty() {
        return Err("文件只有空白，没有可阅读的内容".into());
    }
    if text
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\t')
    {
        return Err("文件包含非文本控制字符，可能不是普通 TXT 或编码不正确".into());
    }
    if text.contains('\u{fffd}') {
        warning = Some("内容含有替换字符 �，源文件可能已有乱码，请核对后再导入。".into());
    }
    Ok((text, encoding.into(), warning))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoding_reports_gbk_and_rejects_unreadable_files() {
        assert_eq!(
            decode(b"\xef\xbb\xbfhello\r\nworld\r", "auto").unwrap().0,
            "hello\nworld\n"
        );
        let (bytes, _, _) = encoding_rs::GBK.encode("窗外的风，吹过书页。\r\n第二行");
        let decoded = decode(&bytes, "auto").unwrap();
        assert_eq!(decoded.1, "GBK");
        assert!(decoded.2.is_some());
        assert!(!decoded.0.contains('\r'));
        assert!(decode(&bytes, "utf8").is_err());
        for input in [
            &b""[..],
            &b" \r\n\t"[..],
            &b"binary\x00data"[..],
            &b"\xff\xfea\x00"[..],
            &b"\x81"[..],
        ] {
            assert!(decode(input, "auto").is_err());
        }
        assert!(decode("已有�乱码".as_bytes(), "auto").unwrap().2.is_some());
    }
}
