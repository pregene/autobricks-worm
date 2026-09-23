import Foundation
import FSKit

extension StorageVolume {
    func lookupItem(named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        locked {
            do {
                let path = try child(name, directory)
                _ = try client.entry(path)
                replyHandler(item(path), name, nil)
            } catch { replyHandler(nil, nil, error) }
        }
    }
    func createItem(named name: FSFileName, type: FSItem.ItemType, inDirectory directory: FSItem,
                    attributes: FSItem.SetAttributesRequest,
                    replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        locked {
            do {
                if let error = NamespaceGuard.nameError(name) { throw error }
                guard type == .file || type == .directory else { throw Self.error(ENOTSUP) }
                if attributes.isValid(.size) && attributes.size != 0 { throw Self.error(EINVAL) }
                guard let retentionSeconds else { throw Self.error(EINVAL) }
                let path = try child(name, directory)
                _ = try client.request(type == .directory ? "mkdir" : "create", path: path,
                                       fields: ["retention": retentionSeconds])
                if attributes.isValid(.mode) { attributes.consumedAttributes.insert(.mode) }
                generation += 1
                replyHandler(item(path), name, nil)
            } catch { replyHandler(nil, nil, error) }
        }
    }
    func createSymbolicLink(named name: FSFileName, inDirectory directory: FSItem,
                           attributes: FSItem.SetAttributesRequest, linkContents: FSFileName,
                           replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        replyHandler(nil, nil, Self.error(EPERM))
    }
    func createLink(to item: FSItem, named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) { replyHandler(nil, Self.error(EPERM)) }
    func readSymbolicLink(_ item: FSItem, replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        replyHandler(nil, Self.error(EINVAL))
    }
    func renameItem(_ item: FSItem, inDirectory directory: FSItem, named name: FSFileName,
                    to newName: FSFileName, inDirectory newDirectory: FSItem, overItem: FSItem?,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) { replyHandler(nil, Self.error(EPERM)) }
    func removeItem(_ item: FSItem, named name: FSFileName, fromDirectory directory: FSItem,
                    replyHandler: @escaping (Error?) -> Void) {
        locked {
            do {
                let item = try node(item)
                guard try child(name, directory) == item.path else { throw Self.error(EINVAL) }
                guard item.openModes.isEmpty, nodes[item.path + ".meta"]?.openModes.isEmpty != false else {
                    throw Self.error(EBUSY)
                }
                _ = try client.request("delete", path: item.path)
                item.deleted = true
                nodes.removeValue(forKey: item.path)
                nodes.removeValue(forKey: item.path + ".meta")?.deleted = true
                generation += 1
                replyHandler(nil)
            } catch { replyHandler(error) }
        }
    }
    func enumerateDirectory(_ directory: FSItem, startingAt cookie: FSDirectoryCookie,
                            verifier: FSDirectoryVerifier, attributes: FSItem.GetAttributesRequest?,
                            packer: FSDirectoryEntryPacker,
                            replyHandler: @escaping (FSDirectoryVerifier, Error?) -> Void) {
        locked {
            let current = FSDirectoryVerifier(rawValue: generation)
            do {
                if verifier.rawValue != 0 && verifier != current { throw Self.error(ESTALE) }
                let entries = try client.entries(node(directory).path)
                guard cookie.rawValue <= entries.count else { throw Self.error(EINVAL) }
                for index in Int(cookie.rawValue)..<entries.count {
                    let entry = entries[index]
                    let item = self.item(entry.name)
                    let accepted = packer.packEntry(name: FSFileName(string: (entry.name as NSString).lastPathComponent),
                        itemType: entry.directory ? .directory : .file, itemID: item.identifier,
                        nextCookie: FSDirectoryCookie(rawValue: UInt64(index + 1)),
                        attributes: attributes == nil ? nil : try self.attributes(item))
                    if !accepted { break }
                }
                replyHandler(current, nil)
            } catch { replyHandler(current, error) }
        }
    }
    func reclaimItem(_ item: FSItem, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
}
