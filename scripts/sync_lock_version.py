"""Keep the root package's Cargo.lock version in sync with Cargo.toml."""
import re
import tomllib
from pathlib import Path

manifest = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]
lock_path = Path("Cargo.lock")
lock = lock_path.read_text(encoding="utf-8")
pattern = re.compile(
    rf'(?m)(\[\[package\]\]\nname = "{re.escape(manifest["name"])}"\nversion = ")[^"]+("\n)'
)
updated, count = pattern.subn(rf"\g<1>{manifest['version']}\g<2>", lock, count=1)
if count != 1:
    raise SystemExit(f"Could not find exactly one root package entry for {manifest['name']} in Cargo.lock")
lock_path.write_text(updated, encoding="utf-8")
