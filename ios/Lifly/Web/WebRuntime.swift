import Foundation
import Observation
import WebKit

/// Owns a single persistent WKWebView that hosts the embed SPA. The JS bundle is
/// parsed once; switching tools is a client-side navigation (native updates the
/// context and calls `__lifly_refresh()`), so opening a tool is near-instant
/// instead of spinning up a fresh WebView each time.
@MainActor
@Observable
final class WebRuntime {
    static let shared = WebRuntime()

    let assetCache = ToolAssetCache()

    // Host-screen UI state.
    var title: String = ""
    var isReady: Bool = false
    @ObservationIgnored var onClose: () -> Void = {}

    // Current tool, surfaced to the web via the bridge `getContext`.
    @ObservationIgnored private var currentToolId = ""
    @ObservationIgnored private var currentToolName = ""
    @ObservationIgnored private var currentToolDescription = ""

    @ObservationIgnored private(set) var webView: WKWebView?
    @ObservationIgnored private var bridge: LiflyBridge!
    @ObservationIgnored private let media = MediaCapture()
    @ObservationIgnored private let navDelegate = NavDelegate()

    private init() {
        bridge = LiflyBridge(contextProvider: { [weak self] in
            guard let self else { return ["platform": "ios"] }
            return [
                "toolId": self.currentToolId,
                "toolName": self.currentToolName,
                "toolDescription": self.currentToolDescription,
                "platform": "ios",
                "appVersion": Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0",
                "locale": Locale.current.identifier,
            ]
        })
        bridge.onSetTitle = { [weak self] t in
            if !t.isEmpty { self?.title = t }
        }
        bridge.onClose = { [weak self] in self?.onClose() }
        let media = media
        bridge.onScanDocument = { await media.scanDocument() }
        bridge.onPickPhotos = { max in await media.pickPhotos(max: max) }

        navDelegate.onFinish = { [weak self] in self?.isReady = true }
    }

    /// Create and load the persistent WebView once (call on app/catalog start).
    func ensureLoaded() {
        guard webView == nil else { return }

        let controller = WKUserContentController()
        controller.addUserScript(
            WKUserScript(source: LiflyBridgeScript.source, injectionTime: .atDocumentStart, forMainFrameOnly: true)
        )
        controller.addScriptMessageHandler(bridge, contentWorld: .page, name: LiflyBridge.handlerName)

        let config = WKWebViewConfiguration()
        config.userContentController = controller
        config.setURLSchemeHandler(assetCache, forURLScheme: ToolAssetCache.scheme)

        let wv = WKWebView(frame: .zero, configuration: config)
        wv.navigationDelegate = navDelegate
        wv.allowsBackForwardNavigationGestures = false
        wv.scrollView.contentInsetAdjustmentBehavior = .always
        webView = wv

        if let url = URL(string: "\(ToolAssetCache.scheme):///embed") {
            wv.load(URLRequest(url: url))
        }
    }

    /// Switch the persistent WebView to a tool (client-side, no reload).
    func open(_ tool: Tool) {
        ensureLoaded()
        currentToolId = tool.id
        currentToolName = tool.name
        currentToolDescription = tool.description ?? ""
        title = tool.name
        // No-op if the web hasn't mounted yet; it reads context on first mount.
        webView?.evaluateJavaScript("window.__lifly_refresh && window.__lifly_refresh()", completionHandler: nil)
    }

    private final class NavDelegate: NSObject, WKNavigationDelegate {
        var onFinish: () -> Void = {}
        func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) { onFinish() }
        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { onFinish() }
        func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { onFinish() }
    }
}
