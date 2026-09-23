import Foundation
import FSKit

/// Wraps every loaded volume before FSKit can dispatch namespace operations.
public final class GuardedFileSystem: FSUnaryFileSystem, FSUnaryFileSystemOperations {
    public typealias Backend = FSUnaryFileSystem & FSUnaryFileSystemOperations
    let backend: Backend

    public init(_ backend: Backend) {
        self.backend = backend
        super.init()
    }

    public func probeResource(resource: FSResource, replyHandler: @escaping (FSProbeResult?, Error?) -> Void) {
        backend.probeResource(resource: resource, replyHandler: replyHandler)
    }

    public func loadResource(resource: FSResource, options: FSTaskOptions,
                      replyHandler: @escaping (FSVolume?, Error?) -> Void) {
        backend.loadResource(resource: resource, options: options) { volume, error in
            Self.guardLoadedVolume(volume, error: error, replyHandler: replyHandler)
        }
    }

    static func guardLoadedVolume(_ volume: FSVolume?, error: Error?,
                                  replyHandler: (FSVolume?, Error?) -> Void) {
        if let error { replyHandler(nil, error); return }
        guard let volume = volume as? NamespaceGuard.Backend else {
            replyHandler(nil, NSError(domain: NSPOSIXErrorDomain, code: Int(ENOTSUP)))
            return
        }
        replyHandler(NamespaceGuard(volume), nil)
    }

    public func unloadResource(resource: FSResource, options: FSTaskOptions,
                        replyHandler: @escaping (Error?) -> Void) {
        backend.unloadResource(resource: resource, options: options, replyHandler: replyHandler)
    }

    public func didFinishLoading() { backend.didFinishLoading?() }
}
