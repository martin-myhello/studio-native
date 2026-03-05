import SwiftRs
import Tauri
import UIKit
import WebKit
import PencilKit

class SetDrawingArgs: Decodable {
    let data: String
}

class SetToolArgs: Decodable {
    let tool: String
}

class PencilKitPlugin: Plugin {
    var webView: WKWebView!
    private var pencilKitView: PencilKitView?

    public override func load(webview: WKWebView) {
        self.webView = webview
    }

    @objc public func isAvailable(_ invoke: Invoke) throws {
        // PencilKit is available on iPad with Apple Pencil support (iOS 14+)
        let isIPad = UIDevice.current.userInterfaceIdiom == .pad
        invoke.resolve(["available": isIPad])
    }

    @objc public func show(_ invoke: Invoke) throws {
        DispatchQueue.main.async { [weak self] in
            guard let self = self, let webView = self.webView else {
                invoke.reject("WebView not available")
                return
            }

            if self.pencilKitView == nil {
                let view = PencilKitView()
                view.isUserInteractionEnabled = true
                view.backgroundColor = .clear
                view.autoresizingMask = [.flexibleWidth, .flexibleHeight]
                webView.superview?.insertSubview(view, aboveSubview: webView)
                self.pencilKitView = view
                print("[PencilKit] Canvas view added to view hierarchy")
            }

            self.pencilKitView?.isHidden = false
            if let parentView = webView.superview {
                self.pencilKitView?.frame = parentView.bounds
            }
            invoke.resolve()
        }
    }

    @objc public func hide(_ invoke: Invoke) throws {
        DispatchQueue.main.async { [weak self] in
            self?.pencilKitView?.isHidden = true
            invoke.resolve()
        }
    }

    @objc public func clear(_ invoke: Invoke) throws {
        DispatchQueue.main.async { [weak self] in
            self?.pencilKitView?.clearCanvas()
            invoke.resolve()
        }
    }

    @objc public func getDrawing(_ invoke: Invoke) throws {
        DispatchQueue.main.async { [weak self] in
            guard let data = self?.pencilKitView?.getDrawingData() else {
                invoke.resolve(["data": NSNull()])
                return
            }
            invoke.resolve(["data": data.base64EncodedString()])
        }
    }

    @objc public func setDrawing(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetDrawingArgs.self)

        guard let data = Data(base64Encoded: args.data) else {
            invoke.reject("Invalid base64 data")
            return
        }

        DispatchQueue.main.async { [weak self] in
            self?.pencilKitView?.loadDrawingData(data)
            invoke.resolve()
        }
    }

    @objc public func getImage(_ invoke: Invoke) throws {
        DispatchQueue.main.async { [weak self] in
            guard let pencilView = self?.pencilKitView else {
                invoke.resolve([
                    "image": NSNull(),
                    "x": 0,
                    "y": 0,
                    "width": 0,
                    "height": 0,
                ])
                return
            }

            let drawing = pencilView.canvasView.drawing
            if drawing.strokes.isEmpty {
                invoke.resolve([
                    "image": NSNull(),
                    "x": 0,
                    "y": 0,
                    "width": 0,
                    "height": 0,
                ])
                return
            }

            let bounds = drawing.bounds
            let image = drawing.image(from: bounds, scale: 2.0)
            guard let pngData = image.pngData() else {
                invoke.resolve([
                    "image": NSNull(),
                    "x": 0,
                    "y": 0,
                    "width": 0,
                    "height": 0,
                ])
                return
            }

            invoke.resolve([
                "image": pngData.base64EncodedString(),
                "x": bounds.origin.x,
                "y": bounds.origin.y,
                "width": bounds.width,
                "height": bounds.height,
            ])
        }
    }

    @objc public func setTool(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetToolArgs.self)

        DispatchQueue.main.async { [weak self] in
            self?.pencilKitView?.setTool(args.tool)
            invoke.resolve()
        }
    }
}

@_cdecl("init_plugin_pencilkit")
func initPlugin() -> Plugin {
    return PencilKitPlugin()
}
