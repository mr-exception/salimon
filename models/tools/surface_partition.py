"""Convex planar panel subtraction with linearly interpolated UV/position payloads."""

AREA = 1e-10  # square meters, matching authored quality policies

def area(points):
    return sum(a[0]*b[1]-a[1]*b[0] for a, b in zip(points, points[1:]+points[:1])) / 2


def split(poly, a, b):
    """Split vertices (position, UV) by one projected edge, preserving UVs."""
    inside, outside = [], []
    def side(p):
        return (b[0]-a[0])*(p[0][1]-a[1])-(b[1]-a[1])*(p[0][0]-a[0])
    for p, q in zip(poly, poly[1:]+poly[:1]):
        sp, sq = side(p), side(q)
        (inside if sp >= 0 else outside).append(p)
        if (sp < 0) != (sq < 0):
            t = sp/(sp-sq)
            point = tuple(tuple(x+t*(y-x) for x, y in zip(v, w)) for v, w in zip(p, q))
            inside.append(point)
            outside.append(point)
    return inside, outside


def subtract(poly, cutter):
    if area(cutter) < 0:
        cutter = cutter[::-1]
    remainder, pieces = poly, []
    for a, b in zip(cutter, cutter[1:]+cutter[:1]):
        if not remainder:
            break
        remainder, outside = split(remainder, a, b)
        if len(outside) >= 3 and abs(area([v[0] for v in outside])) > AREA:
            pieces.append(outside)
    # No positive-area intersection: preserve original tessellation/UV exactly.
    if len(remainder) < 3 or abs(area([v[0] for v in remainder])) <= AREA:
        return [poly]
    return pieces
