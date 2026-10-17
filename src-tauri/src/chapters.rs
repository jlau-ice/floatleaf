use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub title: String,
    pub anchor: usize,
    pub level: usize,
}

pub fn detect(text: &str) -> Vec<Chapter> {
    static HEADING: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(concat!(
            r"(?i)^(?:第[\s0-9０-９〇零一二三四五六七八九十百千万亿两壹贰叁肆伍陆柒捌玖拾佰仟]+[章回节卷部篇].*",
            r"|(?:chapter|part|book)\s+(?:\d+|[ivxlcdm]+)(?:\b.*)",
            r"|(?:序章|序言|序幕|楔子|引子|前言|尾声|终章|后记|番外|prologue|epilogue)(?:\s.*|[：:].*)?)$"
        )).unwrap()
    });
    let mut chapters = Vec::new();
    let mut anchor = 0;
    for line in text.split_inclusive('\n') {
        let title = line.trim().trim_start_matches('#').trim();
        if title.chars().count() <= 80
            && !title.ends_with(['。', '！', '？', ';', '；'])
            && HEADING.is_match(title)
        {
            chapters.push(Chapter {
                title: title.to_owned(),
                anchor,
                level: 0,
            });
        }
        anchor += line.encode_utf16().count();
    }
    if chapters.first().is_some_and(|chapter| chapter.anchor > 0) {
        chapters.insert(
            0,
            Chapter {
                title: "开篇".into(),
                anchor: 0,
                level: 0,
            },
        );
    }
    chapters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_preserve_utf16_offsets_and_ignore_prose() {
        let text =
            "说明😀\n第一章 起风\n他说第一章很好看。\n  第二十二回 回家\nChapter IV — Home\n后记\n";
        let chapters = detect(text);
        assert_eq!(
            chapters
                .iter()
                .map(|c| c.title.as_str())
                .collect::<Vec<_>>(),
            [
                "开篇",
                "第一章 起风",
                "第二十二回 回家",
                "Chapter IV — Home",
                "后记"
            ]
        );
        assert_eq!(chapters[1].anchor, 5);
        let start = text.find("  第二").unwrap();
        assert_eq!(chapters[2].anchor, text[..start].encode_utf16().count());
        assert!(detect("没有标题的普通段落。\n这是第一章的内容。\n").is_empty());
        assert_eq!(detect("第１章 开始\n正文\n第两百章 归来").len(), 2);
    }
}
