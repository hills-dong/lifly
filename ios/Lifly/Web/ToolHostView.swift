import SwiftUI
import WebKit

/// Hosts a single tool's web UI in a WKWebView, wired to the native bridge.
struct ToolHostScreen: View {
    let tool: Tool
    @Environment(\.dismiss) private var dismiss
    @State private var title: String

    init(tool: Tool) {
        self.tool = tool
        _title = State(initialValue: tool.name)
    }

    var body: some View {
        ToolWebView(tool: tool, title: $title, onClose: { dismiss() })
            .ignoresSafeArea(.container, edges: .bottom)
            .navigationTitle(title)
            .navigationBarTitleDisplayMode(.inline)
    }
}

private struct ToolWebView: UIViewRepresentable {
    let tool: Tool
    @Binding var title: String
    let onClose: () -> Void

    func makeCoordinator() -> Coordinator { Coordinator(self) }

    func makeUIView(context: Context) -> WKWebView {
        let controller = WKUserContentController()
        controller.addUserScript(
            WKUserScript(source: LiflyBridgeScript.source, injectionTime: .atDocumentStart, forMainFrameOnly: true)
        )
        controller.addScriptMessageHandler(
            context.coordinator.bridge,
            contentWorld: .page,
            name: LiflyBridge.handlerName
        )

        let config = WKWebViewConfiguration()
        config.userContentController = controller
        config.setURLSchemeHandler(context.coordinator.assetCache, forURLScheme: ToolAssetCache.scheme)

        let webView = WKWebView(frame: .zero, configuration: config)
        webView.allowsBackForwardNavigationGestures = false
        webView.scrollView.contentInsetAdjustmentBehavior = .always
        context.coordinator.webView = webView

        if let url = context.coordinator.toolURL {
            webView.load(URLRequest(url: url))
        }
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {}

    static func dismantleUIView(_ webView: WKWebView, coordinator: Coordinator) {
        let controller = webView.configuration.userContentController
        controller.removeScriptMessageHandler(forName: LiflyBridge.handlerName, contentWorld: .page)
        controller.removeAllUserScripts()
        webView.stopLoading()
    }

    final class Coordinator {
        let bridge: LiflyBridge
        let toolURL: URL?
        let media = MediaCapture()
        let assetCache = ToolAssetCache()
        weak var webView: WKWebView?

        init(_ parent: ToolWebView) {
            let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0"
            let context: [String: Any] = [
                "toolId": parent.tool.id,
                "platform": "ios",
                "appVersion": appVersion,
                "locale": Locale.current.identifier,
            ]
            bridge = LiflyBridge(context: context)
            toolURL = ToolAssetCache.entryURL(toolId: parent.tool.id)

            bridge.onSetTitle = { newTitle in
                if !newTitle.isEmpty { parent.title = newTitle }
            }
            bridge.onClose = { parent.onClose() }

            let media = self.media
            bridge.onScanDocument = { await media.scanDocument() }
            bridge.onPickPhotos = { max in await media.pickPhotos(max: max) }
        }
    }
}
