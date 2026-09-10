import AppKit
import Foundation

let width = 56, height = 68
let input = URL(fileURLWithPath: CommandLine.arguments[1])
let source = NSImage(contentsOf: input)!
let sourceRep = source.representations.first as! NSBitmapImageRep
var reports = [[String: Any]]()
func report(_ name: String, _ rep: NSBitmapImageRep) {
    var samples = [[String: Any]]()
    for y in [1, 40, 67] {
        let color = rep.colorAt(x: 1, y: y)!
        let srgb = color.usingColorSpace(.sRGB)!
        samples.append(["y": y, "srgb": [srgb.redComponent, srgb.greenComponent, srgb.blueComponent, srgb.alphaComponent]])
    }
    reports.append(["stage": name, "space": rep.colorSpace.localizedName ?? "unknown", "samples": samples])
}
report("decoded PNG", sourceRep)

// Reproduce iTermImage.dataForImage and its secure decode into a DeviceRGB bitmap.
var bytes = Data(count: width * height * 4)
bytes.withUnsafeMutableBytes { storage in
    let context = CGContext(data: storage.baseAddress!, width: width, height: height,
        bitsPerComponent: 8, bytesPerRow: width * 4, space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue)!
    context.draw(sourceRep.cgImage!, in: CGRect(x: 0, y: 0, width: width, height: height))
}
let device = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .deviceRGB, bytesPerRow: width * 4, bitsPerPixel: 32)!
bytes.copyBytes(to: device.bitmapData!, count: bytes.count)
report("worker DeviceRGB round trip", device)
let decoded = NSImage(size: NSSize(width: width, height: height))
decoded.addRepresentation(device)

// Reproduce NSImage.safelyResizedImageWithSize, including its calibrated buffer.
let resized = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: width, pixelsHigh: height,
    bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
    colorSpaceName: .calibratedRGB, bytesPerRow: 0, bitsPerPixel: 0)!
resized.size = NSSize(width: width, height: height)
NSGraphicsContext.saveGraphicsState()
NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: resized)
decoded.draw(in: NSRect(x: 0, y: 0, width: width, height: height), from: .zero, operation: .copy, fraction: 1)
NSGraphicsContext.restoreGraphicsState()
report("calibrated resize", resized)
let data = try JSONSerialization.data(withJSONObject: reports, options: [.prettyPrinted, .sortedKeys])
FileHandle.standardOutput.write(data)
