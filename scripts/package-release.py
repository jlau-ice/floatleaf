"""将两端 CI 安装包、试用说明和校验清单整理为 Release 附件。"""

import hashlib
import json
import shutil
import sys
import tomllib
import zipfile
from pathlib import Path

tag, installer_directory, output_directory = sys.argv[1:]
root = Path(__file__).resolve().parent.parent
version = json.loads((root / "package.json").read_text())["version"]
assert tag == f"v{version}", "版本标签必须与 package.json 一致"
assert json.loads((root / "src-tauri/tauri.conf.json").read_text())["version"] == version
assert tomllib.loads((root / "src-tauri/Cargo.toml").read_text())["package"]["version"] == version
lock = tomllib.loads((root / "src-tauri/Cargo.lock").read_text())
assert next(p["version"] for p in lock["package"] if p["name"] == "floatleaf") == version

output = Path(output_directory)
output.mkdir(parents=True, exist_ok=True)
installer_names = [f"FloatLeaf_{version}_aarch64.dmg", f"FloatLeaf_{version}_x64-setup.exe"]
for name in installer_names:
    matches = list(Path(installer_directory).rglob(name))
    assert len(matches) == 1, f"需要唯一安装包：{name}"
    shutil.copyfile(matches[0], output / name)
shutil.copyfile(root / "LICENSE", output / "LICENSE.txt")
shutil.copyfile(root / f"docs/RELEASE_{tag}.md", output / "Retest-Guide.txt")

exe = output / installer_names[1]
exe_checksum = f"{hashlib.sha256(exe.read_bytes()).hexdigest()}  {exe.name}\n"
with zipfile.ZipFile(output / f"FloatLeaf_{version}_windows-x64_trial.zip", "w", zipfile.ZIP_DEFLATED) as archive:
    for name in [exe.name, "LICENSE.txt", "Retest-Guide.txt"]:
        archive.write(output / name, name)
    archive.writestr("SHA256SUMS.txt", exe_checksum)

checksums = "".join(
    f"{hashlib.sha256(file.read_bytes()).hexdigest()}  {file.name}\n"
    for file in sorted(output.iterdir()) if file.name != "SHA256SUMS.txt"
)
(output / "SHA256SUMS.txt").write_text(checksums)
print(checksums, end="")
