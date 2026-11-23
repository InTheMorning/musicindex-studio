"""Make contact sheets of the walk screenshots, and crop them for the page.

Contact sheets: six shots on one image at half size, with the name of each
shot. Read the sheets, then write the crop boxes. A full shot is 1280 x 800,
so a box on a sheet is half of the box on the shot.

Usage:
  python3 -I tools/tutorial-walk/crop.py sheets OUT_FOLDER SHOT.png...
  python3 -I tools/tutorial-walk/crop.py crop CROPS.json IMG_FOLDER

CROPS.json maps a page image name to [shot path, [left, top, right, bottom]].
This script needs Pillow.
"""
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw


def sheets(out, shots):
    out = Path(out)
    out.mkdir(parents=True, exist_ok=True)
    for start in range(0, len(shots), 6):
        sheet = Image.new("RGB", (1280, 1200), "white")
        for i, shot in enumerate(shots[start:start + 6]):
            image = Image.open(shot).convert("RGB").resize((640, 400))
            draw = ImageDraw.Draw(image)
            draw.rectangle((0, 0, 300, 18), fill="yellow")
            draw.text((4, 3), Path(shot).parent.name + "/" + Path(shot).stem, fill="black")
            sheet.paste(image, ((i % 2) * 640, (i // 2) * 400))
        name = out / f"sheet{start // 6}.png"
        sheet.save(name)
        print("sheet:", name)


def crop(spec, folder):
    folder = Path(folder)
    folder.mkdir(parents=True, exist_ok=True)
    for name, (shot, box) in json.loads(Path(spec).read_text()).items():
        image = Image.open(shot).crop(tuple(box))
        image.save(folder / f"{name}.png", optimize=True)
        print("crop:", name, image.size)


if __name__ == "__main__":
    if len(sys.argv) >= 4 and sys.argv[1] == "sheets":
        sheets(sys.argv[2], sys.argv[3:])
    elif len(sys.argv) == 4 and sys.argv[1] == "crop":
        crop(sys.argv[2], sys.argv[3])
    else:
        sys.exit(__doc__)
