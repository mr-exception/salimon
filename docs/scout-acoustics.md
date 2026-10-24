# Scout tactile sounds and lightweight acoustics (#583)

Runtime observes real source state; audio never grants physics, power or action
permission. Seven travel channels follow bunk, table, padded seat, privacy screen,
shutter, locker and cargo-door fractions. Release/reversal and endpoint transients
require actual changed fractions. Unchanged blocked input emits no cue; halted
motion retires its travel voice on the next observed tick with a short click-free
fade. Resume in mid-travel after load does not replay a release/latch.

Seat/bunk harness toggles and actual successful mount attachment changes emit
physical contact sounds. The mount adapter journals only changed authoritative
attachment state; rejected inputs and ordinary carrying motion do not click.
Original mount IDs/source positions remain authoritative, for all supported
mounts (the journal does not truncate the first eight objects). The one-player
interaction journal coalesces multiple changes before the next observation to
the latest contact position. Existing feedback/prompts are the visual alternatives.

`scout_audio/acoustics.rs` derives cargo separation from the physical bulkhead
coordinate and actual door opening, with the finite privacy-screen aperture
from its physical nook. Closed cargo transmission is 0.22 gain / 0.08 low-pass
coefficient; closed privacy screen is 0.72 / 0.4. Open boundaries approach 1.
Multiple crossed partitions multiply, up to eight; finite spans do not filter
sounds outside the screen. These are restrained authored presentation values,
not simulated atmospheric pressure, ray tracing or measured material acoustics.

Cockpit, connected habitation/engineering and aft cargo are stable geometric
area tags; the cockpit boundary follows the authored seat region. Real closed
partitions govern separation; there is no imaginary wall between cockpit and
habitation. Future modular room geometry (#545) can supply equivalent partitions;
current tests cover multiple boundaries and finite apertures. Distance attenuation
and ship-local stereo panning still apply. Cargo and habitation footsteps have
different timbres. No random creaking, music, chatter or invented operating source
is added. Powered ambience follows real Core/equipment state; manual mechanism
sounds remain possible during blackout if the mechanism actually moves.

## Continuous playback and bounded work

The existing CPAL stream now owns 27 fixed voice slots (including three survival
slots); runtime admits at most 24 scene voices and 12 pending events, with alerts
prioritized. Dense incidental transients may be coalesced/dropped at that budget.
Mechanism friction uses restrained filtered noise/resonance, with heavier shutter
and bulkhead travel/end stops, small latch/clamp contact and fabric restraints.
There is no queued file/process playback or per-callback allocation/locking/logging.
A per-voice one-pole filter is calibrated to 48 kHz and adjusted to actual device
rate; its coefficient is prepared once per callback, not once per sample.

Callback mixer storage is 2,736 bytes; atomic mailbox 568 bytes. Input sources,
volume/mute, pause, focus, exterior/terminal and stale-control retirement remain
bounded. Reset/restore increments an excluded observational audio epoch even
when restored simulation time is identical. Checkpoints contain no audio events,
voices, counters or device handles. The scene/audio system does not advance time
or change save mass, joules, identity or pending mechanism movement.

## Evidence and review

E2E/benchmarks retain device-playback suppression. Explicit E2E captures expose
`preview_only:true`, observed gains/filters/events and area tags through the same
mapping; these previews are not OS playback. A separate CPAL output test captures
the configured virtual ALSA file sink, retaining original float32 PCM, listening
WAV and callback counters. That sink is unpaced and reports one-frame callbacks
and overruns; it cannot certify physical latency, device scheduling or speaker
quality. Production-control WAV sequences and isolated synthetic timbre/stress
references are labeled separately in [the report](../reports/issue-583/README.md).

Human #561 owns listening/timbre/stereo approval on suitable headphones/speakers;
#557 real device behavior/latency and #563 hardware/CPU budgets. Existing volume
and master/ship mute controls remain the user controls. Original WAVs and native
images are review candidates, not promoted baselines or hardware signoff.
