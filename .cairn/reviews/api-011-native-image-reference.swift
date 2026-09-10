import AppKit
import Foundation

final class ReferenceView: NSView {
    let image: NSImage
    init(image: NSImage) {
        self.image = image
        super.init(frame: NSRect(x: 0, y: 0, width: 240, height: 130))
    }
    required init?(coder: NSCoder) { fatalError("not used") }
    override func draw(_ dirtyRect: NSRect) {
        NSColor(srgbRed: 0, green: 0, blue: 0, alpha: 1).setFill()
        bounds.fill()
        image.draw(in: NSRect(x: 10, y: 10, width: 56, height: 68), from: .zero,
                   operation: .sourceOver, fraction: 1)
        NSColor(srgbRed: 1, green: 0, blue: 0, alpha: 1).setFill()
        NSRect(x: 90, y: 10, width: 28, height: 28).fill()
        NSColor(srgbRed: 0, green: 0, blue: 1, alpha: 1).setFill()
        NSRect(x: 130, y: 10, width: 28, height: 28).fill()
    }
}
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 100, y: 100, width: 240, height: 130),
    styleMask: [.titled], backing: .buffered, defer: false)
window.title = CommandLine.arguments[2]
window.contentView = ReferenceView(image: NSImage(contentsOfFile: CommandLine.arguments[1])!)
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
Timer.scheduledTimer(withTimeInterval: 20, repeats: false) { _ in app.terminate(nil) }
app.run()
