#!/usr/bin/env python3
"""Regression coverage for Blender-authored scout spatial contracts."""
import copy
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
from validate_asset import ValidationError
from test_scout_export import scout


class ScoutSpatialContractsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        document, _ = read_document(REPO / "client/assets/ship/export/salimon_phase0_ship.glb")
        preservation = json.loads((SCOUT / "preservation.json").read_text())
        cls.document = document
        cls.preservation = preservation
        cls.contracts = build_spatial_contracts(document, preservation)

    def test_missing_or_renamed_spatial_contracts_are_rejected(self):
        for name in ("COLLIDER_Cargo_Deck", "COLLIDER_Engine_Port_Body",
                     "MARKER_CockpitSeat", "MARKER_ExitDoor", "MARKER_PlayerStart"):
            with self.subTest(name=name):
                document = copy.deepcopy(self.document)
                next(node for node in document["nodes"] if node["name"] == name)["name"] += "_renamed"
                with self.assertRaisesRegex(ValidationError, "node set changed"):
                    scout.validate_preservation({"document": document}, {})

    def test_rotated_or_malformed_collision_boxes_are_rejected(self):
        for field, value, message in (
            ("rotation", [0, 0, 1, 0], "axis-aligned"),
            ("scale", [2, 1, 1], "applied scale"),
            ("sizeMeters", [1, 0, 1], "positive"),
            ("collisionShape", "sphere", "must be box"),
        ):
            with self.subTest(field=field):
                document = copy.deepcopy(self.document)
                proxy = next(node for node in document["nodes"]
                             if node["name"] == "COLLIDER_Engine_Port_Body")
                if field in ("rotation", "scale"):
                    proxy[field] = value
                else:
                    proxy["extras"]["salimon"][field] = value
                with self.assertRaisesRegex(ValueError, message):
                    build_spatial_contracts(document, self.preservation)

    def test_authored_engine_and_cargo_edits_reach_runtime_layouts(self):
        document = copy.deepcopy(self.document)
        engine = next(node for node in document["nodes"]
                      if node["name"] == "COLLIDER_Engine_Port_Body")
        engine["translation"][0] += 1.0
        deck = next(node for node in document["nodes"] if node["name"] == "COLLIDER_Cargo_Deck")
        deck["translation"][1] += 0.1
        edited = build_spatial_contracts(document, self.preservation)
        self.assertAlmostEqual(edited["anchors"]["enginePort"][0],
                               self.contracts["anchors"]["enginePort"][0] + 1.0)
        self.assertAlmostEqual(edited["cargoRoom"]["clearMinMeters"][1],
                               self.contracts["cargoRoom"]["clearMinMeters"][1] + 0.1)
        self.assertNotEqual(thruster_rust_source(edited), thruster_rust_source(self.contracts))
        self.assertNotEqual(cargo_rust_source(edited), cargo_rust_source(self.contracts))
        self.assertNotEqual(anchors_rust_source(edited), anchors_rust_source(self.contracts))

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
