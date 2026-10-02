#!/usr/bin/env python3
"""Scout category extension: authored geometry, spatial and cockpit contracts."""
import json
import math
from pathlib import Path
import sys

SCOUT = Path(__file__).resolve().parent
REPO = SCOUT.parents[3]
ROOT = REPO / "client/assets/ship"
sys.path.insert(0, str(REPO / "models/tools"))
from validate_asset import (ValidationError, accessor_values, position_bounds, read_document,
                            require, validate_asset, validate_manifest)
from spatial_contracts import build_spatial_contracts

TASK7_BASELINE_DIMENSIONS = [10.15, 3.72, 8.30]
TASK10_REVISED_DIMENSIONS = [20.90, 4.00, 21.00]
TASK10_REVISED_SCALE = [2.0, 4.0 / 3.72, 2.0]

def validate_preservation(context, data):
    """Keep runtime-significant hierarchy, transforms, extras and material roles."""
    baseline = json.loads((SCOUT / 'preservation.json').read_text())
    doc = context['document']
    nodes = {n['name']: n for n in doc['nodes']}
    parents = {doc['nodes'][child]['name']: n['name'] for n in doc['nodes']
               for child in n.get('children', [])}
    require(nodes.keys() == {n['name'] for n in baseline['nodes']}, 'scout node set changed')
    for expected in baseline['nodes']:
        name = expected['name']
        actual = nodes[name]
        require(parents.get(name) == expected['parent'], f'{name}: parent changed')
        require(('mesh' in actual) == expected['visual'], f'{name}: visual role changed')
        spatial = name.startswith(('COLLIDER_', 'MARKER_'))
        if not spatial:
            require(actual.get('extras', {}) == expected.get('extras', {}), f'{name}: legacy extras changed')
            require(all(abs(a-b) <= 1e-6 for a, b in zip(actual.get('translation', [0, 0, 0]),
                                                        expected.get('translation', [0, 0, 0]))),
                    f'{name}: spatial transform changed')
        require(actual.get('rotation', [0, 0, 0, 1]) == [0, 0, 0, 1] and
                actual.get('scale', [1, 1, 1]) == [1, 1, 1] and 'matrix' not in actual,
                f'{name}: legacy baked transform required')
    materials = {m['name']: m for m in doc['materials']}
    for expected in baseline['materials']:
        actual = materials[expected['name']]
        for key, default in (('alphaMode', 'OPAQUE'), ('doubleSided', False)):
            require(actual.get(key, default) == expected.get(key, default),
                    f'{expected["name"]}: {key} changed')
        for key, default in (('baseColorFactor', [1, 1, 1, 1]),
                             ('metallicFactor', 1), ('roughnessFactor', 1)):
            a = actual.get('pbrMetallicRoughness', {}).get(key, default)
            b = expected.get('pbrMetallicRoughness', {}).get(key, default)
            a, b = (a, b) if isinstance(a, list) else ([a], [b])
            require(len(a) == len(b) and all(abs(x-y) < 1e-6 for x, y in zip(a, b)), f'{expected["name"]}: {key} changed')
        require(all(abs(x-y) < 1e-6 for x, y in zip(actual.get('emissiveFactor', [0, 0, 0]),
                                                  expected.get('emissiveFactor', [0, 0, 0]))),
                f'{expected["name"]}: emission changed')


def node_position_bounds(document: dict[str, object], binary: bytes, node_name: str) -> tuple[list[float], list[float]]:
    node = next(node for node in document["nodes"] if node["name"] == node_name)
    primitive = document["meshes"][node["mesh"]]["primitives"][0]
    return position_bounds(document, binary, primitive["attributes"]["POSITION"])


def overall_position_bounds(document, binary):
    points = [point for mesh in document["meshes"] for primitive in mesh["primitives"]
              for point in accessor_values(document, binary, primitive["attributes"]["POSITION"])]
    return ([min(p[a] for p in points) for a in range(3)],
            [max(p[a] for p in points) for a in range(3)])


def node(document: dict[str, object], name: str) -> dict[str, object]:
    return next(candidate for candidate in document["nodes"] if candidate["name"] == name)


def assert_vectors_close(actual: list[float], expected: list[float]) -> None:
    require(len(actual) == len(expected), 'len(actual) == len(expected)')
    require(all(abs(left - right) <= 1.0e-6 for left, right in zip(actual, expected)), (
        actual,
        expected,
    ))


def validate_center_monitor_scale(document: dict, binary: bytes, manifest: dict) -> None:
    """Check the authored center assembly without recreating procedural meshes."""
    contract = json.loads((SCOUT / "monitor-contract.json").read_text())["centerVertices"]
    measured = []
    for name, expected in contract.items():
        exported = node(document, name)
        primitive = document["meshes"][exported["mesh"]]["primitives"][0]
        positions = accessor_values(document, binary, primitive["attributes"]["POSITION"])
        for point in expected:
            require(any(all(abs(a-b) <= 1e-6 for a,b in zip(point, actual)) for actual in positions), f"{name}: center assembly vertex changed")
        measured.extend(next(actual for actual in positions if all(abs(a-b) <= 1e-6 for a,b in zip(point, actual))) for point in expected)
    bounds = {"min": [min(p[a] for p in measured) for a in range(3)],
              "max": [max(p[a] for p in measured) for a in range(3)]}
    pivot, scale, seatward = [4.426, .80, 0.0], .49, .40
    metadata = document["extras"]["salimon"]["cockpitInstruments"]["centerAssembly"]
    require(metadata == manifest["cockpitInstruments"]["centerAssembly"], 'metadata == manifest["cockpitInstruments"]["centerAssembly"]')
    require(metadata["uniformScaleFromAssetVersion7"] == scale, 'metadata["uniformScaleFromAssetVersion7"] == scale')
    require(metadata["uniformScaleFromAssetVersion8"] == .7, 'metadata["uniformScaleFromAssetVersion8"] == .7')
    require(metadata["seatwardOffsetMeters"] == seatward, 'metadata["seatwardOffsetMeters"] == seatward')
    assert_vectors_close(metadata["pivotMeters"], pivot)
    baseline = {"min": [4.264, .8, -1.02], "max": [4.511, 1.695, 1.02]}
    for bound in ("min", "max"):
        assert_vectors_close(metadata["baselineBoundsMeters"][bound], baseline[bound])
        assert_vectors_close(metadata["boundsMeters"][bound], bounds[bound])
    collider = node(document, metadata["collisionNode"])
    center = collider["translation"]
    size = collider["extras"]["salimon"]["sizeMeters"]
    assert_vectors_close([value-width/2 for value, width in zip(center, size)], [3.94662, .25, -1.06])
    assert_vectors_close([value+width/2 for value, width in zip(center, size)], [6.12, 1.23855, 1.06])


def validate_sightlines(document: dict, binary: bytes) -> None:
    """Ray-test real exported triangles, catching opaque walls behind new panes."""
    def values(index):
        return accessor_values(document, binary, index)
    def sub(a,b): return tuple(x-y for x,y in zip(a,b))
    def dot(a,b): return sum(x*y for x,y in zip(a,b))
    def cross(a,b): return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
    def hit(origin,direction,tri):
        a,b,c=tri; e1,e2=sub(b,a),sub(c,a); h=cross(direction,e2); det=dot(e1,h)
        if abs(det)<1e-8:return None
        inv=1/det; delta=sub(origin,a); u=inv*dot(delta,h)
        if u<0 or u>1:return None
        q=cross(delta,e1); v=inv*dot(direction,q)
        if v<0 or u+v>1:return None
        distance=inv*dot(e2,q)
        return distance if distance>.001 else None
    meshes=[]
    for mesh in document["meshes"]:
        p=mesh["primitives"][0]; positions=values(p["attributes"]["POSITION"]); indices=values(p["indices"])
        triangles=[[positions[indices[i+j][0]] for j in range(3)] for i in range(0,len(indices),3)]
        glass=document["materials"][p["material"]]["name"]=="Cockpit Glass"
        meshes.append((mesh["name"],glass,triangles))
    eye=.23*4/3.72+1.75
    checks=[("behind chair forward",(1.3,eye,0),(1,0,0)),
            ("seated forward",(2.76,1.799032258064516,0),(1,0,0)),
            ("port window",(-4.5,eye,2.2),(0,0,1)),
            ("starboard window",(-4.5,eye,-2.2),(0,0,-1)),
            ("aft port window",(-5.8,eye,2.6),(-1,0,0)),
            ("aft starboard window",(-5.8,eye,-2.6),(-1,0,0))]
    for label,origin,direction in checks:
        glass_hit=False
        for name,glass,triangles in meshes:
            distances=[d for tri in triangles if (d:=hit(origin,direction,tri)) is not None]
            if distances:
                if glass:glass_hit=True
                else:require(False, f"{label} obstructed by {name} at {min(distances):.2f}m")
        require(glass_hit, f"{label} misses modeled glazing")
    # Sample both sides of the console from the real seated eye. Rays must
    # pass through the LOWER pane, then reach the world without any opaque
    # hull, deck, monitor, furniture or collision proxy covering the view.
    for pitch,absolute_yaw in ((12,28),(18,28),(26,32),(30,34),(30,36)):
        for yaw in (-absolute_yaw, absolute_yaw):
            p,y=math.radians(pitch),math.radians(yaw)
            direction=(math.cos(p)*math.cos(y),-math.sin(p),math.cos(p)*math.sin(y))
            hits=[(d,name,glass) for name,glass,triangles in meshes for tri in triangles
                  if (d:=hit((2.76,1.799032258064516,0),direction,tri)) is not None]
            require(hits, f"lower window ray misses glazing: {pitch}/{yaw}")
            require(any(name == "Cockpit_Lower_Glazing" for _,name,_ in hits), 'any(name == "Cockpit_Lower_Glazing" for _,name,_ in hits)')
            require(all(glass for _,_,glass in hits), f"lower view {pitch}/{yaw} blocked: {hits}")
    # A standing player sees through the actual cargo doorway to the far wall;
    # neither the old sill/glazing, wing nor an opaque hull cap covers the opening.
    for x in (.7, 1.3, 1.8):
        origin=(x,eye,3.5)
        hits=[(d,name) for name,glass,triangles in meshes
              for tri in triangles if (d:=hit(origin,(0,0,1),tri)) is not None]
        require(min(hits)[1] == "Cargo_Port_Wall", f"cargo passage blocked at X={x}: {min(hits)}")
        require(abs(min(hits)[0]-7.1) < 1e-5, 'abs(min(hits)[0]-7.1) < 1e-5')
    origin=(-.5,eye,8.0)
    hits=[(d,name) for name,glass,triangles in meshes for tri in triangles
          if (d:=hit(origin,(0,-1,0),tri)) is not None]
    require(min(hits)[1] == "Cargo_Deck", "cargo floor must be level and not covered by wing geometry")
    require(abs(min(hits)[0]-1.75) < 1e-5, 'abs(min(hits)[0]-1.75) < 1e-5')

    # Live cockpit surfaces must present an unobstructed rectangle from the
    # actual seated eye, with top-left UVs and normals toward the pilot.
    seated_eye=(2.76,1.799032258064516,0.)
    for label, role, x, y0, y1, z0, z1 in (
        ("Center","speed",3.95446,.8735,1.2067,-.441,.441),
        ("Port","thruster-power",3.70,.905,1.22,2.035,2.665),
        ("Starboard","thruster-power",3.70,.905,1.22,-2.665,-2.035),
    ):
        name=f"Monitor_{label}"
        monitor=node(document,name)
        metadata=monitor["extras"]["salimon"]
        require(metadata["displayRole"] == role, 'metadata["displayRole"] == role')
        require(metadata["uvOrigin"] == "top-left", 'metadata["uvOrigin"] == "top-left"')
        if label != "Center":
            require(metadata["powerSource"] == "shared-thruster-command", 'metadata["powerSource"] == "shared-thruster-command"')
        primitive=document["meshes"][monitor["mesh"]]["primitives"][0]
        positions=values(primitive["attributes"]["POSITION"])
        normals=values(primitive["attributes"]["NORMAL"])
        uvs=values(primitive["attributes"]["TEXCOORD_0"])
        require(len(positions)==4 and len(values(primitive["indices"]))==6, 'len(positions)==4 and len(values(primitive["indices"]))==6')
        center=(x,(y0+y1)/2,(z0+z1)/2)
        toward_eye=sub(seated_eye,center)
        horizontal_length=math.hypot(toward_eye[0],toward_eye[2])
        expected_normal=(toward_eye[0]/horizontal_length,0.,toward_eye[2]/horizontal_length)
        tangent=(expected_normal[2],0.,-expected_normal[0])
        for position, normal, uv in zip(positions,normals,uvs):
            horizontal_offset=(uv[0]-.5)*(z1-z0)
            assert_vectors_close(position,[
                center[0]+tangent[0]*horizontal_offset,
                y1-(y1-y0)*uv[1],
                center[2]+tangent[2]*horizontal_offset,
            ])
            assert_vectors_close(normal,expected_normal)
        require(set(uvs)=={(0.,0.),(0.,1.),(1.,0.),(1.,1.)}, 'set(uvs)=={(0.,0.),(0.,1.),(1.,0.),(1.,1.)}')
        assert_vectors_close(metadata["pilotFacingTargetMeters"],list(seated_eye))
        expected_yaw=math.degrees(math.atan2(expected_normal[2],-expected_normal[0]))
        require(abs(metadata["pilotFacingYawDegrees"]-expected_yaw) < 1e-5, 'abs(metadata["pilotFacingYawDegrees"]-expected_yaw) < 1e-5')
        if label != "Center":
            require(abs(expected_yaw) > 45., f"{name} still faces mostly toward ship rear")
        origin=seated_eye
        for u,v in ((.08,.08),(.92,.08),(.5,.5),(.08,.92),(.92,.92)):
            horizontal_offset=(u-.5)*(z1-z0)
            target=(center[0]+tangent[0]*horizontal_offset,
                    y1-(y1-y0)*v,
                    center[2]+tangent[2]*horizontal_offset)
            direction=sub(target,origin)
            hits=[(distance,mesh_name) for mesh_name,glass,triangles in meshes
                  if not glass for tri in triangles
                  if (distance:=hit(origin,direction,tri)) is not None]
            require(hits, f"{name} inspection ray missed all surfaces")
            nearest=min(hits)
            require(nearest[1] == name, f"{name} display occluded by {nearest[1]} at UV {(u,v)}")
            require(abs(nearest[0]-1.) < 1e-5, 'abs(nearest[0]-1.) < 1e-5')

    # End caps used to conceal the thruster emitters. Trace down each nozzle.
    for label, z in (("Port",7.10),("Starboard",-7.10)):
        origin=(-10.5,4/3.72,z)
        hits=[]
        for name,glass,triangles in meshes:
            if not glass:
                hits.extend((distance,name) for tri in triangles
                            if (distance:=hit(origin,(1,0,0),tri)) is not None)
        require(min(hits)[1] == f"Engine_{label}_Glow", f"{label} nozzle emitter is occluded")


def validate_scout(context, data):
    validate_preservation(context, data)
    document, geometry = context["document"], context["binary"]
    mesh_names = [mesh["name"] for mesh in document["meshes"]]
    node_names = [n["name"] for n in document["nodes"]]
    material_names = [m["name"] for m in document["materials"]]
    for mesh in document["meshes"]:
        require(len(mesh["primitives"]) == 1, f"{mesh['name']}: expected one primitive")
        primitive = mesh["primitives"][0]
        require(set(primitive["attributes"]) == {"POSITION", "NORMAL", "TEXCOORD_0"},
                f"{mesh['name']}: required shader attributes changed")
        for semantic, kind in (("NORMAL", "VEC3"), ("TEXCOORD_0", "VEC2")):
            accessor = document["accessors"][primitive["attributes"][semantic]]
            require(accessor["type"] == kind and accessor["componentType"] == 5126,
                    f"{mesh['name']}: {semantic} must be float {kind}")
    preservation = json.loads((SCOUT / "preservation.json").read_text())
    spatial = build_spatial_contracts(document, preservation)
    if "spatialContracts" in document["extras"]["salimon"]:
        require(document["extras"]["salimon"]["spatialContracts"] == spatial, "embedded spatial contracts are stale")
    collision_group = node(document, "Collision_Proxies")
    for name in ("COLLIDER_Engine_Port_Body", "COLLIDER_Engine_Port_SweptFin",
                 "COLLIDER_Engine_Starboard_Body", "COLLIDER_Engine_Starboard_SweptFin"):
        collider = node(document, name)
        require(document["nodes"].index(collider) in collision_group["children"], 'document["nodes"].index(collider) in collision_group["children"]')
        require("mesh" not in collider, "collision proxies must not add draw calls")
        metadata = collider["extras"]["salimon"]
        require(metadata["collisionShape"] == "box", 'metadata["collisionShape"] == "box"')
        require(metadata["purpose"] == "exterior-thruster", 'metadata["purpose"] == "exterior-thruster"')
        authored = spatial["colliders"][name]
        assert_vectors_close(collider["translation"], authored["centerMeters"])
        assert_vectors_close(metadata["sizeMeters"], authored["sizeMeters"])
    # The two engines must not become one invisible wall across the aft gate.
    port = node(document, "COLLIDER_Engine_Port_Body")
    starboard = node(document, "COLLIDER_Engine_Starboard_Body")
    require(port["translation"][2] - port["extras"]["salimon"]["sizeMeters"][2] / 2 > 5.1, 'port["translation"][2] - port["extras"]["salimon"]["sizeMeters"][2] / 2 > 5.1')
    require(starboard["translation"][2] + starboard["extras"]["salimon"]["sizeMeters"][2] / 2 < -5.1, 'starboard["translation"][2] + starboard["extras"]["salimon"]["sizeMeters"][2] / 2 < -5.1')

    glass = document["materials"][material_names.index("Cockpit Glass")]
    require(glass["alphaMode"] == "BLEND", "cockpit glass must use inexpensive alpha blending")
    require(glass["doubleSided"] is True, "cockpit glass must render from inside and outside")
    require(0.0 < glass["pbrMetallicRoughness"]["baseColorFactor"][3] < 0.5, '0.0 < glass["pbrMetallicRoughness"]["baseColorFactor"][3] < 0.5')

    glazing_min, glazing_max = node_position_bounds(document, geometry, "Cockpit_Glazing")
    _, nose_max = node_position_bounds(document, geometry, "Hull_Nose")
    _, console_max = node_position_bounds(document, geometry, "Cockpit_Console_Center")
    require(glazing_min[0] <= 5.2 and glazing_max[0] >= 9.8, 'glazing_min[0] <= 5.2 and glazing_max[0] >= 9.8')
    require(glazing_min[1] <= 1.08 and glazing_max[1] >= 2.83, 'glazing_min[1] <= 1.08 and glazing_max[1] >= 2.83')
    require(glazing_min[2] <= -3.0 and glazing_max[2] >= 3.0, 'glazing_min[2] <= -3.0 and glazing_max[2] >= 3.0')
    require(nose_max[1] <= 1.10, "solid nose must stay below the forward window")
    require(console_max[1] <= 1.30, "console must stay below the seated eye line")
    require(nose_max[0] >= console_max[0], "nose must still extend beyond the forward console")

    windows = document["extras"]["salimon"]["cockpitWindows"]
    require(windows["glazingNode"] == "Cockpit_Glazing", 'windows["glazingNode"] == "Cockpit_Glazing"')
    require(windows["material"] == "Cockpit Glass", 'windows["material"] == "Cockpit Glass"')
    require(windows["lowerGlazingNode"] == "Cockpit_Lower_Glazing", 'windows["lowerGlazingNode"] == "Cockpit_Lower_Glazing"')
    lower_node=node(document, windows["lowerGlazingNode"])
    lower_primitive=document["meshes"][lower_node["mesh"]]["primitives"][0]
    require(document["materials"][lower_primitive["material"]]["name"] == "Cockpit Glass", 'document["materials"][lower_primitive["material"]]["name"] == "Cockpit Glass"')
    require(lower_node["extras"]["salimon"]["exterior_visibility"] is True, 'lower_node["extras"]["salimon"]["exterior_visibility"] is True')
    assert_vectors_close(windows["seatedViewpointMeters"], [2.76, 1.799032258064516, 0.0])
    assert_vectors_close(windows["standingViewpointMeters"], [1.30, 1.9973118279569892, 0.0])
    require(windows["dynamicShadows"] is False, 'windows["dynamicShadows"] is False')
    require(windows["postEffects"] is False, 'windows["postEffects"] is False')

    metrics = document["extras"]["salimon"]
    assert_vectors_close(metrics["scaleFromTask7Baseline"], TASK10_REVISED_SCALE)
    require(metrics["humanBodyHeightMeters"] == 1.80, 'metrics["humanBodyHeightMeters"] == 1.80')
    require(metrics["humanEyeHeightMeters"] == 1.75, 'metrics["humanEyeHeightMeters"] == 1.75')
    assert_vectors_close(metrics["task7BaselineDimensionsMeters"], TASK7_BASELINE_DIMENSIONS)
    assert_vectors_close(metrics["overallDimensionsMeters"], TASK10_REVISED_DIMENSIONS)
    overall_min, overall_max = overall_position_bounds(document, geometry)
    measured_dimensions = [maximum - minimum for minimum, maximum in zip(overall_min, overall_max)]
    assert_vectors_close(measured_dimensions, TASK10_REVISED_DIMENSIONS)
    require(measured_dimensions[0] >= TASK7_BASELINE_DIMENSIONS[0] * 2.0, 'measured_dimensions[0] >= TASK7_BASELINE_DIMENSIONS[0] * 2.0')
    require(measured_dimensions[2] >= TASK7_BASELINE_DIMENSIONS[2] * 2.0, 'measured_dimensions[2] >= TASK7_BASELINE_DIMENSIONS[2] * 2.0')
    require(abs(overall_min[1] - metrics["lowestLocalYMeters"]) <= 1.0e-6, 'abs(overall_min[1] - metrics["lowestLocalYMeters"]) <= 1.0e-6')
    assert_vectors_close(node(document, "MARKER_CockpitSeat")["translation"], [2.76, 1.129032258064516, 0.0])
    assert_vectors_close(node(document, "MARKER_ExitDoor")["translation"], [-7.10, 1.3440860215053763, 0.0])
    assert_vectors_close(node(document, "MARKER_PlayerStart")["translation"], [0.50, 1.9973118279569892, -2.20])
    exterior_collider = node(document, "COLLIDER_ExteriorHull")
    assert_vectors_close(exterior_collider["translation"], [0.05, 1.76*4/3.72, .5])
    assert_vectors_close(
        exterior_collider["extras"]["salimon"]["sizeMeters"],
        [20.90, 4.00, 21.0],
    )
    require(metrics["externalAssetDependencies"] == 0, 'metrics["externalAssetDependencies"] == 0')

    # Compare measured payload counts with the declared budget contract.
    manifest = json.loads((ROOT / "asset-manifest.json").read_text())
    require(manifest["assetVersion"] == document["asset"]["extras"]["salimon"]["assetVersion"], 'manifest["assetVersion"] == document["asset"]["extras"]["salimon"]["assetVersion"]')
    require(manifest["cockpitWindows"]["lowerGlazingNode"] == windows["lowerGlazingNode"], 'manifest["cockpitWindows"]["lowerGlazingNode"] == windows["lowerGlazingNode"]')
    require(manifest["cockpitWindows"]["lowerViewSamplesDegrees"] == windows["lowerViewSamplesDegrees"], 'manifest["cockpitWindows"]["lowerViewSamplesDegrees"] == windows["lowerViewSamplesDegrees"]')
    lower_normals=accessor_values(document,geometry,lower_primitive["attributes"]["NORMAL"])
    require(all(abs(sum(n*n for n in normal)-1.0)<1e-5 for normal in lower_normals), 'all(abs(sum(n*n for n in normal)-1.0)<1e-5 for normal in lower_normals)')
    require(any(normal[1] > .99 for normal in lower_normals), "floor panes face upward")
    require(any(normal[1] < -.9 for normal in lower_normals), "nose pane faces downward")

    require(manifest["budgets"]["maximumTriangles"] == 6000, 'manifest["budgets"]["maximumTriangles"] == 6000')
    require(context["metrics"]["maxTriangles"] == metrics["triangleCount"], 'context["metrics"]["maxTriangles"] == metrics["triangleCount"]')
    require(context["metrics"]["maxPrimitives"] == metrics["drawPrimitiveCount"], 'context["metrics"]["maxPrimitives"] == metrics["drawPrimitiveCount"]')
    require(context["metrics"]["maxMaterials"] == metrics["materialCount"], 'context["metrics"]["maxMaterials"] == metrics["materialCount"]')
    assert_vectors_close(manifest["task10Scale"]["dimensionsMeters"], measured_dimensions)
    require(metrics["walkableInteriorClearanceMeters"]["width"] == 9.2, 'metrics["walkableInteriorClearanceMeters"]["width"] == 9.2')
    floor = node(document, "COLLIDER_InteriorFloor")
    floor_top = floor["translation"][1] + floor["extras"]["salimon"]["sizeMeters"][1] / 2
    deck_min, deck_max = node_position_bounds(document, geometry, "Deck_Walkable")
    require(abs(floor_top - deck_max[1]) < 1e-6, "visible floor and walking plane differ")
    _, seat_max = node_position_bounds(document, geometry, "Pilot_Seat_Back")
    require(seat_max[1] < windows["standingViewpointMeters"][1] - .35, 'seat_max[1] < windows["standingViewpointMeters"][1] - .35')
    assert_vectors_close(node(document,"COLLIDER_EnergyCore")["translation"],[-1.0,1.07*4/3.72,0.0])
    # Visual redesigns must remain inside the gameplay collider so the cage
    # never snags a walking route or protrudes through the player's capsule.
    core_collider = node(document, "COLLIDER_EnergyCore")
    core_center = core_collider["translation"]
    core_size = core_collider["extras"]["salimon"]["sizeMeters"]
    for name in mesh_names:
        if name.startswith(("Core_", "Energy_Core")) and name != "Core_Power_Conduits":
            core_min, core_max = node_position_bounds(document, geometry, name)
            for axis in range(3):
                require(core_min[axis] >= core_center[axis]-core_size[axis]/2-1e-6, name)
                require(core_max[axis] <= core_center[axis]+core_size[axis]/2+1e-6, name)
    for collider_name, mesh_name in (("COLLIDER_AftDoor", "Exit_Door"),
                                     ("COLLIDER_InteriorCeiling", "Ceiling_Inner")):
        collider = node(document, collider_name)
        center = collider["translation"]
        size = collider["extras"]["salimon"]["sizeMeters"]
        mesh_min, mesh_max = node_position_bounds(document, geometry, mesh_name)
        for axis in range(3):
            require(abs(center[axis]-size[axis]/2-mesh_min[axis]) < 1e-6, 'abs(center[axis]-size[axis]/2-mesh_min[axis]) < 1e-6')
            require(abs(center[axis]+size[axis]/2-mesh_max[axis]) < 1e-6, 'abs(center[axis]+size[axis]/2-mesh_max[axis]) < 1e-6')
    require(metrics["cargoRoom"] == manifest["interior"]["cargoRoom"], 'metrics["cargoRoom"] == manifest["interior"]["cargoRoom"]')
    for key, expected in spatial["cargoRoom"].items():
        actual = metrics["cargoRoom"][key]
        if isinstance(expected, list):
            assert_vectors_close(actual, expected)
        elif isinstance(expected, (int, float)):
            require(abs(actual-expected) < 1e-6, f"cargoRoom.{key}: authored bounds differ")
        else:
            require(actual == expected, f"cargoRoom.{key}: role differs")
    for name in (n.removeprefix("COLLIDER_") for n in spatial["colliders"] if n.startswith("COLLIDER_Cargo_")):
        lower, upper = node_position_bounds(document, geometry, name)
        proxy = node(document, "COLLIDER_" + name)
        assert_vectors_close(proxy["translation"], [(a+b)/2 for a,b in zip(lower,upper)])
        assert_vectors_close(proxy["extras"]["salimon"]["sizeMeters"], [b-a for a,b in zip(lower,upper)])
    require(abs(node_position_bounds(document, geometry, "Cargo_Deck")[1][1] - .23*4/3.72) < 1e-6, 'abs(node_position_bounds(document, geometry, "Cargo_Deck")[1][1] - .23*4/3.72) < 1e-6')
    # All corners of the expanded exterior remain within the flight sphere.
    require(max(sum(p[a]**2 for a in range(3))**.5
               for mesh in document["meshes"] for primitive in mesh["primitives"]
               for p in accessor_values(document, geometry, primitive["attributes"]["POSITION"])) < 16.0, 'max(sum(p[a]**2 for a in range(3))**.5                for mesh in document["meshes"] for primitive in mesh["primitives"]                for p in accessor_values(document, geometry, primitive["attributes"]["POSITION"])) < 16.0')
    # Profile geometry must remain shaped (not a relabeled cube) and compact.
    for name in ("Pilot_Seat_Shell","Pilot_Seat_Back","Pilot_Seat_Cushions",
                 "Pilot_Seat_Bolsters","Pilot_Seat_Headrest"):
        shaped=node(document,name)
        primitive=document["meshes"][shaped["mesh"]]["primitives"][0]
        require(document["accessors"][primitive["attributes"]["POSITION"]]["count"] >= 40, 'document["accessors"][primitive["attributes"]["POSITION"]]["count"] >= 40')
        lower,upper=node_position_bounds(document, geometry,name)
        require(lower[0] >= 1.55 and upper[0] <= 3.35, 'lower[0] >= 1.55 and upper[0] <= 3.35')
        require(lower[2] >= -.80 and upper[2] <= .80, 'lower[2] >= -.80 and upper[2] <= .80')
        require(upper[1] < windows["standingViewpointMeters"][1]-.25, 'upper[1] < windows["standingViewpointMeters"][1]-.25')
    require(metrics["cockpitInstruments"]["powerSource"] == "shared-thruster-command", 'metrics["cockpitInstruments"]["powerSource"] == "shared-thruster-command"')
    require(metrics["cockpitInstruments"]["additionalDraws"] == 0, 'metrics["cockpitInstruments"]["additionalDraws"] == 0')
    instruments = manifest["cockpitInstruments"]
    assert_vectors_close(instruments["pilotFacingTargetMeters"], windows["seatedViewpointMeters"])
    for label in ("Center", "Port", "Starboard"):
        name = f"Monitor_{label}"
        if label != "Center":
            require(instruments["sideYawDegrees"][name] == node(document, name)["extras"]["salimon"]["pilotFacingYawDegrees"], 'instruments["sideYawDegrees"][name] == node(document, name)["extras"]["salimon"]["pilotFacingYawDegrees"]')
        lower, upper = node_position_bounds(document, geometry, name)
        assert_vectors_close(instruments["surfaceBoundsMeters"][name]["min"], lower)
        assert_vectors_close(instruments["surfaceBoundsMeters"][name]["max"], upper)
    validate_center_monitor_scale(document, geometry, manifest)
    require(metrics["energyCore"]["role"] == "energy-storage", 'metrics["energyCore"]["role"] == "energy-storage"')
    require(metrics["energyCore"]["phase0"] == "visual-only", 'metrics["energyCore"]["phase0"] == "visual-only"')
    roof = node(document, "Hull_Roof")
    roof_primitive = document["meshes"][roof["mesh"]]["primitives"][0]
    roof_top_y = node_position_bounds(document, geometry, "Hull_Roof")[1][1]
    upward_top_vertices = 0
    positions = accessor_values(document, geometry, roof_primitive["attributes"]["POSITION"])
    normals = accessor_values(document, geometry, roof_primitive["attributes"]["NORMAL"])
    for position, normal in zip(positions, normals):
        if abs(position[1] - roof_top_y) < 1e-6 and abs(normal[1]) > .99:
            require(normal[1] > 0, "roof top winds inward in standard glTF viewers")
            upward_top_vertices += 1
    require(upward_top_vertices >= 4, 'upward_top_vertices >= 4')
    validate_sightlines(document, geometry)



REGISTRIES = {("ships", "ship", 1): validate_scout}


def main(export=None):
    """Validate GLB directly; optionally verify matching interchange and runtime code."""
    if export is None:
        metrics = validate_asset("ship.salimon-scout", extension_validators=REGISTRIES)
        export = ROOT / "export"
    else:
        from export_asset import resolve_manifest
        _, manifest = resolve_manifest("ship.salimon-scout", REPO)
        metrics = validate_manifest(manifest, export / "salimon_phase0_ship.glb", REPO,
                                    extension_validators=REGISTRIES)
    runtime = export / "salimon_phase0_ship.glb"
    document, binary = read_document(runtime)
    interchange = json.loads((export / "salimon_phase0_ship.gltf").read_text())
    expected = json.loads(json.dumps(document))
    expected["buffers"][0]["uri"] = "salimon_phase0_ship.bin"
    require(interchange == expected, "glTF and GLB documents differ")
    require((export / "salimon_phase0_ship.bin").read_bytes() == binary,
            "glTF and GLB geometry payloads differ")
    if export == ROOT / "export":
        from spatial_contracts import anchors_rust_source, cargo_rust_source, sidecar_json, thruster_rust_source
        spatial = build_spatial_contracts(document, json.loads((SCOUT / "preservation.json").read_text()))
        require((ROOT / "spatial-contracts.json").read_text() == sidecar_json(spatial), "spatial sidecar is stale")
        for name, source in (("ship_anchors.rs", anchors_rust_source), ("cargo_layout.rs", cargo_rust_source),
                             ("thruster_collision.rs", thruster_rust_source)):
            require((REPO / "client/character/src" / name).read_text() == source(spatial), f"{name}: generated layout is stale")
    print(json.dumps({"asset": "ship.salimon-scout", "status": "passed", "metrics": metrics}, sort_keys=True))


if __name__ == "__main__":
    try:
        main()
    except (ValidationError, OSError, ValueError) as exc:
        print(f"scout validation failed: {exc}", file=sys.stderr)
        sys.exit(1)
