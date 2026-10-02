#!/usr/bin/env python3
"""Regression coverage for Blender-authored scout spatial contracts."""
import json
from pathlib import Path
import sys
import unittest

REPO = Path(__file__).resolve().parents[2]
SCOUT = REPO / "models/assets/ships/salimon-scout"
sys.path.insert(0, str(SCOUT))
sys.path.insert(0, str(REPO / "models/tools"))

from spatial_contracts import (
    anchors_rust_source,
    build_spatial_contracts,
    cargo_rust_source,
    sidecar_json,
    thruster_rust_source,
)
from validate_asset import read_document


class ScoutSpatialContractsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        document, _ = read_document(REPO / "client/assets/ship/export/salimon_phase0_ship.glb")
        preservation = json.loads((SCOUT / "preservation.json").read_text())
        cls.contracts = build_spatial_contracts(document, preservation)

    def test_authored_contract_inventory_and_semantics(self):
        self.assertEqual(len(self.contracts["colliders"]), 22)
        self.assertEqual(
            set(self.contracts["markers"]),
            {"MARKER_CockpitSeat", "MARKER_ExitDoor", "MARKER_PlayerStart"},
        )
        self.assertEqual(self.contracts["markers"]["MARKER_CockpitSeat"]["purpose"], "cockpit-seat")
        self.assertEqual(self.contracts["markers"]["MARKER_ExitDoor"]["purpose"], "exit-door")
        self.assertEqual(self.contracts["markers"]["MARKER_PlayerStart"]["purpose"], "player-start")
        self.assertEqual(
            self.contracts["colliders"]["COLLIDER_Engine_Port_Body"]["purpose"],
            "exterior-thruster",
        )
        self.assertEqual(
            self.contracts["colliders"]["COLLIDER_Cargo_Deck"]["purpose"],
            "cargo-room",
        )

    def test_generated_runtime_outputs_match_checked_in_files(self):
        self.assertEqual(
            (REPO / "client/character/src/cargo_layout.rs").read_text(),
            cargo_rust_source(self.contracts),
        )
        self.assertEqual(
            (REPO / "client/character/src/thruster_collision.rs").read_text(),
            thruster_rust_source(self.contracts),
        )
        self.assertEqual(
            (REPO / "client/character/src/ship_anchors.rs").read_text(),
            anchors_rust_source(self.contracts),
        )
        self.assertEqual(
            (REPO / "client/assets/ship/spatial-contracts.json").read_text(),
            sidecar_json(self.contracts),
        )


if __name__ == "__main__":
    unittest.main()
