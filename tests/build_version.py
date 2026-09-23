"""Exercise native build entry points and version locking in an isolated copy."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


class BuildVersionTest(unittest.TestCase):
    def test_success_concurrency_and_failure(self):
        source = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory(prefix="ab-worm-build-") as directory:
            root = Path(directory)
            for name in (
                "Cargo.toml", "Cargo.lock", "VERSION", "build.rs",
                "build.sh", "build.ps1", "scripts", "src",
            ):
                item = source / name
                if item.is_dir():
                    shutil.copytree(item, root / name)
                else:
                    shutil.copy2(item, root / name)
            initial = (root / "VERSION").read_text().strip()
            major, minor, patch = map(int, initial.split("."))
            env = dict(os.environ, CARGO_TARGET_DIR=str(root / "target"))
            command = (
                ["powershell", "-NoProfile", "-ExecutionPolicy", "Bypass", "-File", str(root / "build.ps1")]
                if os.name == "nt" else [str(root / "build.sh")]
            ) + ["--offline", "--locked"]

            def launch():
                return subprocess.Popen(command, cwd=root, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)

            first = launch()
            output, _ = first.communicate(timeout=180)
            self.assertEqual(first.returncode, 0, output)
            self.assertEqual((root / "VERSION").read_text().strip(), f"{major}.{minor}.{patch + 1}")

            processes = [launch(), launch()]
            for process in processes:
                output, _ = process.communicate(timeout=180)
                self.assertEqual(process.returncode, 0, output)
            expected = f"{major}.{minor}.{patch + 3}"
            self.assertEqual((root / "VERSION").read_text().strip(), expected)
            executable = root / "target" / "debug" / ("ab-worm.exe" if os.name == "nt" else "ab-worm")
            banner = subprocess.check_output([str(executable), "--version"], text=True)
            self.assertEqual(banner.strip(), f"Autobricks WORM Filesystem {expected} (C) 2026 Autobricks, Co.")

            (root / "src" / "main.rs").write_text("invalid Rust source\n")
            failed = launch()
            output, _ = failed.communicate(timeout=180)
            self.assertNotEqual(failed.returncode, 0, output)
            self.assertEqual((root / "VERSION").read_text().strip(), expected)


if __name__ == "__main__":
    unittest.main()
