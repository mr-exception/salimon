# Packaged black-box smoke

Build/stage with [build_game.py](BUILDING.md), then launch the standalone artifact:

```sh
python3 scripts/packaged_smoke.py --binary artifacts/build/x86_64-unknown-linux-gnu/release/salimon-client
```

Use `python` on Windows. No Cargo, IDE, source assets, Python dependencies, E2E
launch flags, stdin protocol, or game-native automation actions are used. The
runner launches the absolute executable from a fresh empty working directory,
waits for native window creation, renderer initialization and a presented frame,
then captures startup. Real OS F2 input switches to the precision tour and back
to playable first-person gameplay. Game log acknowledgements prove both keys
were handled by the native event loop. A gameplay screenshot is captured and the game must still be running. The
runner then terminates its launched process as cleanup. An expected cleanup
termination code is recorded separately from the successful smoke checks;
this smoke does not claim to test normal window-close behavior.
This intentionally small check complements [deterministic E2E scenarios](README.md).
It does not assert pixel fidelity, performance, or the complete landing loop.

| Platform | Required desktop/input/capture tooling |
| --- | --- |
| Debian/Ubuntu X11 | Visible unobscured desktop, `xdotool`, ImageMagick `import`; install `sudo apt install xdotool imagemagick` |
| macOS | Logged-in graphical desktop, `osascript`/System Events, `screencapture`; grant Accessibility/Automation and Screen Recording permissions to the launching terminal |
| Windows | Interactive unlocked desktop, Windows PowerShell; user32 focuses the launched PID's window and sends key events; System.Drawing captures the desktop |

Linux Wayland-only sessions and headless desktops without a display fail
explicitly. A helper/tool/permission/capture failure never becomes a passing
skip. The runner focuses the launched game and changes focus on your desktop;
use a dedicated test session. Screenshots capture the whole desktop and may
include other visible content. Keep the game unobscured. macOS and Windows
adapters need validation on the target desktop; CI currently exercises Linux
only and does not claim graphical coverage on those two hosts.

For Linux CI or local software-rendered startup validation:

```sh
sudo apt install xvfb xdotool imagemagick mesa-vulkan-drivers
WGPU_BACKEND=vulkan xvfb-run -a -s '-screen 0 1920x1080x24' \
  python3 scripts/packaged_smoke.py --binary artifacts/build/release/salimon-client --timeout 90
```

The Native builds workflow runs this on the staged Linux release package and
uploads evidence even on failure. Xvfb/Mesa coverage does not establish hardware
GPU fidelity. macOS/Windows builds and Python contracts run in CI; native input
smoke runs there remain local opt-in.

`--artifacts DIR` changes the default `artifacts/packaged-smoke/` root;
`--timeout SECONDS` sets the bounded gameplay deadline (default 60, maximum 600).
Capture helpers add up to five seconds per capture and input helpers up to five
seconds per call; termination adds at most four seconds. Each unique `run-*`
directory has `result.json`, stdout/stderr logs, startup/gameplay PNGs and capture
logs. Failures attempt `failure.png` before cleanup. JSON stdout reports checks,
PID, platform, duration, exit code and the precise failing condition. Exit status
is nonzero on launch, rendering, capture, input, deadline or shutdown failure.
Processes are terminated/killed on failure; no orphan game is intentionally left.
