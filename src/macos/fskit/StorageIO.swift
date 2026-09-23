import Foundation
import FSKit

extension StorageVolume {
    func openItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        locked {
            do {
                let item = try node(item)
                let entry = try client.entry(item.path)
                if entry.metadata && modes.contains(.write) { throw Self.error(EPERM) }
                item.openModes.formUnion(modes)
                replyHandler(nil)
            } catch { replyHandler(error) }
        }
    }
    func closeItem(_ item: FSItem, modes: FSVolume.OpenModes, replyHandler: @escaping (Error?) -> Void) {
        locked {
            do { try node(item).openModes = modes; replyHandler(nil) }
            catch { replyHandler(error) }
        }
    }
    func read(from item: FSItem, at offset: off_t, length: Int, into buffer: FSMutableFileDataBuffer,
              replyHandler: @escaping (Int, Error?) -> Void) {
        locked {
            do {
                guard offset >= 0 && length >= 0 else { throw Self.error(EINVAL) }
                let object = try client.request("read", path: node(item).path,
                    fields: ["offset": UInt64(offset), "length": length])
                guard let values = object as? [UInt8] else { throw Self.error(EIO) }
                let data = Data(values)
                let count = buffer.withUnsafeMutableBytes { data.copyBytes(to: $0) }
                replyHandler(count, nil)
            } catch { replyHandler(0, error) }
        }
    }
    func write(contents: Data, to item: FSItem, at offset: off_t, replyHandler: @escaping (Int, Error?) -> Void) {
        locked {
            do {
                guard offset >= 0 else { throw Self.error(EINVAL) }
                let item = try node(item)
                let entry = try client.entry(item.path)
                guard !entry.metadata && !entry.directory else { throw Self.error(EPERM) }
                guard UInt64(offset) == entry.size else { throw Self.error(EPERM) }
                // Bound the JSON FFI request and allow the kernel to retry a partial write.
                let bytes = Array(contents.prefix(1024 * 1024))
                let result = try client.request("write", path: item.path,
                    fields: ["offset": UInt64(offset), "data": bytes])
                guard let count = result as? Int else { throw Self.error(EIO) }
                replyHandler(count, nil)
            } catch { replyHandler(0, error) }
        }
    }
    func synchronize(flags: FSSyncFlags, replyHandler: @escaping (Error?) -> Void) {
        locked {
            do { _ = try client.request("sync"); replyHandler(nil) }
            catch { replyHandler(error) }
        }
    }
    func activate(options: FSTaskOptions, replyHandler: @escaping (FSItem?, Error?) -> Void) {
        locked {
            do {
                retentionSeconds = try RetentionOptions.seconds(options.taskOptions)
                replyHandler(item(""), nil)
            } catch { replyHandler(nil, error) }
        }
    }
    func deactivate(options: FSDeactivateOptions, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func mount(options: FSTaskOptions, replyHandler: @escaping (Error?) -> Void) { replyHandler(nil) }
    func unmount(replyHandler: @escaping () -> Void) { replyHandler() }
}
