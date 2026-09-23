# Filesystem dependency licenses

Reviewed on 2026-09-23 against the official project sources below.

| Component | License | Integration |
| --- | --- | --- |
| Rust fuser 0.16.0 | MIT | Linux Rust FUSE interface |
| Linux libfuse | LGPL 2.1 / GPL 2 by component | FUSE libraries and tools |
| Apple FSKit, Foundation, ExtensionFoundation | Apple system frameworks; Xcode and Apple SDKs Agreement for SDK use | Native macOS filesystem adapter and extension protocols |
| Apple libSystem and Objective-C runtime | Apple system components; component-specific open-source notices | Native runtime services |
| Swift standard library and overlays | Apache-2.0 with Runtime Library Exception for upstream Swift components | Swift FSKit adapter runtime |
| Rust standard library | MIT OR Apache-2.0, with component-specific notices | Shared policy runtime |
| Rust libc 0.2.189 | MIT OR Apache-2.0 | macOS C ABI error constants |
| WinFsp | GPLv3 with a FLOSS exception; commercial licensing available | Windows filesystem runtime |
| winfsp-rs | GPLv3 | Rust bindings for WinFsp |
| Dokany | LGPLv3 / MIT by component | Windows filesystem runtime and FUSE wrapper |

## macOS dependency review

The macOS build uses Apple FSKit directly. The Rust dependency tree for macOS contains `libc`; `fuser` is a Linux dependency. The Swift adapter imports FSKit and Foundation; FSKit's Swift overlay also exposes ExtensionFoundation extension protocols.

Apple frameworks are system components provided by macOS. SDK use is governed by the [Xcode and Apple SDKs Agreement](https://www.apple.com/legal/sla/docs/xcode.pdf). They are used as operating-system libraries under the GPLv3 System Libraries provisions; see [GNU's explanation](https://www.gnu.org/licenses/gpl-faq.html#SystemLibraryException). The Autobricks source remains under the repository's GPLv3 license.

The upstream Swift compiler and standard library use [Apache-2.0 with Runtime Library Exception](https://www.swift.org/LICENSE.txt). Apple-distributed toolchains also carry their SDK agreement and bundled notices. Rust's [COPYRIGHT](https://github.com/rust-lang/rust/blob/main/COPYRIGHT) describes the MIT/Apache-2.0 licensing and separately licensed components in the toolchain and standard library. Preserve the installed toolchain's notices when packaging runtime components.

`libc` 0.2.189 declares MIT OR Apache-2.0: [MIT](https://github.com/rust-lang/libc/blob/0.2.189/LICENSE-MIT), [Apache-2.0](https://github.com/rust-lang/libc/blob/0.2.189/LICENSE-APACHE).

The adapter's dynamic dependencies include libSystem, the Objective-C runtime, FSKit, Foundation, and Swift runtime/overlay libraries. Foundation and other Apple frameworks may introduce further system dependencies. Apple publishes component-specific notices in its [open-source distributions](https://opensource.apple.com/releases/); these do not assign a single open-source license to the proprietary FSKit framework.

Audit a built artifact with:

```sh
cargo tree --locked --target aarch64-apple-darwin
cargo tree --locked --target x86_64-apple-darwin
otool -L target/release/libab_worm_fskit.dylib
```

The existing macFUSE license snapshot is retained below as historical review material. Its custom redistribution terms are specific to macFUSE, not Apple FSKit.

## WinFsp

WinFsp uses GPLv3 with a specific exception for qualifying free/libre and open-source software. The exception permits linking to platform-specific WinFsp DLLs and distributing unmodified official installers, subject to its stated license and attribution conditions. It also requires that the software using the exception is not linked or distributed with proprietary software.

The exception calls for the WinFsp copyright notice and repository link in the user interface and user-facing documentation. The upstream license offers commercial licensing separately.

## winfsp-rs

The Rust bindings declare GPLv3 in their package metadata and README. Their license is separate from the WinFsp runtime's license and FLOSS exception. The reviewed package metadata identifies version `0.13.1+winfsp-2.1`.

## Dokany

Dokany assigns LGPL to its userspace library, kernel driver, network library, FUSE wrapper, and installer. Its license text specifies LGPL version 3; source headers also contain version-3-or-later grants. The control utility and sample programs use MIT.

The LGPL combined-work terms include attribution, license copies, and requirements concerning replacement or relinking of the library. The exact terms follow the component being used.

Component mapping: [Dokany README](https://github.com/dokan-dev/dokany/blob/c7a59fc68ddcfeb4474f2fe7f24be4eb264af6a2/README.md#licensing).

## fuser and libfuse

fuser 0.16.0 uses MIT, with its original copyright and permission notice retained in [the local license copy](../licenses/fuser-0.16.0-MIT.txt).

The [libfuse LICENSE](https://github.com/libfuse/libfuse/blob/master/LICENSE) assigns LGPL 2.1 to `include/`, `lib/`, and `meson.build`, and GPL 2 to the remaining files.

## Source snapshots

The following links identify the reviewed source revisions and local copies of their complete license notices.

- [macfuse/macfuse — 4852a23cf56e](https://github.com/macfuse/macfuse/blob/4852a23cf56e2e9ace90ff669c68b5f860dc1d82/LICENSE.txt): [local license](../licenses/macfuse-LICENSE.txt)
- [winfsp/winfsp — ebd50e1956db](https://github.com/winfsp/winfsp/blob/ebd50e1956dbf7c12db5b897fa9af5dce8f61675/License.txt): [local license](../licenses/winfsp-LICENSE.txt)
- [SnowflakePowered/winfsp-rs — 5342e76c146e](https://github.com/SnowflakePowered/winfsp-rs/blob/5342e76c146ee7239553be15cf4d281e3ecbf6c9/LICENSE.md): [local license](../licenses/winfsp-rs-LICENSE.md)
- [dokan-dev/dokany — c7a59fc68ddc](https://github.com/dokan-dev/dokany/blob/c7a59fc68ddcfeb4474f2fe7f24be4eb264af6a2/license.lgpl.txt): [local license](../licenses/dokany-LGPL-3.0.txt)
- [dokan-dev/dokany — c7a59fc68ddc](https://github.com/dokan-dev/dokany/blob/c7a59fc68ddcfeb4474f2fe7f24be4eb264af6a2/license.mit.txt): [local license](../licenses/dokany-MIT.txt)
