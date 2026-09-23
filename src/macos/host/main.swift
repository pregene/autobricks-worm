import AppKit

let application = NSApplication.shared
application.setActivationPolicy(.regular)
let alert = NSAlert()
alert.messageText = "Autobricks WORM Filesystem"
alert.informativeText = "Enable Autobricks WORM under System Settings > General > Login Items & Extensions > File System Extensions. Then use ab-worm mount SOURCE MOUNTPOINT --retain DAYS."
alert.addButton(withTitle: "OK")
application.activate(ignoringOtherApps: true)
alert.runModal()
