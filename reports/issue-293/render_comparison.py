"""Compare production action-bar rasters at native pixel size (requires Pillow/Rust)."""

import os
from pathlib import Path
import subprocess
import tempfile

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
BASE = "56dc800"
PROMPTS = [
    "Press L to land",
    "Press E to use",
    "Silicate rock - approx 10.5 kg left\nHold left mouse to mine",
]
HARNESS = r'''
fn main() {
    let output = std::env::args().nth(1).unwrap();
    for (dpi_index, dpi) in [1.0, 1.5, 2.0].into_iter().enumerate() {
        for (index, text) in ["Press L to land", "Press E to use",
            "Silicate rock - approx 10.5 kg left\nHold left mouse to mine"
        ].into_iter().enumerate() {
            let mut bar = ActionBar::default();
            bar.set_contextual(Some(text.to_owned()));
            bar.set_scale_factor(dpi);
            let image = bar.image(true).unwrap();
            let mut ppm = format!("P6\n{} {}\n255\n", image.width, image.height).into_bytes();
            for pixel in image.rgba8.chunks_exact(4) {
                ppm.extend_from_slice(&pixel[..3]);
            }
            std::fs::write(format!("{output}/{dpi_index}-{index}.ppm"), ppm).unwrap();
        }
    }
}
'''


def main():
    versions = {
        "Before": subprocess.check_output(
            ["git", "show", f"{BASE}:client/runtime/src/action_bar.rs"], cwd=ROOT
        ).decode(),
        "After": (ROOT / "client/runtime/src/action_bar.rs").read_text(),
    }
    with tempfile.TemporaryDirectory() as temporary:
        scratch = Path(temporary)
        for label, source in versions.items():
            output = scratch / label
            output.mkdir()
            source_path = output / "main.rs"
            source_path.write_text(source + HARNESS)
            binary = output / "raster"
            subprocess.run(
                [os.environ.get("RUSTC", "rustc"), "--edition=2024", "-Awarnings",
                 str(source_path), "-o", str(binary)], check=True
            )
            subprocess.run([str(binary), str(output)], check=True)
        for dpi_index, dpi in enumerate([1.0, 1.5, 2.0]):
            sheet = Image.new("RGB", (1400, 900), (20, 27, 40))
            draw = ImageDraw.Draw(sheet)
            draw.text((20, 15), f"Issue #293 | {dpi}x DPI | native physical pixels, no resizing", fill="white")
            draw.text((20, 38), "CPU production raster comparison; not a native game screenshot. Alpha shown opaque.", fill="white")
            y = 75
            for index in range(len(PROMPTS)):
                for label in versions:
                    with Image.open(scratch / label / f"{dpi_index}-{index}.ppm") as raster:
                        draw.text((20, y), f"{label}: {raster.width} x {raster.height} px", fill="white")
                        sheet.paste(raster, (20, y + 18))
                        y += raster.height + 32
                y += 8
            sheet.save(Path(__file__).parent / f"dpi-{dpi}.png")


if __name__ == "__main__":
    main()
