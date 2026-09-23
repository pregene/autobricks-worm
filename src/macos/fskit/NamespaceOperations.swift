import Foundation
import FSKit

public extension NamespaceGuard {
    func createItem(named name: FSFileName, type: FSItem.ItemType, inDirectory directory: FSItem,
                    attributes: FSItem.SetAttributesRequest,
                    replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        if let error = Self.nameError(name) { replyHandler(nil, nil, error); return }
        backend.createItem(named: name, type: type, inDirectory: directory, attributes: attributes, replyHandler: replyHandler)
    }

    func createSymbolicLink(named name: FSFileName, inDirectory directory: FSItem,
                           attributes: FSItem.SetAttributesRequest, linkContents: FSFileName,
                           replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        if let error = Self.nameError(name) { replyHandler(nil, nil, error); return }
        backend.createSymbolicLink(named: name, inDirectory: directory, attributes: attributes,
                                   linkContents: linkContents, replyHandler: replyHandler)
    }

    func createLink(to item: FSItem, named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        if let error = Self.nameError(name) { replyHandler(nil, error); return }
        backend.createLink(to: item, named: name, inDirectory: directory, replyHandler: replyHandler)
    }

    func renameItem(_ item: FSItem, inDirectory directory: FSItem, named name: FSFileName,
                    to newName: FSFileName, inDirectory newDirectory: FSItem, overItem: FSItem?,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        replyHandler(nil, Self.denied)
    }

    func setVolumeName(_ name: FSFileName, replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        replyHandler(nil, Self.denied)
    }
}
