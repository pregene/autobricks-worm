"""Run Rust policies and native FSKit callback tests."""

import os
from pathlib import Path
import subprocess
import sys

from build_macos import compile_adapter


def main():
    if sys.platform != "darwin":
        raise ValueError("This test command requires macOS.")
    if len(sys.argv) != 1:
        raise ValueError("Usage: scripts/test-macos.sh")
    root = Path(__file__).resolve().parent.parent
    for command in (["cargo", "test", "--locked"],
                    ["cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
                    ["cargo", "build", "--locked", "--lib"]):
        subprocess.run(command, cwd=root, check=True)
    target = Path(os.environ.get("CARGO_TARGET_DIR", root / "target"))
    if not target.is_absolute():
        target = root / target
    artifact = compile_adapter(root, target / "debug", target / "debug/libautobricks_worm.a", tests=True)
    subprocess.run([str(artifact)], check=True)


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"macOS test failed: {error}", file=sys.stderr)
        sys.exit(1)
