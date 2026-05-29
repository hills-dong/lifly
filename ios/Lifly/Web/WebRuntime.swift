import Foundation
import WebKit

/// Shared web runtime: one asset cache reused by every tool WebView, plus a
/// hidden prewarm WebView that warms the web content process / JS engine and the
/// on-disk asset cache so the first tool opens fast.
/// (WKProcessPool is intentionally not used — it's a no-op on modern WebKit,
/// which already shares the web content process.)
@MainActor
final class WebRuntime {
    static let shared = WebRuntime()

    let assetCache = ToolAssetCache()

    private var prewarmWebView: WKWebView?

    private init() {}

    /// Warm the content process + asset cache. Loads `/prewarm`, which the SPA
    /// serves (index.html fallback) but matches no route, so it fetches and caches
    /// the JS/CSS bundle without making any tool API calls.
    func prewarmIfNeeded() {
        guard prewarmWebView == nil else { return }

        let config = WKWebViewConfiguration()
        config.setURLSchemeHandler(assetCache, forURLScheme: ToolAssetCache.scheme)

        let webView = WKWebView(frame: .zero, configuration: config)
        if let url = URL(string: "\(ToolAssetCache.scheme):///prewarm") {
            webView.load(URLRequest(url: url))
        }
        prewarmWebView = webView
    }
}
