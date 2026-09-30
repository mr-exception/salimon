# Native game builds

Build on the destination OS. Python 3.10+ and stable Rust 1.89+ (installed through
rustup) are required on all platforms. The repository's `rust-toolchain.toml`
selects stable and `Cargo.lock` fixes the dependency versions. The first build
requires access to crates.io. No Python packages, IDE, Blender, or game editor
are required. Run from the repository root:

```sh
python3 scripts/build_game.py --profile debug
python3 scripts/build_game.py --profile release
```

On Windows use `python` instead of `python3` (or `py -3`). The interface and
profiles are identical. Debug is the default; release enables Cargo's optimized
profile. The command also works from another directory using an absolute path
to the script; Cargo always runs at the repository root.

## Platform prerequisites

| Platform | Build tools | Runtime |
| --- | --- | --- |
| macOS | Xcode Command Line Tools (`xcode-select --install`), Python 3, rustup | Graphical desktop and Metal-capable GPU |
| Windows | Python 3, rustup with the native MSVC toolchain, Visual Studio 2022 Build Tools with **Desktop development with C++**, MSVC compiler and Windows SDK | Graphical desktop, current GPU driver with DX12 or Vulkan support; Visual C++ 2015–2022 Redistributable if absent |
| Debian/Ubuntu | Python 3, rustup, `build-essential`; packages below | X11 or Wayland desktop and a working Vulkan driver |

Debian-based Linux setup (in addition to installing Rust via rustup):

```sh
sudo apt-get update
sudo apt-get install -y python3 build-essential libxkbcommon-dev libwayland-dev \
  libx11-dev libxi-dev libxcursor-dev libxrandr-dev libvulkan1 mesa-vulkan-drivers
```

Mesa provides Vulkan for supported open-source drivers and software rendering.
For proprietary GPUs install the vendor's Vulkan driver instead. Development
packages above also supply the required desktop runtime libraries. A machine
that only runs a copied build needs `libxkbcommon0`, `libwayland-client0`,
`libx11-6`, `libxi6`, `libxcursor1`, `libxrandr2`, `libvulkan1` and its GPU's
Vulkan driver. Headless builds are supported; interactive execution needs a
display. Runtime distribution compatibility is limited to the build host's
library/OS baseline or newer compatible systems; no static Linux bundle is made.

## Outputs and execution

Default output is `artifacts/build/<Rust host triple>/<debug|release>/`:

- `salimon-client` (macOS/Linux) or `salimon-client.exe` (Windows).
- `build-info.json` with host, profile, Rust version, and executable SHA-256.
- `README.txt` with local launch requirements.

The executable embeds its GLB, shader and world data and generates surface
textures; it needs no repository-relative asset files. Copy the output folder
to a compatible machine and launch the executable directly, from any working
directory. Linux/macOS file transfers must preserve executable permission
(`chmod +x salimon-client` if necessary). This is a raw executable, not a macOS
`.app` bundle or Windows installer.

For example, on x86-64 Debian Linux:

```sh
./artifacts/build/x86_64-unknown-linux-gnu/release/salimon-client
```

Override the exact staging directory (paths with spaces are supported):

```sh
python3 scripts/build_game.py --profile release --output "artifacts/my build"
```

Cargo intermediates remain in `target/` or `CARGO_TARGET_DIR`. The script uses
Cargo's reported executable path rather than assuming a fixed target directory,
and explicitly builds for the Rust host triple even if `.cargo/config.toml`
sets another target. It does not cross-compile or produce universal binaries.
macOS Intel/Apple Silicon and Windows architectures require their respective
native Rust host toolchains. Do not use a cross-host toolchain override.

## Failures and checks

Missing Rust, outdated Rust, missing macOS tools, missing Debian C linker,
unsupported hosts and non-MSVC Windows toolchains fail with a clear message.
Other missing compiler/SDK dependencies are reported by Cargo's preserved
compiler/linker diagnostics. Build and staging failures return a nonzero exit
status. A failed compilation leaves any **previous** staged build intact; it
does not create a new successful artifact. Files in a successful output folder
are replaced; use separate output directories for builds you want to retain.

- `cargo`/`rustc` missing: reopen the terminal after rustup installation.
- `link.exe` or Windows SDK missing: install the C++ workload above, then reopen
  the terminal; ensure Rust's host ends in `-pc-windows-msvc`.
- macOS tools missing: run `xcode-select --install`; check `xcode-select -p`.
- Cargo download error: restore network access and rerun; `--locked` remains set.
- Linux launch failure: verify desktop libraries, `DISPLAY`/Wayland session,
  Vulkan support and driver installation; a successful build does not prove GPU
  availability.
- macOS unsigned executable/security prompt: allow a trusted local build in
  macOS security settings. Signing/notarization is not provided.

Run fast build-script and existing runner contracts without a GPU:

```sh
python3 -m unittest discover -s scripts -p 'test_*.py'
```

`.github/workflows/native-build.yml` runs these contracts and Rust quality gates,
builds/stages debug and release on the native macOS runner, Windows (x86-64 MSVC)
and Ubuntu (x86-64), and uploads the runnable folders. Linux also launches the
staged release executable under Xvfb with Mesa software Vulkan and runs the
existing landed-Earth gameplay scenario. It uploads logs/results even on failure.
No graphical launch is claimed for macOS/Windows CI; validate native GPU/window
behavior with the [manual smoke checks](../README.md#native-smoke-check) and the
[E2E runner](README.md) on those desktops. Linux software rendering is a startup
check, not a hardware performance/visual-quality benchmark. Packaged OS-input
automation remains separate work in issue #32.

Signing, notarization, installers, automatic updates, non-Debian distributions
and application store packaging are outside this build interface.
