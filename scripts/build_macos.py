"""Compile the native Apple FSKit adapter using the installed Apple SDK."""

import json
import os
from pathlib import Path
import subprocess


def compile_adapter(root, output, rust_library, *, tests=False, env=None, target=None):
    environment = dict(os.environ if env is None else env)
    sdk = subprocess.check_output(["xcrun", "--sdk", "macosx", "--show-sdk-path"], text=True).strip()
    if not (Path(sdk) / "System/Library/Frameworks/FSKit.framework").exists():
        raise ValueError("Install Apple developer tools with the macOS 15.4 or newer SDK for FSKit.")
    info = json.loads(subprocess.check_output(["xcrun", "swiftc", "-print-target-info"], text=True))
    triple = info["target"]["triple"]
    architecture = triple.split("-", 1)[0]
    if target:
        architecture = {"aarch64-apple-darwin": "arm64", "x86_64-apple-darwin": "x86_64"}[target]
    minimum = environment.get("MACOSX_DEPLOYMENT_TARGET", triple.split("macosx", 1)[-1])
    minimum = max((minimum, "15.4"), key=lambda value: tuple(map(int, value.split("."))))
    output.mkdir(parents=True, exist_ok=True)
    native = root / "src/macos/fskit"
    command = ["xcrun", "swiftc", "-sdk", sdk, "-target", f"{architecture}-apple-macosx{minimum}",
               "-module-cache-path", str(output / "swift-cache"),
               "-import-objc-header", str(native / "PolicyBridge.h"),
               "-warnings-as-errors", *map(str, sorted(native.glob("*.swift")))]
    if tests:
        artifact = output / "fskit-tests"
        command.extend(map(str, sorted((root / "tests/macos").glob("*.swift"))))
    else:
        artifact = output / "libab_worm_fskit.dylib"
        command.extend(["-emit-library", "-emit-module", "-emit-module-path",
                        str(output / "AutobricksWORMFSKit.swiftmodule"),
                        "-module-name", "AutobricksWORMFSKit",
                        "-Xlinker", "-install_name", "-Xlinker", "@rpath/libab_worm_fskit.dylib"])
    command.extend([str(rust_library), "-o", str(artifact)])
    subprocess.run(command, cwd=root, env=environment, check=True)
    return artifact
