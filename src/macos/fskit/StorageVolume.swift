import Foundation
import FSKit

final class StorageItem: FSItem {
    let path: String
    let identifier: FSItem.Identifier
    var deleted = false
    var openModes: FSVolume.OpenModes = []
    init(path: String, identifier: UInt64) {
        self.path = path
        self.identifier = FSItem.Identifier(rawValue: identifier)!
        super.init()
    }
}

/// Serializes filesystem callbacks while preserving the Rust storage transaction boundary.
final class StorageVolume: FSVolume, FSVolume.Operations, FSVolume.ReadWriteOperations,
    FSVolume.OpenCloseOperations {
    let client: StorageClient
    var retentionSeconds: UInt64?
    let sourceURL: URL
    let mutex = NSRecursiveLock()
    var nodes: [String: StorageItem] = [:]
    var nextID: UInt64 = 3
    var generation: UInt64 = 1
    var enableOpenUnlinkEmulation = false
    let maximumLinkCount = 1
    let maximumNameLength = 240
    let restrictsOwnershipChanges = true
    let truncatesLongNames = false

    init(source: URL, retentionSeconds: UInt64? = nil) throws {
        self.sourceURL = source
        self.retentionSeconds = retentionSeconds
        self.client = try StorageClient(path: source.path)
        super.init(volumeID: FSVolume.Identifier(uuid: UUID()), volumeName: FSFileName(string: "Autobricks WORM"))
        nodes[""] = StorageItem(path: "", identifier: 2)
    }

    func locked<T>(_ body: () throws -> T) rethrows -> T {
        mutex.lock(); defer { mutex.unlock() }
        return try body()
    }
    static func error(_ code: Int32) -> NSError { NSError(domain: NSPOSIXErrorDomain, code: Int(code)) }
    func node(_ item: FSItem) throws -> StorageItem {
        guard let item = item as? StorageItem, !item.deleted else { throw Self.error(ESTALE) }
        return item
    }
    func item(_ path: String) -> StorageItem {
        if let item = nodes[path] { return item }
        let item = StorageItem(path: path, identifier: nextID)
        nextID += 1
        nodes[path] = item
        return item
    }
    func child(_ name: FSFileName, _ directory: FSItem) throws -> String {
        let parent = try node(directory)
        guard try client.entry(parent.path).directory else { throw Self.error(ENOTDIR) }
        guard let name = name.string, !name.isEmpty, name != ".", name != "..", !name.contains("/"),
              !name.contains("\\"), name.utf8.count <= maximumNameLength else { throw Self.error(EINVAL) }
        return parent.path.isEmpty ? name : "\(parent.path)/\(name)"
    }

    var supportedVolumeCapabilities: FSVolume.SupportedCapabilities {
        let caps = FSVolume.SupportedCapabilities()
        caps.caseFormat = .sensitive
        caps.supports64BitObjectIDs = true
        caps.doesNotSupportSettingFilePermissions = true
        return caps
    }
    var volumeStatistics: FSStatFSResult {
        let result = FSStatFSResult(fileSystemTypeName: "abworm")
        result.blockSize = 4096
        result.ioSize = 65536
        if let info = try? FileManager.default.attributesOfFileSystem(forPath: sourceURL.path),
           let total = info[.systemSize] as? NSNumber, let free = info[.systemFreeSize] as? NSNumber {
            result.totalBytes = total.uint64Value
            result.freeBytes = free.uint64Value
            result.availableBytes = free.uint64Value
        }
        return result
    }
}
