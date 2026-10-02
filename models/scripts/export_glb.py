"""Export the currently opened Blender scene as a game-ready GLB.

Usage:
    blender --background path/to/model.blend \
      --python models/scripts/export_glb.py -- path/to/output.glb
"""

from pathlib import Path
import sys

import bpy


def script_args() -> list[str]:
    if "--" not in sys.argv:
        return []
    return sys.argv[sys.argv.index("--") + 1 :]


def main() -> None:
    args = script_args()
    if len(args) != 1:
        raise SystemExit("expected exactly one output .glb path after --")

    output = Path(args[0]).resolve()
    if output.suffix.lower() != ".glb":
        raise SystemExit("output path must end with .glb")

    output.parent.mkdir(parents=True, exist_ok=True)

    # Export object names and Blender custom properties. Those are part of the
    # runtime contract for interactive Salimon assets.
    bpy.ops.export_scene.gltf(
        filepath=str(output),
        export_format="GLB",
        export_yup=True,
        export_extras=True,
        export_apply=False,
        export_texcoords=True,
        export_normals=True,
        export_materials="EXPORT",
    )
    print(f"Exported {output}")


if __name__ == "__main__":
    main()
