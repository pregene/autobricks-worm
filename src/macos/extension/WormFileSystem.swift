import Foundation
import FSKit

@available(macOS 26.0, *)
final class WormFileSystem: FSUnaryFileSystem, FSUnaryFileSystemOperations {
    private var source: URL?
    private let lock = NSLock()

    func probeResource(resource: FSResource, replyHandler: @escaping (FSProbeResult?, Error?) -> Void) {
        guard resource is FSPathURLResource else {
            replyHandler(.notRecognized, nil); return
        }
        replyHandler(.usable(name: "Autobricks WORM", containerID: FSContainerIdentifier(uuid: UUID())), nil)
    }

    func loadResource(resource: FSResource, options: FSTaskOptions,
                      replyHandler: @escaping (FSVolume?, Error?) -> Void) {
        lock.lock(); defer { lock.unlock() }
        guard source == nil else { replyHandler(nil, StorageVolume.error(EBUSY)); return }
        guard let path = resource as? FSPathURLResource, path.isWritable else {
            replyHandler(nil, StorageVolume.error(EINVAL)); return
        }
        guard path.url.startAccessingSecurityScopedResource() else {
            replyHandler(nil, StorageVolume.error(EACCES)); return
        }
        do {
            let volume = try StorageVolume(source: path.url)
            source = path.url
            containerStatus = .ready
            replyHandler(volume, nil)
        } catch {
            path.url.stopAccessingSecurityScopedResource()
            replyHandler(nil, error)
        }
    }

    func unloadResource(resource: FSResource, options: FSTaskOptions,
                        replyHandler: @escaping (Error?) -> Void) {
        lock.lock(); defer { lock.unlock() }
        source?.stopAccessingSecurityScopedResource()
        source = nil
        replyHandler(nil)
    }
}
