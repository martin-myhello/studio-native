import UIKit
import PencilKit

class PencilKitView: UIView {

    let canvasView = PKCanvasView()
    private let toolPicker = PKToolPicker()

    override init(frame: CGRect) {
        super.init(frame: frame)
        setupCanvas()
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        setupCanvas()
    }

    private func setupCanvas() {
        canvasView.backgroundColor = .clear
        canvasView.isOpaque = false
        canvasView.drawingPolicy = .pencilOnly
        canvasView.translatesAutoresizingMaskIntoConstraints = false
        canvasView.isScrollEnabled = false
        canvasView.overrideUserInterfaceStyle = .light

        addSubview(canvasView)
        NSLayoutConstraint.activate([
            canvasView.topAnchor.constraint(equalTo: topAnchor),
            canvasView.bottomAnchor.constraint(equalTo: bottomAnchor),
            canvasView.leadingAnchor.constraint(equalTo: leadingAnchor),
            canvasView.trailingAnchor.constraint(equalTo: trailingAnchor),
        ])

        toolPicker.setVisible(true, forFirstResponder: canvasView)
        toolPicker.addObserver(canvasView)
        canvasView.becomeFirstResponder()
    }

    // Pencil touches -> draw on canvas. All other touches -> pass through to WebView.
    override func hitTest(_ point: CGPoint, with event: UIEvent?) -> UIView? {
        if let touches = event?.allTouches {
            for touch in touches {
                if touch.type == .pencil {
                    return canvasView.hitTest(convert(point, to: canvasView), with: event)
                }
            }
        }
        return nil
    }

    func clearCanvas() {
        canvasView.drawing = PKDrawing()
    }

    func getDrawingData() -> Data? {
        let drawing = canvasView.drawing
        if drawing.strokes.isEmpty { return nil }
        return drawing.dataRepresentation()
    }

    func loadDrawingData(_ data: Data) {
        do {
            let drawing = try PKDrawing(data: data)
            canvasView.drawing = drawing
        } catch {
            print("[PencilKit] Failed to load drawing: \(error)")
        }
    }

    func setContentOffset(_ x: CGFloat, y: CGFloat) {
        canvasView.transform = CGAffineTransform(translationX: -x, y: -y)
    }

    func setTool(_ toolName: String) {
        let tool: PKTool
        switch toolName {
        case "pen":
            tool = PKInkingTool(.pen, color: .black, width: 3)
        case "pencil":
            tool = PKInkingTool(.pencil, color: .black, width: 3)
        case "marker":
            tool = PKInkingTool(.marker, color: .yellow, width: 10)
        case "eraser":
            tool = PKEraserTool(.bitmap)
        default:
            tool = PKInkingTool(.pen, color: .black, width: 3)
        }
        canvasView.tool = tool
    }
}
