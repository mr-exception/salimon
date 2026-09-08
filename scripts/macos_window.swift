// PID-scoped window/display evidence; never capture the surrounding desktop.
import AppKit
import CoreGraphics
import Foundation

func fail(_ message: String) -> Never {
    FileHandle.standardError.write(Data((message + "\n").utf8))
    exit(1)
}

guard CommandLine.arguments.count == 2 || CommandLine.arguments.count == 3,
      let pid = Int(CommandLine.arguments[1]), pid > 0 else {
    fail("usage: swift macos_window.swift PID [capture.png]")
}
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID)
    as? [[String: Any]] ?? []
guard let window = windows.first(where: {
    ($0[kCGWindowOwnerPID as String] as? Int) == pid && ($0[kCGWindowLayer as String] as? Int) == 0
}), let number = window[kCGWindowNumber as String] as? Int,
    let bounds = window[kCGWindowBounds as String] as? [String: Any] else {
    fail("no on-screen layer-zero window for launched PID \(pid)")
}
var report: [String: Any] = [
    "pid": pid, "window_id": number, "os_window_bounds": bounds,
    "os_bounds_unit": "logical points; includes decorations",
    "drawable_source": "native window/resize logs, not OS logical bounds",
    "displays": NSScreen.screens.map { screen -> [String: Any] in
        let id = (screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber)?.uint32Value ?? 0
        let mode = CGDisplayCopyDisplayMode(id)
        return ["name": screen.localizedName, "scale_factor": screen.backingScaleFactor,
                "mode_pixels": [mode?.pixelWidth ?? 0, mode?.pixelHeight ?? 0],
                "mode_points": [mode?.width ?? 0, mode?.height ?? 0],
                "refresh_hz": mode?.refreshRate ?? 0]
    }
]
if CommandLine.arguments.count == 3 {
    let task = Process()
    task.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    task.arguments = ["-x", "-o", "-l", String(number), CommandLine.arguments[2]]
    do { try task.run() } catch { fail("cannot start window capture: \(error)") }
    task.waitUntilExit()
    guard task.terminationStatus == 0 else { fail("PID-scoped capture failed; check Screen Recording permission") }
    report["capture"] = CommandLine.arguments[2]
    report["capture_source"] = "macOS window compositor; not optical scanout"
}
do {
    let data = try JSONSerialization.data(withJSONObject: report, options: [.prettyPrinted, .sortedKeys])
    print(String(data: data, encoding: .utf8)!)
} catch { fail("cannot encode window context: \(error)") }
