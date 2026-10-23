// #313: wait for an actual target window before focus activation during preparation.
// #312 adaptation: motion uses a HID mouseMoved event without click edges.
import ApplicationServices
import AppKit
import CoreGraphics
import Foundation
let args = CommandLine.arguments
func fail(_ message: String) -> Never { fputs(message + "\n", stderr); exit(1) }
guard args.count >= 3, let pid = Int32(args[1]) else { fail("usage: input PID focus | key CODE MILLISECONDS | mouse DX DY") }
guard CGPreflightPostEventAccess() else { fail("macOS event posting access is unavailable") }
guard let app = NSRunningApplication(processIdentifier: pid) else { fail("target PID is not running") }
let source = CGEventSource(stateID: .hidSystemState)
switch args[2] {
case "focus":
    guard AXIsProcessTrusted() else {fail("existing Accessibility permission unavailable; no permission requested")}
    app.unhide();app.activate(options:[.activateAllWindows]);Thread.sleep(forTimeInterval:0.3)
    let axApp=AXUIElementCreateApplication(pid)
    var raw:CFTypeRef?
    guard AXUIElementCopyAttributeValue(axApp,kAXWindowsAttribute as CFString,&raw) == .success,
          let windows=raw as? [AXUIElement],windows.count==1 else {fail("expected one target accessibility window")}
    _ = AXUIElementPerformAction(windows[0],kAXRaiseAction as CFString)
    var positionRaw:CFTypeRef?;var sizeRaw:CFTypeRef?
    guard AXUIElementCopyAttributeValue(windows[0],kAXPositionAttribute as CFString,&positionRaw) == .success,
          AXUIElementCopyAttributeValue(windows[0],kAXSizeAttribute as CFString,&sizeRaw) == .success,
          let positionRaw=positionRaw,let sizeRaw=sizeRaw else {fail("AX window geometry unavailable")}
    var origin=CGPoint.zero;var size=CGSize.zero
    guard AXValueGetValue(positionRaw as! AXValue,.cgPoint,&origin),AXValueGetValue(sizeRaw as! AXValue,.cgSize,&size) else {fail("AX window geometry invalid")}
    let point=CGPoint(x:origin.x+size.width/2,y:origin.y+size.height/2)
    for type in [CGEventType.leftMouseDown,.leftMouseUp] {
        guard let event=CGEvent(mouseEventSource:source,mouseType:type,mouseCursorPosition:point,mouseButton:.left) else {fail("cannot create focus click")}
        event.post(tap:.cghidEventTap);Thread.sleep(forTimeInterval:0.05)
    }
    let front=NSWorkspace.shared.frontmostApplication?.processIdentifier ?? -1
    print("frontmost=\(front) target=\(pid)")
    guard front==pid else {fail("target PID is not frontmost")}
case "key":
    app.unhide()
    app.activate(options: [.activateAllWindows])
    Thread.sleep(forTimeInterval: 0.3)
    guard args.count == 5, let code = UInt16(args[3]), let ms = Double(args[4]), ms >= 0, ms <= 5000 else { fail("invalid key/duration") }
    guard let down = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: true), let up = CGEvent(keyboardEventSource: source, virtualKey: code, keyDown: false) else { fail("cannot create key events") }
    defer { up.post(tap: .cghidEventTap) }
    down.post(tap: .cghidEventTap)
    Thread.sleep(forTimeInterval: ms / 1000)
case "move":
    guard args.count == 5, let targetX=Double(args[3]), let targetY=Double(args[4]), abs(targetX)<10000, abs(targetY)<10000 else { fail("invalid window position") }
    guard AXIsProcessTrusted() else {fail("existing Accessibility permission unavailable; no permission requested")}
    app.unhide();app.activate(options:[.activateAllWindows]);Thread.sleep(forTimeInterval:0.3)
    let axApp=AXUIElementCreateApplication(pid)
    var raw:CFTypeRef?
    guard AXUIElementCopyAttributeValue(axApp,kAXWindowsAttribute as CFString,&raw) == .success,
          let windows=raw as? [AXUIElement],windows.count==1 else {fail("expected one target accessibility window")}
    var point=CGPoint(x:targetX,y:targetY)
    guard let value=AXValueCreate(.cgPoint,&point) else {fail("cannot create target position")}
    let result=AXUIElementSetAttributeValue(windows[0],kAXPositionAttribute as CFString,value)
    guard result == .success else {fail("native window position failed: \(result.rawValue)")}
    Thread.sleep(forTimeInterval:0.3)
    print("AX window position requested \(targetX),\(targetY) PID=\(pid)")
case "mouse":
    app.unhide()
    app.activate(options: [.activateAllWindows])
    Thread.sleep(forTimeInterval: 0.3)
    guard args.count == 5, let dx = Int64(args[3]), let dy = Int64(args[4]), abs(dx) <= 1000, abs(dy) <= 1000 else { fail("invalid relative motion") }
    let position = CGEvent(source: nil)?.location ?? CGPoint(x: 0, y: 0)
    guard let event = CGEvent(mouseEventSource: source, mouseType: .mouseMoved, mouseCursorPosition: CGPoint(x: position.x + Double(dx), y: position.y + Double(dy)), mouseButton: .left) else { fail("cannot create mouse event") }
    event.setIntegerValueField(.mouseEventDeltaX, value: dx)
    event.setIntegerValueField(.mouseEventDeltaY, value: dy)
    event.post(tap: .cghidEventTap)
default: fail("unknown input operation")
}
print("posted PID=\(pid) operation=\(args.dropFirst(2).joined(separator: " "))")
