"""Render the rectangle-based brand SVG to Windows icons. Requires Pillow."""
from pathlib import Path
from xml.etree import ElementTree
from PIL import Image, ImageColor, ImageDraw

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "public" / "mirror-mark.svg"
SCALE = 32
canvas = Image.new("RGBA", (32 * SCALE, 32 * SCALE))


def color(value, opacity=1.0):
    if value in (None, "none"):
        return None
    return (*ImageColor.getrgb(value), round(float(opacity) * 255))


for node in ElementTree.parse(SOURCE).getroot():
    if node.tag.split("}")[-1] != "rect":
        raise ValueError("Icon renderer supports the brand's rectangle elements only")
    x, y, width, height, radius = [float(node.get(k, 0)) * SCALE for k in ("x", "y", "width", "height", "rx")]
    stroke_width = float(node.get("stroke-width", 0)) * SCALE
    half = stroke_width / 2
    layer = Image.new("RGBA", canvas.size)
    ImageDraw.Draw(layer).rounded_rectangle(
        (x - half, y - half, x + width + half, y + height + half),
        radius=radius + half,
        fill=color(node.get("fill")),
        outline=color(node.get("stroke"), node.get("stroke-opacity", 1)),
        width=max(1, round(stroke_width)),
    )
    canvas = Image.alpha_composite(canvas, layer)

icons = ROOT / "src-tauri" / "icons"
icons.mkdir(parents=True, exist_ok=True)
canvas = canvas.resize((256, 256), Image.Resampling.LANCZOS)
canvas.save(icons / "icon.png")
canvas.save(icons / "icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])
(ROOT / "public" / "mark.svg").write_bytes(SOURCE.read_bytes())
print("Generated matching application, tray and favicon assets.")
