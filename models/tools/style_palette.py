"""Proposed Option A linear colors; functional material roles are preserved."""
SCOUT_COLORS = {
    'Hull Graphite': (0.09, 0.12, 0.16, 1),
    'Hull Ceramic': (0.24, 0.30, 0.37, 1),
    'Copper Accent': (0.83, 0.53, 0.18, 1),
    'Interior Dark': (0.09, 0.12, 0.16, 1),
    'Interior Light': (0.72, 0.69, 0.60, 1),
    'Honey Wood': (0.24, 0.30, 0.37, 1),
    'Petrol Teal': (0.14, 0.65, 0.72, 1),
    'Terracotta Upholstery': (0.24, 0.30, 0.37, 1),
}


def apply(materials):
    """Blender material collection; no geometry, alpha, emission or texture edits."""
    for material in materials:
        if material.name not in SCOUT_COLORS:
            continue
        color = SCOUT_COLORS[material.name]
        material.diffuse_color = color
        if material.use_nodes:
            material.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = color
