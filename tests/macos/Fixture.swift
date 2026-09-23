import Foundation
import FSKit

/// In-memory callback spy used only by the native adapter tests.
final class Fixture: FSVolume, FSVolume.Operations, FSVolume.ReadWriteOperations, FSVolume.OpenCloseOperations {
    let root = FSItem()
    var calls: [String] = []
    let maximumLinkCount = 1024
    let maximumNameLength = 255
    let restrictsOwnershipChanges = true
    let truncatesLongNames = false
    let supportedVolumeCapabilities = FSVolume.SupportedCapabilities()
    let volumeStatistics = FSStatFSResult(fileSystemTypeName: "ab-worm-test")
    init() {
        super.init(volumeID: FSVolume.Identifier(uuid: UUID()), volumeName: FSFileName(string: "ab-worm-test"))
    }
    func createItem(named name: FSFileName, type: FSItem.ItemType, inDirectory directory: FSItem,
                    attributes: FSItem.SetAttributesRequest, replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        calls.append("create"); replyHandler(FSItem(), name, nil)
    }
    func createSymbolicLink(named name: FSFileName, inDirectory directory: FSItem,
                           attributes: FSItem.SetAttributesRequest, linkContents: FSFileName,
                           replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) {
        calls.append("symlink"); replyHandler(FSItem(), name, nil)
    }
    func createLink(to item: FSItem, named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        calls.append("link"); replyHandler(name, nil)
    }
    func renameItem(_ item: FSItem, inDirectory directory: FSItem, named name: FSFileName,
                    to newName: FSFileName, inDirectory newDirectory: FSItem, overItem: FSItem?,
                    replyHandler: @escaping (FSFileName?, Error?) -> Void) {
        calls.append("rename"); replyHandler(newName, nil)
    }
    func mount(options: FSTaskOptions, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func unmount(replyHandler: @escaping () -> Void) { replyHandler() }
    func synchronize(flags: FSSyncFlags, replyHandler: @escaping (Error?) -> Void) {
        calls.append("sync"); replyHandler(nil)
    }
    func activate(options: FSTaskOptions, replyHandler: @escaping (FSItem?, Error?) -> Void) { replyHandler(root, nil) }
    func deactivate(options: FSDeactivateOptions, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func getAttributes(_ attributes: FSItem.GetAttributesRequest, of item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) { replyHandler(FSItem.Attributes(), nil) }
    func setAttributes(_ attributes: FSItem.SetAttributesRequest, on item: FSItem,
                       replyHandler: @escaping (FSItem.Attributes?, Error?) -> Void) { replyHandler(FSItem.Attributes(), nil) }
    func lookupItem(named name: FSFileName, inDirectory directory: FSItem,
                    replyHandler: @escaping (FSItem?, FSFileName?, Error?) -> Void) { replyHandler(root, name, nil) }
    func reclaimItem(_ item: FSItem, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func readSymbolicLink(_ item: FSItem, replyHandler: @escaping (FSFileName?, Error?) -> Void) { replyHandler(nil, nil) }
    func removeItem(_ item: FSItem, named name: FSFileName, fromDirectory directory: FSItem,
                    replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func enumerateDirectory(_ directory: FSItem, startingAt cookie: FSDirectoryCookie,
                            verifier: FSDirectoryVerifier, attributes: FSItem.GetAttributesRequest?,
                            packer: FSDirectoryEntryPacker,
                            replyHandler: @escaping (FSDirectoryVerifier, Error?) -> Void) { replyHandler(verifier, nil) }
    func openItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        calls.append("open"); replyHandler(nil)
    }
    func closeItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        calls.append("close"); replyHandler(nil)
    }
    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer,
              replyHandler: @escaping (Int, Error?) -> Void) { replyHandler(0, nil) }
    func write(contents: Data, to item: FSItem, at offset: off_t,
               replyHandler: @escaping (Int, Error?) -> Void) {
        calls.append("write"); replyHandler(0, NSError(domain: NSPOSIXErrorDomain, code: Int(ENOSPC)))
    }
}
