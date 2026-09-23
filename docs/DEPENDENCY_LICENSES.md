# Filesystem dependency licenses

Reviewed on 2026-09-23 against the official project sources below.

| Component | License | Integration |
| --- | --- | --- |
| Rust fuser 0.16.0 | MIT | Rust FUSE interface |
| Linux libfuse | LGPL 2.1 / GPL 2 by component | FUSE libraries and tools |
| macFUSE 5.4.0 | Custom BSD-style terms with an additional commercial bundling condition | macOS filesystem runtime |
| WinFsp | GPLv3 with a FLOSS exception; commercial licensing available | Windows filesystem runtime |
| winfsp-rs | GPLv3 | Rust bindings for WinFsp |
| Dokany | LGPLv3 / MIT by component | Windows filesystem runtime and FUSE wrapper |

## macFUSE

The main license requires retention of copyright, conditions, and disclaimers, and restricts endorsement using contributor names. Clause 4 requires prior written permission for binary redistribution bundled with commercial software, including automated download or installation in that context. This additional condition distinguishes it from the standard BSD license.

The package also identifies licenses for inherited components, including BSD-style terms, APSL-covered portions, and the LGPL FUSE userspace library. Preserve those component notices when distributing the corresponding components.

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
