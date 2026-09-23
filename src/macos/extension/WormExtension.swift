import FSKit

@main
struct WormExtension: UnaryFileSystemExtension {
    let fileSystem = GuardedFileSystem(WormFileSystem())
}
