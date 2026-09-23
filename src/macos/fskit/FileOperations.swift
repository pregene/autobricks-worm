import Foundation
import FSKit

public extension NamespaceGuard {
    func openItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        backend.openItem(item, modes: modes, replyHandler: replyHandler)
    }
    func closeItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        backend.closeItem(item, modes: modes, replyHandler: replyHandler)
    }
    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer,
              replyHandler: @escaping (Int, Error?) -> Void) {
        backend.read(from: item, at: offset, length: length, into: buffer, replyHandler: replyHandler)
    }
    func write(contents: Data, to item: FSItem, at offset: off_t,
               replyHandler: @escaping (Int, Error?) -> Void) {
        backend.write(contents: contents, to: item, at: offset, replyHandler: replyHandler)
    }
}
