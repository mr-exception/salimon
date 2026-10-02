"""Bootstrap the existing validated Phase 0 ship into an editable Blender file.

Run with:
    blender --background --python models/scripts/bootstrap_ship.py
"""

from pathlib import Path

import bpy


ROOT = Path(__file__).resolve().parents[2]
SOURCE_GLTF = ROOT / "client/assets/ship/export/salimon_phase0_ship.gltf"
OUTPUT_BLEND = ROOT / "models/ship/salimon_phase0_ship.blend"


def main() -> None:
    if not SOURCE_GLTF.exists():
        raise FileNotFoundError(SOURCE_GLTF)

    bpy.ops.wm.read_factory_settings(use_empty=True)
    bpy.ops.import_scene.gltf(filepath=str(SOURCE_GLTF))
    OUTPUT_BLEND.parent.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.save_as_mainfile(filepath=str(OUTPUT_BLEND))
    print(f"Saved editable ship source to {OUTPUT_BLEND}")


if __name__ == "__main__":
    main()
