"""Option A resource/tool palette in linear RGB; geometry stays asset-owned."""
RESOURCE_COLORS = {
    'Iron_Ore': (.25, .22, .19, 1),
    'Iron_Inclusions': (.49, .47, .43, 1),
    'Oxide_Crust': (.55, .24, .08, 1),
    'Silicate_Dark': (.24, .30, .25, 1),
    'Silicate_Light': (.52, .55, .43, 1),
    'Ice_Blue': (.13, .40, .58, 1),
    'Ice_Frost': (.64, .82, .88, 1),
}
# Atlas colors carry the manufactured palette. Material factors stay white so
# the renderer does not multiply graphite by graphite and crush grip readability.
TOOL_TILES = ((.24, .30, .37), (.09, .12, .16), (.52, .57, .62), (.83, .53, .18))


def tool_pixel(x, y):
    tile = x // 128 + 2 * (y // 128)
    u, v = x % 128, y % 128
    # Deliberate broad seams/ribs replace stochastic grain and tiny scuffs.
    value = 1.0
    if tile == 0 and (u in (14, 15, 111, 112) or v in (14, 15, 111, 112)):
        value = .78
    elif tile == 1 and v % 16 < 3:
        value = .82
    return (*[c * value for c in TOOL_TILES[tile]], 1.0)
