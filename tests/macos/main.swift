import Foundation
import FSKit

var checks = 0
func expect(_ condition: Bool, _ message: String) {
    guard condition else { fatalError(message) }
    checks += 1
}
func checkError(_ error: Error?, _ expected: Int32) {
    let value = error as NSError?
    expect(value?.domain == NSPOSIXErrorDomain && value?.code == Int(expected), "Expected POSIX error \(expected), got \(String(describing: error))")
}
let fixture = Fixture()
let guardVolume = NamespaceGuard(fixture)
let attributes = FSItem.SetAttributesRequest()
for (text, code) in [("audit.log", Int32(0)), ("audit.log.meta", EPERM), ("audit.META", EPERM), (".meta", EPERM), ("", EINVAL), (".", EINVAL), ("..", EINVAL), ("a/b", EINVAL)] {
    let name = FSFileName(string: text)
    let count = fixture.calls.count
    var replies = 0
    for type: FSItem.ItemType in [.file, .directory] {
        guardVolume.createItem(named: name, type: type, inDirectory: fixture.root, attributes: attributes) { item, resultName, error in
            replies += 1
            if code == 0 { expect(item != nil && resultName?.data == name.data && error == nil, "Creation must reach backend") }
            else { checkError(error, code) }
        }
    }
    guardVolume.createSymbolicLink(named: name, inDirectory: fixture.root, attributes: attributes, linkContents: FSFileName(string: "audit.log")) { _, _, error in
        replies += 1
        if code == 0 { expect(error == nil, "Symbolic link must reach backend") } else { checkError(error, code) }
    }
    guardVolume.createLink(to: fixture.root, named: name, inDirectory: fixture.root) { _, error in
        replies += 1
        if code == 0 { expect(error == nil, "Hard link must reach backend") } else { checkError(error, code) }
    }
    expect(replies == 4, "Every callback must reply exactly once")
    expect(fixture.calls.count == count + (code == 0 ? 4 : 0), "Rejected names must not reach backend")
}
for replacement in [nil, fixture.root] {
    var replied = false
    guardVolume.renameItem(fixture.root, inDirectory: fixture.root, named: FSFileName(string: "audit.log"), to: FSFileName(string: "renamed"), inDirectory: fixture.root, overItem: replacement) { _, error in
        replied = true; checkError(error, EPERM)
    }
    expect(replied, "Rename must reply")
}
expect(!fixture.calls.contains("rename"), "Rename must never reach backend")
guardVolume.setVolumeName(FSFileName(string: "changed")) { _, error in checkError(error, EPERM) }
let raw = FSFileName(data: Data([0xff, 0x2e, 0x4d, 0x45, 0x54, 0x41]))
checkError(NamespaceGuard.nameError(raw), EPERM)
guardVolume.write(contents: Data([1, 2, 3]), to: fixture.root, at: 0) { count, error in
    expect(count == 0, "Partial write count must be preserved"); checkError(error, ENOSPC)
}
expect(fixture.calls.last == "write", "I/O must reach backend")
print("FSKit adapter: \(checks) checks passed")
GuardedFileSystem.guardLoadedVolume(fixture, error: nil) { volume, error in
    expect(volume is NamespaceGuard && error == nil, "Every loaded volume must be guarded")
}
let unsupportedVolume = FSVolume(volumeID: FSVolume.Identifier(uuid: UUID()), volumeName: FSFileName(string: "invalid"))
GuardedFileSystem.guardLoadedVolume(unsupportedVolume, error: nil) { volume, error in
    expect(volume == nil, "Unsupported backend must not escape guard"); checkError(error, ENOTSUP)
}
GuardedFileSystem.guardLoadedVolume(fixture, error: NSError(domain: NSPOSIXErrorDomain, code: Int(EIO))) { volume, error in
    expect(volume == nil, "Failed load must not return a volume"); checkError(error, EIO)
}
print("FSKit filesystem loading: all checks passed (\(checks) total)")

let storageDirectory = FileManager.default.temporaryDirectory.appendingPathComponent("ab-worm-fskit-\(UUID().uuidString)")
try FileManager.default.createDirectory(at: storageDirectory, withIntermediateDirectories: false)
do {
    let volume = try StorageVolume(source: storageDirectory, retentionSeconds: 365 * 86400)
    let guardVolume = NamespaceGuard(volume)
    let root = volume.item("")
    var file: FSItem?
    var callbackCount = 0
    guardVolume.createItem(named: FSFileName(string: "audit.log"), type: .file, inDirectory: root,
                          attributes: FSItem.SetAttributesRequest()) { item, _, error in
        expect(error == nil && item != nil, "Persistent file creation must succeed")
        file = item; callbackCount += 1
    }
    expect(callbackCount == 1, "Persistent create must reply once")
    for offset: Int64 in [0, 16] {
        guardVolume.write(contents: Data("0123456789abcdef".utf8), to: file!, at: offset) { count, error in
            expect(count == 16 && error == nil, "Persistent append must succeed")
        }
    }
    guardVolume.write(contents: Data([0]), to: file!, at: 0) { count, error in
        expect(count == 0, "Overwrite must not write bytes"); checkError(error, EPERM)
    }
    let truncate = FSItem.SetAttributesRequest()
    truncate.size = 0
    guardVolume.setAttributes(truncate, on: file!) { _, error in checkError(error, EPERM) }
    let meta = volume.item("audit.log.meta")
    guardVolume.openItem(meta, modes: [.write]) { error in checkError(error, EPERM) }
    guardVolume.write(contents: Data([0]), to: meta, at: 0) { _, error in checkError(error, EPERM) }
    guardVolume.removeItem(meta, named: FSFileName(string: "audit.log.meta"), fromDirectory: root) { error in checkError(error, EPERM) }
    guardVolume.removeItem(file!, named: FSFileName(string: "audit.log"), fromDirectory: root) { error in checkError(error, EPERM) }
    let record = try JSONSerialization.jsonObject(with: Data(contentsOf: storageDirectory.appendingPathComponent("audit.log.meta"))) as! [String: Any]
    expect(record["lock_offset"] as? Int == 32, "Persistent LOCK must be 32")
    expect(record["checksum"] as? String == "3eb1bd439947eb762998e566ccc2e099c791118b2f40579cc4f7da2b5061b7f9", "Persistent digest must match whole-file SHA-256")
    expect((record["retain_until"] as! UInt64) - (record["created_at"] as! UInt64) == 365 * 86400, "Retention must start at creation")
    expect(try volume.client.entries("").map(\.name) == ["audit.log", "audit.log.meta"], "Directory view must hide internal control files")
}
try FileManager.default.removeItem(at: storageDirectory)
expect(try RetentionOptions.seconds(["-o", "retain=365"]) == 365 * 86400, "Retention days must convert to seconds")
expect(try RetentionOptions.seconds(["-o", "retain=0"]) == 0, "Zero-day retention must be explicit")
for arguments in [[], ["retain=-1"], ["retain=1", "retain=2"], ["retain=18446744073709551615"]] {
    do { _ = try RetentionOptions.seconds(arguments); fatalError("Invalid retention must fail") }
    catch { checkError(error, EINVAL) }
}
print("FSKit persistent storage: all checks passed (\(checks) total)")
