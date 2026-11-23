"""Write the test media of a tutorial walk: three short WAV files with 16-bit
samples, one with 24-bit samples, and a cover image. Studio refuses a WAV file
with 32-bit integer samples, so the walk does not use one.

Usage: python3 -I tools/tutorial-walk/media.py FOLDER
"""
import math
import struct
import sys
import wave
import zlib
from pathlib import Path


def tone(path, seconds, hertz, width):
    rate = 44100
    peak = (1 << (8 * width - 1)) - 1
    frames = bytearray()
    for n in range(int(rate * seconds)):
        value = int(0.3 * peak * math.sin(2 * math.pi * hertz * n / rate))
        frames += value.to_bytes(width, "little", signed=True) * 2
    with wave.open(str(path), "wb") as out:
        out.setnchannels(2)
        out.setsampwidth(width)
        out.setframerate(rate)
        out.writeframes(bytes(frames))


def cover(path, size=600):
    """A plain PNG: a light triangle on a dark ground. No image library."""
    rows = bytearray()
    for y in range(size):
        rows.append(0)
        half = y / 2
        for x in range(size):
            inside = abs(x - size / 2) < half and 0.15 * size < y < 0.85 * size
            rows += bytes((236, 236, 244) if inside else (24, 24, 36))

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body))

    head = struct.pack(">IIBBBBB", size, size, 8, 2, 0, 0, 0)
    path.write_bytes(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", head)
                     + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b""))


def main():
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    for number, (seconds, hertz) in enumerate([(25, 220), (30, 330), (20, 440)], start=1):
        tone(folder / f"0{number}-new-song.wav", seconds, hertz, 2)
    tone(folder / "t24.wav", 5, 262, 3)
    cover(folder / "cover.png")
    print("media:", ", ".join(sorted(p.name for p in folder.iterdir())))


if __name__ == "__main__":
    main()
