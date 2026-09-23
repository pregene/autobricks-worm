import Foundation
import FSKit

extension StorageVolume {
    func attributes(_ item: StorageItem) throws -> FSItem.Attributes {
        let entry = try client.entry(item.path)
        let result = FSItem.Attributes()
        result.type = entry.directory ? .directory : .file
        result.mode = entry.directory ? 0o700 : (entry.metadata ? 0o400 : 0o600)
        result.uid = getuid(); result.gid = getgid()
        result.linkCount = entry.directory ? 2 : 1
        result.size = entry.directory ? 0 : entry.size
        result.allocSize = entry.directory ? 0 : ((entry.size + 4095) / 4096) * 4096
        result.fileID = item.identifier
        let parent = (item.path as NSString).deletingLastPathComponent
        result.parentID = item.path.isEmpty ? .parentOfRoot : self.item(parent).identifier
        result.birthTime = timespec(tv_sec: Int(entry.created_at), tv_nsec: 0)
        result.modifyTime = timespec(tv_sec: Int(entry.modified_at), tv_nsec: 0)
        result.changeTime = result.modifyTime
        result.accessTime = result.modifyTime
        result.inhibitKernelOffloadedIO = true
        return result
    }
    func getAttributes(_ request: FSItem.GetAttributesRequest, of item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        locked {
            do { replyHandler(try attributes(node(item)), nil) }
            catch { replyHandler(nil, error) }
        }
    }
    func setAttributes(_ request: FSItem.SetAttributesRequest, on item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        locked {
            do {
                let item = try node(item)
                let current = try attributes(item)
                if item.path.hasSuffix(".meta") { throw Self.error(EPERM) }
                // A no-op size request is safe, including O_TRUNC on a new empty file.
                let protected: [FSItem.Attribute] = [.mode, .uid, .gid, .flags, .birthTime, .modifyTime, .accessTime, .changeTime, .backupTime, .addedTime]
                if protected.contains(where: { request.isValid($0) }) { throw Self.error(EPERM) }
                if request.isValid(.size) {
                    guard request.size == current.size else { throw Self.error(EPERM) }
                    request.consumedAttributes.insert(.size)
                }
                replyHandler(current, nil)
            } catch { replyHandler(nil, error) }
        }
    }
}
