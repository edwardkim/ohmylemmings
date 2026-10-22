// Inspect ordinary window metadata; no accessibility or screen recording permission.
import AppKit
import CoreGraphics
import Foundation

let applications = NSWorkspace.shared.runningApplications.filter {
    $0.bundleIdentifier == "com.github.edwardkim.ohmylemmings"
}
let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]] ?? []
let rows: [[String: Any]] = applications.map { app in
    let visible = windows.filter { window in
        let owner = window[kCGWindowOwnerPID as String] as? Int32
        let layer = window[kCGWindowLayer as String] as? Int
        return owner == app.processIdentifier && layer == 0
    }
    return ["pid": app.processIdentifier, "name": app.localizedName ?? "", "windows": visible.count]
}
let data = try JSONSerialization.data(withJSONObject: rows, options: [.sortedKeys])
print(String(decoding: data, as: UTF8.self))
