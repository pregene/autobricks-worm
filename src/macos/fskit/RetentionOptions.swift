import Foundation

struct RetentionOptions {
    static func seconds(_ arguments: [String]) throws -> UInt64 {
        let values = arguments.flatMap { $0.split(separator: ",").map(String.init) }
            .filter { $0.hasPrefix("retain=") }
        guard values.count == 1, let days = UInt64(values[0].dropFirst(7)) else {
            throw NSError(domain: NSPOSIXErrorDomain, code: Int(EINVAL),
                userInfo: [NSLocalizedDescriptionKey: "Mount requires retain=DAYS"])
        }
        let (seconds, overflow) = days.multipliedReportingOverflow(by: 86400)
        guard !overflow else { throw NSError(domain: NSPOSIXErrorDomain, code: Int(EINVAL)) }
        return seconds
    }
}
