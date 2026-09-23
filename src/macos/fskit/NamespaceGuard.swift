import Foundation
import FSKit

/// Applies the shared Rust namespace policy before calling a volume backend.
public final class NamespaceGuard: FSVolume, FSVolume.Operations, FSVolume.ReadWriteOperations,
    FSVolume.OpenCloseOperations, FSVolume.RenameOperations {
    public typealias Backend = FSVolume & FSVolume.Operations & FSVolume.ReadWriteOperations & FSVolume.OpenCloseOperations
    let backend: Backend

    public init(_ backend: Backend) {
        self.backend = backend
        super.init(volumeID: backend.volumeID, volumeName: backend.name)
    }

    static func nameError(_ name: FSFileName) -> NSError? {
        let result = name.data.withUnsafeBytes { buffer in
            ab_worm_check_entry_name(buffer.bindMemory(to: UInt8.self).baseAddress, buffer.count)
        }
        return result == 0 ? nil : NSError(domain: NSPOSIXErrorDomain, code: Int(result))
    }

    static var denied: NSError { NSError(domain: NSPOSIXErrorDomain, code: Int(EPERM)) }

    @available(macOS 26.0, *)
    public var enableOpenUnlinkEmulation: Bool {
        get { false }
        set { }
    }
    public var maximumLinkCount: Int { backend.maximumLinkCount }
    public var maximumNameLength: Int { backend.maximumNameLength }
    public var restrictsOwnershipChanges: Bool { backend.restrictsOwnershipChanges }
    public var truncatesLongNames: Bool { backend.truncatesLongNames }
    public var supportedVolumeCapabilities: FSVolume.SupportedCapabilities { backend.supportedVolumeCapabilities }
    public var volumeStatistics: FSStatFSResult { backend.volumeStatistics }
}
