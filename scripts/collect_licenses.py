"""Collect license texts for the union of locked Linux and Windows dependencies."""
import json
from pathlib import Path
import subprocess
import urllib.request
import urllib.error

metadata = json.loads(subprocess.check_output(["cargo", "metadata", "--locked", "--format-version", "1"]))
active = set()
for target in ("x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"):
    tree = subprocess.check_output(["cargo", "tree", "--locked", "--target", target, "--prefix", "none", "--format", "{p}"], text=True)
    active.update(tuple(line.split()[:2]) for line in tree.splitlines())
fetched_text = {}
sections = ["WinLLDP — dependency licenses for Linux and Windows x64. Versions are locked in Cargo.lock.\n"]
for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
    if (package["name"], "v" + package["version"]) not in active or package["name"] == "winlldp":
        continue
    root = Path(package["manifest_path"]).parent
    files = set()
    for path in root.rglob("*"):
        if not path.is_file() or len(path.relative_to(root).parts) > 4:
            continue
        name = path.name.upper()
        if any(part in name for part in ("LICENSE", "LICENCE", "COPYING", "NOTICE")) or name in ("OFL.TXT", "UFL.TXT", "HACK-REGULAR.TXT"):
            files.add(path)
    sections.append(f"\n{'=' * 72}\n{package['name']} {package['version']}\nSPDX: {package.get('license') or 'see source'}\nSource: {package.get('repository') or package.get('source') or ''}\n")
    for path in sorted(files):
        try:
            text = path.read_text(encoding="utf-8")
        except (UnicodeError, OSError):
            continue
        sections.append(f"\n--- {path.relative_to(root)} ---\n{text}\n")
    if not files and package.get("license") and "MIT" in package["license"]:
        sections.append("\n--- SPDX MIT (package metadata: " + package["license"] + ") ---\n" + Path("scripts/licenses/MIT.txt").read_text(encoding="utf-8") + "\n")
        continue
    if not files:
        vcs = root / ".cargo_vcs_info.json"
        repository = (package.get("repository") or "").removesuffix(".git").rstrip("/")
        if not vcs.is_file() or not repository.startswith("https://github.com/"):
            raise SystemExit(f"Missing license provenance: {package['name']} {package['version']}")
        repository = "/".join(repository.split("/")[:5])
        sha = json.loads(vcs.read_text())["git"]["sha1"]
        base = repository.replace("https://github.com/", "https://raw.githubusercontent.com/") + "/" + sha + "/"
        found = 0
        for name in ("LICENSE", "LICENSE-MIT", "LICENSE-APACHE", "LICENSE.md", "LICENSE.txt", "LICENSE-MIT.txt", "LICENSE-APACHE.txt"):
            url = base + name
            if url not in fetched_text:
                try:
                    fetched_text[url] = urllib.request.urlopen(url, timeout=20).read().decode("utf-8")
                except (urllib.error.URLError, UnicodeError):
                    fetched_text[url] = None
            if fetched_text[url] is not None:
                sections.append(f"\n--- {url} ---\n{fetched_text[url]}\n")
                found += 1
        if not found:
            raise SystemExit(f"Missing upstream license: {package['name']} {package['version']} at {base}")
Path("THIRD_PARTY_LICENSES.txt").write_text("".join(sections), encoding="utf-8")
print("Collected", len(active), "active dependency license sections")
