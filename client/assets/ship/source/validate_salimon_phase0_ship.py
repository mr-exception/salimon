#!/usr/bin/env python3
"""Compatibility entry point for the Blender-authored scout validator."""
from pathlib import Path
import sys

SCOUT = Path(__file__).resolve().parents[4] / "models/assets/ships/salimon-scout"
sys.path.insert(0, str(SCOUT))
from validate import main
from validate_asset import ValidationError

if __name__ == "__main__":
    try:
        main()
    except (ValidationError, OSError, ValueError) as exc:
        print(f"scout validation failed: {exc}", file=sys.stderr)
        sys.exit(1)
