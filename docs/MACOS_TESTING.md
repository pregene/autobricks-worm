# macOS development and testing

## Requirements

- macOS 26 or later for directory-backed FSKit mounting.
- Apple developer tools with Swift and the macOS 26 or newer SDK.
- Rust/Cargo and Python 3.
- Active Apple Developer Program membership, or access to an enrolled team, for signing the FSKit extension used in mount tests.

An ordinary Apple Account and acceptance of the Apple Developer agreement are separate from program membership. Apple's [macOS capability table](https://developer.apple.com/help/account/reference/supported-capabilities-macos) lists FSKit Module support for Apple Developer Program and Developer ID signing, but not the free Apple Developer account tier.

Compilation and the automated tests below can run with Command Line Tools and ad-hoc signing. OS-mounted extension testing requires the developer signing setup as well as user activation.

## Developer account and signing

1. Check membership at [Apple Developer Account](https://developer.apple.com/account/). Enroll through [Apple Developer Program](https://developer.apple.com/programs/enroll/) if needed, or join an enrolled team.
2. Install Xcode and add the account under Settings > Apple Accounts. Select the enrolled team and create an Apple Development certificate under Manage Certificates.
3. Configure the extension's App ID and provisioning profile for the `com.apple.developer.fskit.fsmodule` entitlement. Match the profile, bundle identifier, and signing team, and embed the applicable profile before signing the extension.
4. Sign the containing app and extension with the team's identity, then register and enable the extension for mounting.

The build script accepts `AB_WORM_SIGN_IDENTITY` as the codesign identity. Its default is ad-hoc signing (`-`). The variable selects the signing certificate; account enrollment and provisioning profile setup are separate steps.

References: [signing certificates](https://developer.apple.com/documentation/xcode/sharing-your-teams-signing-certificates), [development provisioning profiles](https://developer.apple.com/help/account/provisioning-profiles/create-a-development-provisioning-profile), and [Apple's FSKit extension guide](https://developer.apple.com/documentation/fskit/building-a-passthrough-file-system).

## Automated tests

```sh
./scripts/test-macos.sh
```

This runs Rust tests, Clippy, and native FSKit callback tests linked to the Rust library. Coverage includes persistent append writes, SHA-256 metadata, creation-based retention, reserved `.meta` names, metadata mutation protection, and rename rejection. These tests invoke callbacks directly; successful results do not establish OS mount readiness.

## Build and mount

```sh
./build.sh --release --locked
./target/release/ab-worm --version
```

The native build produces `ab-worm`, the Rust and Swift adapter libraries, and `Autobricks WORM.app` in `target/release/`. The app contains the FSKit extension. Keep the app beside the executable.

With developer signing configured, use separate backing and mount directories:

```sh
mkdir -p worm-storage worm-mount
./target/release/ab-worm mount ./worm-storage ./worm-mount --retain 365
```

The mount command registers the bundled extension. Enable Autobricks WORM under System Settings > General > Login Items & Extensions > File System Extensions, then retry the command. `--retain` specifies days from creation for new files; existing retention deadlines remain unchanged.

```sh
./target/release/ab-worm unmount ./worm-mount
```

## Validation status

Rust tests, native FSKit callback tests, release compilation, and build-version tests passed during local development. OS-mounted validation is pending: the local ad-hoc-signed extension is listed but FSKit reports it as disabled. If the activation toggle does not stay enabled, check membership, signing identity, entitlements, and provisioning before retrying. The disabled status alone does not identify the cause.
