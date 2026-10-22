// #312 adaptation: motion uses a HID mouseMoved event without click edges.
import AppKit
import CoreGraphics
import Foundation
let args = CommandLine.arguments
func fail(_ message: String) -> Never { fputs(message + "\n", stderr); exit(1) }
guard args.count >= 3, let pid = Int32(args[1]) else { fail("usage: input PID focus | key CODE MILLISECONDS | mouse DX DY") }
guard CGPreflightPostEventAccess() else { fail("macOS event posting access is unavailable") }
guard let app = NSRunningApplication(processIdentifier: pid) else { fail("target PID is not running") }
app.unhide()
app.activate(options: [.activateAllWindows])
Thread.sleep(forTimeInterval: 0.3)
if args[2] != "focus" && args[2] != "window" {
    guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else { fail("target PID is not foreground; refusing global input") }
}
let source = CGEventSource(stateID: .hidSystemState)
switch args[2] {
case "focus":
    guard let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]],
          let window = windows.filter({ ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }).max(by: { a,b in
        func area(_ w: [String:Any]) -> Double { let r = w[kCGWindowBounds as String] as? [String:Double] ?? [:]; return (r["Width"] ?? 0) * (r["Height"] ?? 0) }; return area(a) < area(b)
    }),
          let bounds = window[kCGWindowBounds as String] as? [String: Double],
          let x = bounds["X"], let y = bounds["Y"], let width = bounds["Width"], let height = bounds["Height"] else { fail("target window bounds unavailable") }
    let point = CGPoint(x: x + width / 2, y: y + height / 2)
    for type in [CGEventType.leftMouseDown, .leftMouseUp] {
        guard let click = CGEvent(mouseEventSource: source, mouseType: type, mouseCursorPosition: point, mouseButton: .left) else { fail("cannot create focus click") }
        click.post(tap: .cghidEventTap)
        Thread.sleep(forTimeInterval: 0.05)
    }
    guard NSWorkspace.shared.frontmostApplication?.processIdentifier == pid else { fail("focus did not foreground target PID") }
    print("frontmost=\(NSWorkspace.shared.frontmostApplication?.processIdentifier ?? -1) target=\(pid)")
case "window":
    guard let windows = CGWindowListCopyWindowInfo(.optionOnScreenOnly, kCGNullWindowID) as? [[String: Any]], let window = windows.filter({ ($0[kCGWindowOwnerPID as String] as? Int32) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }).max(by: { a,b in
        func area(_ w: [String:Any]) -> Double { let r = w[kCGWindowBounds as String] as? [String:Double] ?? [:]; return (r["Width"] ?? 0) * (r["Height"] ?? 0) }; return area(a) < area(b)
    }) else { fail("target window missing") }
    let data = try JSONSerialization.data(withJSONObject: window, options: [.sortedKeys])
    print(String(data: data, encoding: .utf8)!)
case "key":
    guard args.count == 5, let code = UInt16(args[3]), let ms = Double(args[4]), ms >= 0, ms <= 5000 else { fail("invalid key/duration") }
    guard let down = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: true), let up = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: false) else { fail("cannot create key events") }
    defer { up.post(tap: .cghidEventTap) }
    down.post(tap: .cghidEventTap)
    Thread.sleep(forTimeInterval: ms / 1000)
case "mouse":
    guard args.count == 5, let dx = Int64(args[3]), let dy = Int64(args[4]), abs(dx) <= 1000, abs(dy) <= 1000 else { fail("invalid relative motion") }
    let position = CGEvent(source: nil)?.location ?? CGPoint(x: 0, y: 0)
    guard let event = CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: CGPoint(x: position.x + Double(dx), y: position.y + Double(dy)), mouseButton: .left) else { fail("cannot create mouse event") }
    event.setIntegerValueField(.mouseEventDeltaX, value: dx)
    event.setIntegerValueField(.mouseEventDeltaY, value: dy)
    event.post(tap: .cghidEventTap)
default: fail("unknown input operation")
}
print("posted PID=\(pid) operation=\(args.dropFirst(2).joined(separator: " "))")
