"""Reject misleading canonical terrain evidence as well as streaming regressions."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

try:
    from PIL import Image
    from review_universe_captures import LIMITS, REQUIRED, orbit_diameter, review
except ImportError:
    Image = None


@unittest.skipIf(Image is None, "requires scripts/visual-requirements.txt (Pillow)")
class UniverseCaptureReviewTests(unittest.TestCase):
    def test_complete_native_evidence_and_failed_integrity_cases(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            image = Image.new("RGB", (128, 80))
            for y in range(15, 65):
                for x in range(39, 89):
                    if (x - 64)**2 + (y - 40)**2 <= 25**2:
                        c = 70 + (x + y) % 160
                        image.putpixel((x, y), (c, c, c))
            a, b = root / "a.png", root / "b.png"
            image.save(a)
            image.save(b)
            steps = []
            for label in sorted(REQUIRED):
                for suffix in ("", "-repeat"):
                    steps.append({"status": "passed", "step": {"screenshot": label + suffix},
                                  "screenshot": {"status": "captured", "source": "GPU color surface", "path": str(b if suffix else a)},
                                  "state": {"visual_context": {"viewport_pixels": [128, 80]},
                                            "reference_camera": {"name": label.removesuffix("-revisit"), "canonical_terrain_clearance_meters": 500},
                                            "terrain": {**{k: 1 for k in LIMITS}, "mode": "Canonical", "triangles": 20,
                                                        "presentation_ready": True, "pending_patches": 0},
                                            "performance": {"samples": 1, "raw_frames": [[1, 2, 3, None, 7, 9]]}}})
            source = {"status": "passed", "steps": steps}
            result = root / "result.json"

            def check(data):
                result.write_text(json.dumps(data))
                return review(result)

            self.assertEqual(check(source)["exact_repeats"], 4)
            for mutation in ("source", "dimensions", "failed", "missing", "duplicate", "context", "blank", "raw", "pending", "underground", "wrong-view", *LIMITS):
                with self.subTest(mutation=mutation):
                    data = copy.deepcopy(source)
                    step = data["steps"][0]
                    if mutation == "source":
                        step["screenshot"]["source"] = "OS desktop"
                    elif mutation == "dimensions":
                        step["state"]["visual_context"]["viewport_pixels"] = [8, 8]
                    elif mutation == "failed":
                        data["status"] = "failed"
                    elif mutation == "missing":
                        data["steps"].pop()
                    elif mutation == "duplicate":
                        data["steps"].append(copy.deepcopy(step))
                    elif mutation == "context":
                        step["state"]["visual_context"]["backend"] = "other"
                    elif mutation == "blank":
                        step["state"]["terrain"]["active_patches"] = 0
                    elif mutation == "raw":
                        step["state"]["performance"]["raw_frames"] = []
                    elif mutation == "underground":
                        step["state"]["reference_camera"]["canonical_terrain_clearance_meters"] = -675
                    elif mutation == "wrong-view":
                        step["state"]["reference_camera"]["name"] = "another-view"
                    elif mutation == "pending":
                        step["state"]["terrain"]["pending_patches"] = 1
                    else:
                        step["state"]["terrain"][mutation] = LIMITS[mutation] + 1
                    with self.assertRaises(ValueError):
                        check(data)
            image.putpixel((0, 0), (255, 0, 0))
            image.save(b)
            with self.assertRaisesRegex(ValueError, "frames differ"):
                check(source)
            b.unlink()
            with self.assertRaises(OSError):
                check(source)

    def test_enlarged_or_shifted_disk_and_blank_image_reject(self):
        for x0, x1 in [(20, 108), (60, 110)]:
            image = Image.new("RGB", (128, 80))
            for x in range(x0, x1):
                image.putpixel((x, 40), (150, 150, 150))
            with self.assertRaisesRegex(ValueError, "physical scale"):
                orbit_diameter(image)
        with self.assertRaisesRegex(ValueError, "missing"):
            orbit_diameter(Image.new("RGB", (128, 80)))
