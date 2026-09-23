"""Build ab-worm, then record its incremented version on success."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile

from build_lock import exclusive_build_lock
from build_macos import compile_adapter


def main():
    parser = argparse.ArgumentParser(description="Build ab-worm and increment VERSION on success.")
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--locked", action="store_true")
    parser.add_argument("--target")
    parser.add_argument("--features")
    options = parser.parse_args()
    native_macos = sys.platform == "darwin" and (
        options.target is None or options.target in ("aarch64-apple-darwin", "x86_64-apple-darwin")
    )
    cargo_args = [flag for flag in ("--release", "--offline", "--locked")
                  if getattr(options, flag[2:])]
    if options.target:
        cargo_args.extend(["--target", options.target])
    if options.features:
        cargo_args.extend(["--features", options.features])
    root = Path(__file__).resolve().parent.parent
    # Keep the lock outside target/ so cargo clean cannot remove an active lock.
    with exclusive_build_lock(root / ".build.lock"):
        version_path = root / "VERSION"
        current = version_path.read_text().strip()
        if not re.fullmatch(r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)", current):
            raise ValueError("VERSION must contain major.minor.patch")
        major, minor, patch = map(int, current.split("."))
        if max(major, minor, patch + 1) > 2**64 - 1:
            raise ValueError("VERSION component overflow")
        version = f"{major}.{minor}.{patch + 1}"
        env = dict(os.environ, AB_WORM_BUILD_VERSION=version)
        result = subprocess.run(
            ["cargo", "build", "--bin", "ab-worm", *(["--lib"] if native_macos else []), *cargo_args],
            cwd=root,
            env=env,
            check=False,
        )
        if result.returncode:
            return result.returncode
        if native_macos:
            target_dir = Path(env.get("CARGO_TARGET_DIR", root / "target"))
            if not target_dir.is_absolute():
                target_dir = root / target_dir
            if options.target:
                target_dir /= options.target
            output_dir = target_dir / ("release" if options.release else "debug")
            compile_adapter(root, output_dir, output_dir / "libautobricks_worm.a",
                            env=env, target=options.target)
        temporary = None
        try:
            with tempfile.NamedTemporaryFile(mode="w", dir=root, prefix=".VERSION-", delete=False) as output:
                temporary = Path(output.name)
                output.write(version + "\n")
                output.flush()
                os.fsync(output.fileno())
            temporary.replace(version_path)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)
        print(f"Built Autobricks WORM Filesystem {version}")
        return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"Build failed: {error}", file=sys.stderr)
        sys.exit(1)
