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
    build_spatial_contracts,
    sidecar_json,
    spatial_rust_source,
    REQUIRED_COLLIDERS,
    MARKERS,
    LAYOUT_BOXES,
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
        for name in ("COLLIDER_CockpitNoseFloor", "COLLIDER_CockpitCenterConsole",
                     "COLLIDER_Engine_Port_Body",
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

    def test_authored_engine_and_seat_edits_reach_runtime_layouts(self):
        document = copy.deepcopy(self.document)
        engine = next(node for node in document["nodes"]
                      if node["name"] == "COLLIDER_Engine_Port_Body")
        engine["translation"][0] += 1.0
        seat = next(node for node in document["nodes"] if node["name"] == "MARKER_CockpitSeat")
        seat["translation"][1] += 0.1
        edited = build_spatial_contracts(document, self.preservation)
        self.assertAlmostEqual(edited["anchors"]["enginePort"][0],
                               self.contracts["anchors"]["enginePort"][0] + 1.0,
                               delta=1e-6)
        self.assertAlmostEqual(edited["anchors"]["cockpitSeat"][1],
                               self.contracts["anchors"]["cockpitSeat"][1] + 0.1,
                               delta=1e-6)
        self.assertNotEqual(spatial_rust_source(edited), spatial_rust_source(self.contracts))
        self.assertNotEqual(sidecar_json(edited), sidecar_json(self.contracts))

    def test_authored_contract_inventory_and_semantics(self):
        self.assertEqual(len(self.contracts["colliders"]), 20)
        self.assertNotIn("cargoRoom", self.contracts)
        self.assertNotIn("cargoCenter", self.contracts["anchors"])
        self.assertFalse(any(name.startswith("COLLIDER_Cargo_")
                             for name in self.contracts["colliders"]))
        self.assertIn("COLLIDER_CockpitNoseFloor", self.contracts["colliders"])
        self.assertIn("COLLIDER_CockpitCenterConsole", self.contracts["colliders"])
        self.assertEqual(
            set(self.contracts["markers"]),
            set(MARKERS),
        )
        self.assertEqual(self.contracts["markers"]["MARKER_CockpitSeat"]["purpose"], "cockpit-seat")
        self.assertEqual(self.contracts["markers"]["MARKER_ExitDoor"]["purpose"], "exit-door")
        self.assertEqual(self.contracts["markers"]["MARKER_PlayerStart"]["purpose"], "player-start")
        self.assertEqual(
            self.contracts["colliders"]["COLLIDER_Engine_Port_Body"]["purpose"],
            "exterior-thruster",
        )

    def test_every_required_node_is_checked_by_spatial_generator(self):
        for name in sorted(REQUIRED_COLLIDERS | set(MARKERS)):
            with self.subTest(name=name):
                document = copy.deepcopy(self.document)
                next(node for node in document["nodes"] if node["name"] == name)["name"] += "_renamed"
                with self.assertRaisesRegex(ValueError, name):
                    build_spatial_contracts(document, self.preservation)

    def test_nonfinite_malformed_or_parent_relative_contracts_fail(self):
        for name in ("COLLIDER_CabinTraversalEnvelope", "MARKER_DoorwayTransition"):
            for translation in ([float("nan"), 0, 0], [0, float("inf"), 0], [0, 0], ["0", 0, 0]):
                with self.subTest(name=name, translation=translation):
                    document = copy.deepcopy(self.document)
                    next(n for n in document["nodes"] if n["name"] == name)["translation"] = translation
                    with self.assertRaisesRegex(ValueError, name):
                        build_spatial_contracts(document, self.preservation)
        for size in ([1, float("inf"), 1], [1, float("nan"), 1], [1, -1, 1], [1, True, 1]):
            document = copy.deepcopy(self.document)
            proxy = next(n for n in document["nodes"] if n["name"] == "COLLIDER_PilotChair")
            proxy["extras"]["salimon"]["sizeMeters"] = size
            with self.assertRaisesRegex(ValueError, "sizeMeters"):
                build_spatial_contracts(document, self.preservation)
        for group in ("Collision_Proxies", "Interaction_Markers", "Salimon_Phase0_Scout"):
            document = copy.deepcopy(self.document)
            next(n for n in document["nodes"] if n["name"] == group)["translation"] = [1, 0, 0]
            with self.assertRaisesRegex(ValueError, "identity translation"):
                build_spatial_contracts(document, self.preservation)

    def test_each_authored_layout_box_edit_changes_its_generated_bounds(self):
        # Covers floor/door/Core/console/chair/nose/hull and both envelopes,
        # rather than testing only engine and seat anchors.
        for rust_name, name in LAYOUT_BOXES.items():
            with self.subTest(name=name):
                document = copy.deepcopy(self.document)
                proxy = next(n for n in document["nodes"] if n["name"] == name)
                proxy["translation"][0] += 0.25
                edited = build_spatial_contracts(document, self.preservation)
                for key in ("lowerMeters", "upperMeters"):
                    self.assertAlmostEqual(edited["colliders"][name][key][0],
                                           self.contracts["colliders"][name][key][0] + 0.25,
                                           delta=1e-6)
                self.assertIn(rust_name, spatial_rust_source(edited))
                self.assertNotEqual(spatial_rust_source(edited), spatial_rust_source(self.contracts))

    def test_unsupported_asymmetric_width_or_uneven_floor_is_rejected(self):
        for name, axis, message in (
            ("COLLIDER_CabinTraversalEnvelope", 2, "centered"),
            ("COLLIDER_AftDoor", 2, "centered"),
            ("COLLIDER_CockpitNoseFloor", 1, "level"),
        ):
            document = copy.deepcopy(self.document)
            next(n for n in document["nodes"] if n["name"] == name)["translation"][axis] += 0.1
            with self.assertRaisesRegex(ValueError, message):
                build_spatial_contracts(document, self.preservation)

    def test_doorway_transition_edit_reaches_generated_crossing_plane(self):
        document = copy.deepcopy(self.document)
        marker = next(n for n in document["nodes"] if n["name"] == "MARKER_DoorwayTransition")
        marker["translation"][0] += 0.25
        edited = build_spatial_contracts(document, self.preservation)
        self.assertAlmostEqual(edited["anchors"]["doorwayTransition"][0], -6.79, delta=1e-6)
        self.assertNotEqual(spatial_rust_source(edited), spatial_rust_source(self.contracts))

    def test_generated_runtime_outputs_match_checked_in_files(self):
        self.assertFalse((REPO / "client/character/src/cargo_layout.rs").exists())
        self.assertEqual(
            (REPO / "client/character/src/spatial_contracts.rs").read_text(),
            spatial_rust_source(self.contracts),
        )
        for retired in ("ship_anchors.rs", "thruster_collision.rs"):
            self.assertFalse((REPO / "client/character/src" / retired).exists())
        self.assertEqual(
            (REPO / "client/assets/ship/spatial-contracts.json").read_text(),
            sidecar_json(self.contracts),
        )


if __name__ == "__main__":
    unittest.main()
