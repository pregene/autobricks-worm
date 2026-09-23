import Foundation
import FSKit

public extension NamespaceGuard {
    func mount(options: FSTaskOptions, replyHandler: @escaping (Error?) -> Void) {
        backend.mount(options: options, replyHandler: replyHandler)
    }
    func unmount(replyHandler: @escaping () -> Void) { backend.unmount(replyHandler: replyHandler) }
    func synchronize(flags: FSSyncFlags, replyHandler: @escaping (Error?) -> Void) {
        backend.synchronize(flags: flags, replyHandler: replyHandler)
    }
    func activate(options: FSTaskOptions, replyHandler: @escaping (FSItem?, Error?) -> Void) {
        backend.activate(options: options, replyHandler: replyHandler)
    }
    func deactivate(options: FSDeactivateOptions, replyHandler: @escaping (Error?) -> Void) {
        backend.deactivate(options: options, replyHandler: replyHandler)
    }
    func getAttributes(_ attributes: FSItem.GetAttributesRequest, of item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        backend.getAttributes(attributes, of: item, replyHandler: replyHandler)
    }
    func setAttributes(_ attributes: FSItem.SetAttributesRequest, on item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) {
        backend.setAttributes(attributes, on: item, replyHandler: replyHandler)
    }
    func lookupItem(named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        backend.lookupItem(named: name, inDirectory: directory, replyHandler: replyHandler)
    }
    func reclaimItem(_ item: FSItem, replyHandler: @escaping (Error?) -> Void) {
        backend.reclaimItem(item, replyHandler: replyHandler)
    }
    func readSymbolicLink(_ item: FSItem, replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        backend.readSymbolicLink(item, replyHandler: replyHandler)
    }
    func removeItem(_ item: FSItem, named name: FSFileName, fromDirectory directory: FSItem,
                    replyHandler: @escaping (Error?) -> Void) {
        backend.removeItem(item, named: name, fromDirectory: directory, replyHandler: replyHandler)
    }
    func enumerateDirectory(_ directory: FSItem, startingAt cookie: FSDirectoryCookie,
                            verifier: FSDirectoryVerifier, attributes: FSItem.GetAttributesRequest?,
                            packer: FSDirectoryEntryPacker,
                            replyHandler: @escaping (FSDirectoryVerifier, Error?) -> Void) {
        backend.enumerateDirectory(directory, startingAt: cookie, verifier: verifier,
                                   attributes: attributes, packer: packer, replyHandler: replyHandler)
    }
}
