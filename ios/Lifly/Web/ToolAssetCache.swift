import Foundation
import WebKit
import CryptoKit

/// Caching proxy for tool web bundles, served under a custom scheme so the
/// WebView can load tools from the app sandbox.
///
/// Strategy:
/// - Content-hashed assets (`/assets/...`) are immutable → cache-first.
/// - HTML and everything else → network-first with cache fallback (so a new web
///   deploy hot-loads when online, and tools still open offline).
///
/// Custom-scheme URLs (`liflyapp:///path`) are mapped back to the configured
/// `webBaseURL` origin for fetching.
final class ToolAssetCache: NSObject, WKURLSchemeHandler {
    static let scheme = "liflyapp"

    private let session: URLSession
    private let cacheDir: URL
    private var activeTasks = Set<ObjectIdentifier>()
    private let lock = NSLock()

    override init() {
        let config = URLSessionConfiguration.default
        config.requestCachePolicy = .reloadIgnoringLocalCacheData
        config.timeoutIntervalForRequest = 20
        session = URLSession(configuration: config)

        let caches = FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask)[0]
        cacheDir = caches.appendingPathComponent("ToolBundles", isDirectory: true)
        try? FileManager.default.createDirectory(at: cacheDir, withIntermediateDirectories: true)
        super.init()
    }

    /// Builds the custom-scheme entry URL for a tool.
    static func entryURL(toolId: String) -> URL? {
        URL(string: "\(scheme):///embed/tools/\(toolId)")
    }

    // MARK: WKURLSchemeHandler

    func webView(_ webView: WKWebView, start urlSchemeTask: WKURLSchemeTask) {
        let id = ObjectIdentifier(urlSchemeTask)
        setActive(id, true)

        guard let requestURL = urlSchemeTask.request.url,
              let realURL = mapToReal(requestURL) else {
            fail(urlSchemeTask, id: id)
            return
        }

        let isImmutableAsset = realURL.path.contains("/assets/")
        let cacheFile = cachePath(for: realURL)

        if isImmutableAsset, let data = try? Data(contentsOf: cacheFile) {
            respond(urlSchemeTask, id: id, requestURL: requestURL, data: data, mime: mimeType(for: realURL))
            return
        }

        let task = session.dataTask(with: realURL) { [weak self] data, response, _ in
            guard let self else { return }
            if let data, let http = response as? HTTPURLResponse, (200..<300).contains(http.statusCode) {
                try? data.write(to: cacheFile, options: .atomic)
                let mime = http.mimeType ?? self.mimeType(for: realURL)
                self.respond(urlSchemeTask, id: id, requestURL: requestURL, data: data, mime: mime)
            } else if let cached = try? Data(contentsOf: cacheFile) {
                self.respond(urlSchemeTask, id: id, requestURL: requestURL, data: cached, mime: self.mimeType(for: realURL))
            } else {
                self.fail(urlSchemeTask, id: id)
            }
        }
        task.resume()
    }

    func webView(_ webView: WKWebView, stop urlSchemeTask: WKURLSchemeTask) {
        setActive(ObjectIdentifier(urlSchemeTask), false)
    }

    // MARK: Helpers

    private func mapToReal(_ url: URL) -> URL? {
        guard var comps = URLComponents(string: AppConfig.webBaseURL),
              let reqComps = URLComponents(url: url, resolvingAgainstBaseURL: false) else {
            return nil
        }
        comps.path = reqComps.path
        comps.query = reqComps.query
        return comps.url
    }

    private func cachePath(for url: URL) -> URL {
        var key = url.path
        if let q = url.query { key += "?" + q }
        // Stable across launches (Swift's String.hashValue is per-process salted).
        let digest = SHA256.hash(data: Data(key.utf8))
        let name = digest.map { String(format: "%02x", $0) }.joined()
        return cacheDir.appendingPathComponent(name)
    }

    private func respond(_ task: WKURLSchemeTask, id: ObjectIdentifier, requestURL: URL, data: Data, mime: String) {
        DispatchQueue.main.async {
            guard self.isActive(id) else { return }
            let textEncoding = (mime.hasPrefix("text/") || mime.contains("javascript") || mime.contains("json") || mime.contains("svg")) ? "utf-8" : nil
            let response = URLResponse(url: requestURL, mimeType: mime, expectedContentLength: data.count, textEncodingName: textEncoding)
            task.didReceive(response)
            task.didReceive(data)
            task.didFinish()
            self.setActive(id, false)
        }
    }

    private func fail(_ task: WKURLSchemeTask, id: ObjectIdentifier) {
        DispatchQueue.main.async {
            guard self.isActive(id) else { return }
            task.didFailWithError(URLError(.resourceUnavailable))
            self.setActive(id, false)
        }
    }

    private func mimeType(for url: URL) -> String {
        switch url.pathExtension.lowercased() {
        case "html", "": return "text/html"
        case "js", "mjs": return "text/javascript"
        case "css": return "text/css"
        case "json": return "application/json"
        case "svg": return "image/svg+xml"
        case "png": return "image/png"
        case "jpg", "jpeg": return "image/jpeg"
        case "webp": return "image/webp"
        case "woff2": return "font/woff2"
        case "woff": return "font/woff"
        case "ttf": return "font/ttf"
        case "ico": return "image/x-icon"
        default: return "application/octet-stream"
        }
    }

    private func setActive(_ id: ObjectIdentifier, _ active: Bool) {
        lock.lock(); defer { lock.unlock() }
        if active { activeTasks.insert(id) } else { activeTasks.remove(id) }
    }

    private func isActive(_ id: ObjectIdentifier) -> Bool {
        lock.lock(); defer { lock.unlock() }
        return activeTasks.contains(id)
    }
}
