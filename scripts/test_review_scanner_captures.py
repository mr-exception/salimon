"""Protect evidence review from accepting absent, changed or misattributed captures."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

try:
    from PIL import Image
    from review_scanner_captures import review
except ImportError:
    Image = None


@unittest.skipIf(Image is None, "requires scripts/visual-requirements.txt (Pillow)")
class ScannerCaptureReviewTests(unittest.TestCase):
    def test_original_repeat_passes_but_changed_pixels_provenance_dimensions_or_power_reject(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = Image.new("RGB", (1280, 800))
            for x in range(635, 645):
                image.putpixel((x, 400), (245, 185, 98))
            a, b = root / "a.png", root / "b.png"
            image.save(a)
            image.save(b)
            steps = [{"status": "passed", "step": {"screenshot": label},
                      "screenshot": {"status": "captured", "source": "GPU color surface", "path": str(path)},
                      "state": {"visual_context": {"viewport_pixels": [1280, 800]},
                                "scanner": {"powered": True, "active_count": 1, "hint": "FRESH"}}}
                     for label, path in (("fresh", a), ("fresh-repeat", b))]
            source = {"status": "passed", "steps": steps}
            result = root / "result.json"

            def check(data):
                result.write_text(json.dumps(data))
                return review(result)

            self.assertEqual(check(source)["exact_repeats"], 1)
            image.putpixel((0, 0), (255, 255, 255))
            image.save(b)
            with self.assertRaisesRegex(ValueError, "frames differ"):
                check(source)
            image.putpixel((0, 0), (0, 0, 0))
            image.save(b)
            for mutation in ("source", "size", "power", "failed"):
                data = copy.deepcopy(source)
                step = data["steps"][0]
                if mutation == "source":
                    step["screenshot"]["source"] = "OS desktop"
                elif mutation == "size":
                    step["state"]["visual_context"]["viewport_pixels"] = [640, 360]
                elif mutation == "power":
                    step["state"]["scanner"]["powered"] = False
                else:
                    data["status"] = "failed"
                with self.assertRaises(ValueError, msg=mutation):
                    check(data)
            b.unlink()
            with self.assertRaises(OSError):
                check(source)
