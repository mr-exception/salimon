# Personal planetary chart (#626)

Backslash opens/closes the powered Scout cockpit chart. It uses the existing
physical holographic projector: only personal field marks appear, one selected
mark at a time. M cycles marks, R edits its personal note, Enter commits text or
selects a manual navigation target, Backspace edits text, and Delete removes a
personal mark. Escape closes and discards an uncommitted draft. Comma, outside
the chart, deliberately inspects a nearby aimed physical site against the active
target (or latest personal mark). F10/F11 continue to create suit observations
outside chart/modal use; they do not consume equipment slots or scanner pulses.

Chart controls require the canonical cockpit, normal living gameplay, power and
no construction/modal conflict. Opening clears held flight/mining controls; it
does not toggle engines, issue steering commands or fly a route. While the chart
is open, other gameplay commands cannot accidentally move, mine or interact.
Power loss prevents changes/projection and retains source history. Personal notes
are typed through native winit keyboard text or the same production session
editor via `chart_text`; protocol input does not establish physical keyboard QA.

## Knowledge and uncertainty

Universe `surface_chart.rs` owns personal annotations, immutable edit/inspection
receipts, deletion tombstones and manual target history. It consumes the suit
owner's [measured observations](suit-observations.md), never a catalog POI search.
Edits change neither original observations nor terrain, organisms, definitions,
scanner cadence, discovery or Earth knowledge. Expired scanner contacts do not
delete personal history. Deletion hides a mark, preserves its history and clears
an active target; IDs are never reused. Draft text and UI selection are transient.

For a measured spot, three projector circles show its relative isotropic error
volume. The current ship reference is quantized using the same navigation grid;
its own error is added conservatively to the stored observation error for relative
ranges and geometry. True ship translation is used only to anchor the physical
projector mesh, not presented as an exact navigation fix.
For a bearing-only mark, rings at illustrative 100/1,000/10,000 m sample ranges
show the angular uncertainty cone. Those ranges are drawing samples, not three
objects, measured distances or target coordinates. Angular errors of 90 degrees
or more suppress the directional cone and say UNCONSTRAINED instead of drawing
a narrow/inverted tangent cone. Readout explicitly states
RANGE UNKNOWN, origin error and provenance; Enter cannot invent a target distance.
A valid measured spot or inspected point can become a manual target, retaining
its measurement/error receipt. Readout shows estimated range/azimuth/elevation
and the active personal target; no pathfinder, autopilot or perfect GPS is added.

## Deliberate close inspection

Comma evaluates an actual aimed ray within 10 m. Geological membership comes
from the existing published terrain hit and its landmark extent; large regions
do not require standing at an inaccessible center. Biological membership requires
an actual published source-part box hit, not empty space in a colony union box.
Terrain, other physical organisms, authored hull/doors and installed construction
can obstruct the source. The observed point must match the original mark's area
or bearing/error cone. A measured navigation reference is required.

The receipt binds stable source ID, geological/biological kind, exact publication,
TT epoch and an uncertain optical position. A 0.5 m quantized local optical offset
adds a conservative 0.5 m to the observer's position envelope. It verifies a known
physical source identity, never a scientific species/mineral identification.
Repeated inspections retain earlier receipts. No site is discovered by merely
opening the chart, sighting scenery, scanning or loading a catalog.

The existing frozen Vesper publication is not modified to add demonstration sites.
Positive inspection tests use a real authored published terrain fixture through
the same universe receipt owner; canonical runtime restoration additionally
checks the current actual site/publication and physical extent. Production wgpu
captures exercise real arbitrary bookmarks, note editing, uncertain spot/cone
presentation, target admission, deletion and reload. No visited-Vesper-site
capture or new planet/world-selection integration is claimed.

## Save and validation

Runtime `session/surface_chart.rs` owns physical admission, source adapters,
readout/geometry mapping and validation. Schema 28 saves the separate chart paired
with the suit book. Old canonical sources without it initialize personal entries
from existing observations; missing current source, dangling/duplicate IDs,
invalid notes/time/receipts and incompatible site publications reject atomically.
Proof positions must remain within the bound source extent plus measured error.
Exact original observations and scanner history remain independently validated.

There are at most 1,024 personal marks, 32 edits and 32 inspections per mark,
4,096 total edit records and 128 manual target receipts. Limits decline further
changes explicitly without dropping source history or partly editing state.
Notes permit 256 Unicode characters without control characters. Duplicate JSON
annotation keys reject; ordered identity lookup is deterministic. Targets require
an original measured spot or a retained close-inspection location receipt.

Run universe/runtime `surface_chart` tests, workspace build/tests, all-target
Clippy/fmt and docs/base-impact checks; native scenario:
`scenarios/evidence/planetary-chart.json`. Its explicit `planetary-chart` setup
creates both marks with ordinary walking/suit commands and initializes cockpit
review position. It does not claim the setup teleport is a boarding route; that
route remains covered by existing canonical journey evidence. Physical keys/OS
behavior (#557), subjective presentation (#561) and M1/Metal budgets (#563) remain
separate release qualification, not cloud acceptance claims.
