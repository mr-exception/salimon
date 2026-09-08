"""Build interface contracts independent of a Rust toolchain or graphical desktop."""

import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import build_game


class BuildTests(unittest.TestCase):
    def rust_info(self, host, version="1.89.0"):
        return f"rustc {version}\nhost: {host}\nrelease: {version}"

    def test_platform_prerequisites_and_native_hosts(self):
        for system, host in (("Darwin", "aarch64-apple-darwin"),
                             ("Windows", "x86_64-pc-windows-msvc"),
                             ("Linux", "x86_64-unknown-linux-gnu")):
            with self.subTest(system=system), patch("build_game.platform.system", return_value=system), \
                    patch("build_game.shutil.which", return_value="tool"), \
                    patch("build_game.Path.exists", return_value=True), \
                    patch("build_game.command_output", return_value=self.rust_info(host)) as output:
                self.assertEqual(build_game.prerequisites()[1], host)
                self.assertEqual(output.call_args_list[0].args[0], ["rustc", "--version", "--verbose"])
                self.assertEqual(output.call_count, 2 if system == "Darwin" else 1)

    def test_missing_tools_old_rust_and_unsupported_hosts_fail(self):
        with patch("build_game.platform.system", return_value="Windows"):
            with patch("build_game.shutil.which", return_value=None):
                with self.assertRaisesRegex(build_game.BuildError, "Missing cargo"):
                    build_game.prerequisites()
            with patch("build_game.shutil.which", return_value="tool"), \
                    patch("build_game.command_output", return_value=self.rust_info("x86_64-pc-windows-msvc", "1.88.0")):
                with self.assertRaisesRegex(build_game.BuildError, "1.89"):
                    build_game.prerequisites()
            with patch("build_game.shutil.which", return_value="tool"), \
                    patch("build_game.command_output", return_value=self.rust_info("x86_64-pc-windows-gnu")):
                with self.assertRaisesRegex(build_game.BuildError, "MSVC"):
                    build_game.prerequisites()
        with patch("build_game.platform.system", return_value="FreeBSD"):
            with self.assertRaisesRegex(build_game.BuildError, "Unsupported"):
                build_game.prerequisites()
        with patch("build_game.platform.system", return_value="Linux"), patch("build_game.Path.exists", return_value=False):
            with self.assertRaisesRegex(build_game.BuildError, "Debian"):
                build_game.prerequisites()

    def test_artifact_discovery_uses_cargo_output_not_a_fixed_target_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "custom target" / "salimon-client"
            binary.parent.mkdir()
            binary.write_bytes(b"native executable")
            messages = [{"reason": "compiler-artifact", "target": {"name": "some-library", "kind": ["lib"]}},
                        {"reason": "compiler-artifact", "target": {"name": "salimon-client", "kind": ["bin"]},
                         "executable": str(binary)}]
            for profile in ("debug", "release"):
                with self.subTest(profile=profile), patch("build_game.subprocess.Popen") as popen:
                    process = popen.return_value.__enter__.return_value
                    process.stdout = io.StringIO("\n".join(json.dumps(message) for message in messages))
                    process.wait.return_value = 0
                    self.assertEqual(build_game.compile_game("x86_64-unknown-linux-gnu", profile), binary)
                    argv = popen.call_args.args[0]
                    self.assertIn("--locked", argv)
                    self.assertEqual(argv[argv.index("--target") + 1], "x86_64-unknown-linux-gnu")
                    self.assertEqual("--release" in argv, profile == "release")

    def test_cargo_error_or_missing_artifact_is_not_success(self):
        for code in (0, 101):
            with self.subTest(code=code), patch("build_game.subprocess.Popen") as popen:
                process = popen.return_value.__enter__.return_value
                process.stdout = io.StringIO("")
                process.wait.return_value = code
                with self.assertRaises(build_game.BuildError):
                    build_game.compile_game("x86_64-unknown-linux-gnu", "debug")

    def test_stage_produces_standalone_artifact_for_each_platform(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "binary"
            source.write_bytes(b"embedded assets")
            source.chmod(0o755)
            for host in ("aarch64-apple-darwin", "x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"):
                with self.subTest(host=host):
                    destination = root / host / "release"
                    result = build_game.stage(source, destination, host, "release", "rustc 1.89.0")
                    self.assertEqual(result.read_bytes(), source.read_bytes())
                    self.assertEqual(result.suffix, ".exe" if "windows" in host else "")
                    if build_game.os.name != "nt":
                        self.assertEqual(result.stat().st_mode & 0o777, source.stat().st_mode & 0o777)
                    info = json.loads((destination / "build-info.json").read_text())
                    self.assertEqual(info["host"], host)
                    self.assertEqual(info["sha256"], build_game.hashlib.sha256(source.read_bytes()).hexdigest())
                    self.assertEqual(len(list(destination.iterdir())), 3)

    def test_failed_build_does_not_replace_previous_output(self):
        with tempfile.TemporaryDirectory() as directory:
            previous = Path(directory) / "salimon-client"
            previous.write_bytes(b"previous build")
            with patch("build_game.prerequisites", return_value=("Linux", "x86_64-unknown-linux-gnu", "rustc")), \
                    patch("build_game.compile_game", side_effect=build_game.BuildError("Cargo failed")), \
                    contextlib.redirect_stderr(io.StringIO()) as error:
                self.assertEqual(build_game.main(["--output", directory]), 1)
                self.assertIn("Cargo failed", error.getvalue())
            self.assertEqual(previous.read_bytes(), b"previous build")

    def test_prerequisite_command_failure_is_actionable(self):
        with patch("build_game.subprocess.check_output", side_effect=subprocess.CalledProcessError(1, ["xcode-select"], output="tools missing")):
            with self.assertRaisesRegex(build_game.BuildError, "tools missing"):
                build_game.command_output(["xcode-select", "-p"])


if __name__ == "__main__":
    unittest.main()
