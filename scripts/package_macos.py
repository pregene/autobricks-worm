"""Build a containing app and native FSKit extension for local registration."""
import os
from pathlib import Path
import plistlib
import subprocess


def build_app(root, output, version, *, env=None, target=None):
    environment = dict(os.environ if env is None else env)
    app = output / "Autobricks WORM.app"
    extension = app / "Contents/Extensions/AutobricksWORM.appex"
    for bundle in (app, extension):
        (bundle / "Contents/MacOS").mkdir(parents=True, exist_ok=True)
    sdk = subprocess.check_output(["xcrun", "--sdk", "macosx", "--show-sdk-path"], text=True).strip()
    native = root / "src/macos/fskit"
    compiler = ["xcrun", "swiftc", "-sdk", sdk, "-module-cache-path", str(output / "swift-cache")]
    architecture = {"aarch64-apple-darwin": "arm64", "x86_64-apple-darwin": "x86_64"}.get(target)
    if architecture is None:
        import platform
        architecture = platform.machine()
    compiler += ["-target", f"{architecture}-apple-macosx26.0", "-warnings-as-errors"]
    subprocess.run([*compiler, "-parse-as-library", "-application-extension",
                    "-import-objc-header", str(native / "PolicyBridge.h"),
                    *map(str, sorted(native.glob("*.swift"))),
                    *map(str, sorted((root / "src/macos/extension").glob("*.swift"))),
                    str(output / "libautobricks_worm.a"),
                    "-o", str(extension / "Contents/MacOS/ab-worm-extension")],
                   env=environment, check=True)
    subprocess.run([*compiler, str(root / "src/macos/host/main.swift"),
                    "-o", str(app / "Contents/MacOS/ab-worm-host")], env=environment, check=True)
    info = plistlib.loads((root / "src/macos/extension/Info.plist").read_bytes())
    for bundle, identifier, executable, package in (
        (extension, "com.autobricks.worm.fskit", "ab-worm-extension", "XPC!"),
        (app, "com.autobricks.worm", "ab-worm-host", "APPL"),
    ):
        values = dict(info) if bundle == extension else {}
        values.update(CFBundleIdentifier=identifier, CFBundleExecutable=executable,
                      CFBundlePackageType=package, CFBundleName="Autobricks WORM",
                      CFBundleDisplayName="Autobricks WORM", CFBundleVersion=version,
                      CFBundleShortVersionString=version, LSMinimumSystemVersion="26.0")
        (bundle / "Contents/Info.plist").write_bytes(plistlib.dumps(values))
    identity = environment.get("AB_WORM_SIGN_IDENTITY", "-")
    subprocess.run(["codesign", "--force", "--sign", identity,
                    "--entitlements", str(root / "src/macos/extension/WormExtension.entitlements"),
                    str(extension)], check=True)
    subprocess.run(["codesign", "--force", "--sign", identity, str(app)], check=True)
    subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)], check=True)
    return app
