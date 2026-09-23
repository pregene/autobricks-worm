import Foundation

struct StorageEntry: Decodable {
    let name: String
    let directory: Bool
    let metadata: Bool
    let size: UInt64
    let created_at: UInt64
    let modified_at: UInt64
}

/// Owns the process-exclusive Rust store for the lifetime of one volume.
final class StorageClient {
    private let handle: UnsafeMutableRawPointer

    init(path: String) throws {
        var error: UnsafeMutablePointer<CChar>?
        guard let opened = path.withCString({ ab_worm_storage_open($0, &error) }) else {
            if let error { _ = try Self.decode(error) }
            throw NSError(domain: NSPOSIXErrorDomain, code: Int(EIO))
        }
        handle = opened
    }
    deinit { ab_worm_storage_close(handle) }

    private static func decode(_ response: UnsafeMutablePointer<CChar>) throws -> Any {
        defer { ab_worm_string_free(response) }
        let data = Data(String(cString: response).utf8)
        guard let object = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw NSError(domain: NSPOSIXErrorDomain, code: Int(EIO))
        }
        if let code = object["error"] as? Int {
            throw NSError(domain: NSPOSIXErrorDomain, code: code,
                          userInfo: [NSLocalizedDescriptionKey: object["message"] as? String ?? "Storage error"])
        }
        return object["result"] ?? NSNull()
    }

    func request(_ operation: String, path: String = "", fields: [String: Any] = [:]) throws -> Any {
        var object = fields
        object["operation"] = operation
        object["path"] = path
        let data = try JSONSerialization.data(withJSONObject: object)
        let text = String(decoding: data, as: UTF8.self)
        return try text.withCString { try Self.decode(ab_worm_storage_request(handle, $0)!) }
    }
    func entry(_ path: String) throws -> StorageEntry {
        let object = try request("entry", path: path)
        return try JSONDecoder().decode(StorageEntry.self, from: JSONSerialization.data(withJSONObject: object))
    }
    func entries(_ path: String) throws -> [StorageEntry] {
        let object = try request("entries", path: path)
        return try JSONDecoder().decode([StorageEntry].self, from: JSONSerialization.data(withJSONObject: object))
    }
}
