"""Create original synthetic TXT/EPUB samples for chapter regression and trial packages."""
from pathlib import Path
import sys
import zipfile

directory = Path(sys.argv[1])
directory.mkdir(parents=True, exist_ok=True)
titles = ["第一章 出发", "第二章 山路", "第三章 归来"]
text = "\n".join(title + "\n" + ("合成测试文字，检查章节跳转与阅读进度。\n" * 30) for title in titles)
(directory / "chapters-utf8.txt").write_text(text, encoding="utf-8")
(directory / "chapters-gbk.txt").write_bytes(text.encode("gbk"))
for version in (2, 3):
    with zipfile.ZipFile(directory / f"chapters-epub{version}.epub", "w", compression=zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("mimetype", "application/epub+zip", compress_type=zipfile.ZIP_STORED)
        archive.writestr("META-INF/container.xml", '<container><rootfiles><rootfile full-path="OPS/book.opf"/></rootfiles></container>')
        nav_item = 'id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"' if version == 3 else 'id="nav" href="toc.ncx" media-type="application/x-dtbncx+xml"'
        archive.writestr("OPS/book.opf", f'''<package xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/" version="{version}.0">
<metadata><dc:title>EPUB {version} 章节合成示例</dc:title></metadata><manifest><item {nav_item}/><item id="body" href="text/body.xhtml" media-type="application/xhtml+xml"/></manifest><spine toc="nav"><itemref idref="body"/></spine></package>''')
        sections = ''.join(f'<h1 id="chapter{i}">{title}</h1>' + (f'<p>EPUB {version} 原创合成文本 😀，这是第 {i + 1} 章的测试段落。</p>' * 30) for i, title in enumerate(titles))
        archive.writestr("OPS/text/body.xhtml", '<html xmlns="http://www.w3.org/1999/xhtml"><body>' + sections + '</body></html>')
        if version == 3:
            links = ''.join(f'<li><a href="text/body.xhtml#chapter{i}">{title}</a></li>' for i, title in enumerate(titles))
            archive.writestr("OPS/nav.xhtml", '<html xmlns:epub="http://www.idpf.org/2007/ops"><body><nav epub:type="toc"><ol>' + links + '</ol></nav></body></html>')
        else:
            links = ''.join(f'<navPoint><navLabel><text>{title}</text></navLabel><content src="text/body.xhtml#chapter{i}"/></navPoint>' for i, title in enumerate(titles))
            archive.writestr("OPS/toc.ncx", '<ncx><navMap>' + links + '</navMap></ncx>')
print(f"Created 4 synthetic chapter samples in {directory}")
