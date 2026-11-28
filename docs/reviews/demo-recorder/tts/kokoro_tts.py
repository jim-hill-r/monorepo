"""Synthesizes demo narration with Kokoro, an open-weight text-to-speech model
that runs on the CPU, for the review deck demo videos (../README.md).

Reads a JSON list of {"id": ..., "text": ...} on stdin, writes <out>/<id>.wav
for each line, and prints {"<id>": <seconds>, ...} on stdout. The model
files come from the kokoro-onnx GitHub release, pinned by SHA-256, and are
kept in --models (downloaded on first use).

    python -I kokoro_tts.py --out DIR --models DIR [--voice af_heart] [--speed 1.0]

The same text, voice and speed always give the same audio, so a demo can be
regenerated without changing its timing.
"""

import argparse
import hashlib
import json
import sys
import urllib.request
import wave
from pathlib import Path

RELEASE = "https://github.com/thewh1teagle/kokoro-onnx/releases/download/model-files-v1.0"
FILES = {
    "kokoro-v1.0.int8.onnx": "6e742170d309016e5891a994e1ce1559c702a2ccd0075e67ef7157974f6406cb",
    "voices-v1.0.bin": "bca610b8308e8d99f32e6fe4197e7ec01679264efed0cac9140fe9c29f1fbf7d",
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def fetch_models(models: Path) -> None:
    models.mkdir(parents=True, exist_ok=True)
    for name, want in FILES.items():
        path = models / name
        if path.exists() and sha256(path) == want:
            continue
        print(f"Downloading {name}", file=sys.stderr)
        part = path.with_suffix(".part")
        urllib.request.urlretrieve(f"{RELEASE}/{name}", part)  # noqa: S310 (fixed https URL)
        got = sha256(part)
        if got != want:
            part.unlink()
            raise SystemExit(f"{name}: SHA-256 {got}, expected {want}")
        part.replace(path)


def write_wav(path: Path, samples, rate: int) -> None:
    import numpy as np

    pcm = (np.clip(samples, -1.0, 1.0) * 32767).astype("<i2")
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(pcm.tobytes())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--models", type=Path, required=True)
    parser.add_argument("--voice", default="af_heart")
    parser.add_argument("--speed", type=float, default=1.0)
    parser.add_argument("--lang", default="en-us")
    args = parser.parse_args()

    lines = json.load(sys.stdin)
    fetch_models(args.models)

    from kokoro_onnx import Kokoro

    kokoro = Kokoro(str(args.models / "kokoro-v1.0.int8.onnx"), str(args.models / "voices-v1.0.bin"))
    args.out.mkdir(parents=True, exist_ok=True)
    durations = {}
    for line in lines:
        samples, rate = kokoro.create(line["text"], voice=args.voice, speed=args.speed, lang=args.lang)
        write_wav(args.out / f"{line['id']}.wav", samples, rate)
        durations[line["id"]] = len(samples) / rate
    json.dump(durations, sys.stdout)


if __name__ == "__main__":
    main()
