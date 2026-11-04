"""Create Windows and UI icon files from the embedded PNG in the source SVG.

Requires Pillow. Run from any directory: python scripts/generate_icons.py
The SVG is a raster container, not a hand-drawn vector asset.
"""
import base64
import io
from pathlib import Path
import xml.etree.ElementTree as ET

from PIL import Image

assets = Path(__file__).resolve().parents[1] / "assets"
root = ET.parse(assets / "winlldp-otter.svg").getroot()
element = root.find("{http://www.w3.org/2000/svg}image")
if element is None:
    raise ValueError("Source image missing")
href = element.attrib["href"]
prefix = "data:image/png;base64,"
if not href.startswith(prefix):
    raise ValueError("Expected embedded PNG")
png = base64.b64decode(href[len(prefix):], validate=True)
with Image.open(io.BytesIO(png)) as source:
    image = source.convert("RGBA")
image.save(assets / "winlldp-otter.png")
sizes = [(n, n) for n in (16, 20, 24, 32, 40, 48, 64, 128, 256)]
image.save(assets / "winlldp.ico", sizes=sizes)
for size in (24, 32, 48, 64):
    image.resize((size, size), Image.Resampling.LANCZOS).save(
        assets / f"winlldp-otter-{size}.png"
    )
with Image.open(assets / "winlldp.ico") as icon:
    if icon.ico.sizes() != set(sizes):
        raise ValueError("Incomplete icon sizes")
print("Created Windows ICO and UI PNGs in", assets)
