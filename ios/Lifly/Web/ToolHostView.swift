import SwiftUI
import WebKit

/// Hosts a single tool's web UI in a WKWebView, wired to the native bridge.
struct ToolHostScreen: View {
    let tool: Tool
    @Environment(\.dismiss) private var dismiss
    @State private var title: String
    @State private var isLoading = true

    init(tool: Tool) {
        self.tool = tool
        _title = State(initialValue: tool.name)
    }

    var body: some View {
        ZStack {
            ToolWebView(tool: tool, title: $title, isLoading: $isLoading, onClose: { dismiss() })
                .ignoresSafeArea(.container, edges: .bottom)
            if isLoading {
                ProgressView()
                    .controlSize(.large)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Color(.systemBackground))
            }
        }
        .navigationTitle(title)
        .navigationBarTitleDisplayMode(.inline)
    }
}

private struct ToolWebView: UIViewRepresentable {
    let tool: Tool
    @Binding var title: String
    @Binding var isLoading: Bool
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
        config.setURLSchemeHandler(WebRuntime.shared.assetCache, forURLScheme: ToolAssetCache.scheme)

        let webView = WKWebView(frame: .zero, configuration: config)
        webView.navigationDelegate = context.coordinator
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

    final class Coordinator: NSObject, WKNavigationDelegate {
        let bridge: LiflyBridge
        let toolURL: URL?
        let media = MediaCapture()
        let setLoading: (Bool) -> Void
        weak var webView: WKWebView?

        init(_ parent: ToolWebView) {
            let appVersion = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0"
            let context: [String: Any] = [
                "toolId": parent.tool.id,
                "toolName": parent.tool.name,
                "toolDescription": parent.tool.description ?? "",
                "platform": "ios",
                "appVersion": appVersion,
                "locale": Locale.current.identifier,
            ]
            bridge = LiflyBridge(context: context)
            toolURL = ToolAssetCache.entryURL(toolId: parent.tool.id)
            setLoading = { parent.isLoading = $0 }
            super.init()

            bridge.onSetTitle = { newTitle in
                if !newTitle.isEmpty { parent.title = newTitle }
            }
            bridge.onClose = { parent.onClose() }

            let media = self.media
            bridge.onScanDocument = { await media.scanDocument() }
            bridge.onPickPhotos = { max in await media.pickPhotos(max: max) }
        }

        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
            setLoading(false)
        }

        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) {
            setLoading(false)
        }

        func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) {
            setLoading(false)
        }
    }
}
