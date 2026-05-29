import Foundation
import WebKit

/// JS shim injected at document start. Exposes `window.lifly` to in-WebView tools.
/// Uses WKScriptMessageHandlerWithReply, so `postMessage` returns a Promise.
enum LiflyBridgeScript {
    static let source = """
    (function () {
      function call(type, payload) {
        return window.webkit.messageHandlers.lifly.postMessage({ type: type, payload: payload || {} });
      }
      window.lifly = {
        isNative: true,
        platform: 'ios',
        getContext: function () { return call('getContext'); },
        api: {
          request: function (method, path, body) {
            return call('api.request', { method: method, path: path, body: body === undefined ? null : body });
          }
        },
        camera: { scanDocument: function () { return call('camera.scanDocument'); } },
        photos: { pick: function (opts) { return call('photos.pick', opts || {}); } },
        setTitle: function (t) { return call('setTitle', { title: t }); },
        close: function () { return call('close'); }
      };
    })();
    """
}

/// Handles messages from `window.lifly`. The bearer token is attached natively in
/// `APIClient.rawRequest`, so JavaScript never sees the JWT.
final class LiflyBridge: NSObject, WKScriptMessageHandlerWithReply {
    static let handlerName = "lifly"

    var onSetTitle: (String) -> Void = { _ in }
    var onClose: () -> Void = {}
    var onScanDocument: () async -> [String]? = { nil }
    var onPickPhotos: (Int) async -> [String]? = { _ in nil }

    let context: [String: Any]
    private let allowedHost: String?

    init(context: [String: Any]) {
        self.context = context
        self.allowedHost = URL(string: AppConfig.webBaseURL)?.host
    }

    func userContentController(
        _ userContentController: WKUserContentController,
        didReceive message: WKScriptMessage,
        replyHandler: @escaping (Any?, String?) -> Void
    ) {
        guard let dict = message.body as? [String: Any],
              let type = dict["type"] as? String else {
            replyHandler(nil, "invalid bridge message")
            return
        }
        let payload = dict["payload"] as? [String: Any] ?? [:]

        // Origin gate for privileged calls: only honor requests from the
        // first-party tool origin — either the custom cache scheme we serve, or
        // the configured web host (when loaded directly over http).
        if type == "api.request" {
            let origin = message.frameInfo.securityOrigin
            let trusted = origin.protocol == ToolAssetCache.scheme
                || (allowedHost != nil && origin.host == allowedHost)
            if !trusted {
                replyHandler(nil, "blocked: untrusted origin")
                return
            }
        }

        switch type {
        case "getContext":
            replyHandler(context, nil)

        case "api.request":
            let method = (payload["method"] as? String ?? "GET").uppercased()
            let path = payload["path"] as? String ?? ""
            let body = payload["body"]
            Task {
                do {
                    let (status, json) = try await APIClient.shared.rawRequest(
                        method: method,
                        path: path,
                        body: (body is NSNull) ? nil : body
                    )
                    replyHandler(["status": status, "body": json], nil)
                } catch {
                    replyHandler(nil, (error as? APIError)?.errorDescription ?? error.localizedDescription)
                }
            }

        case "setTitle":
            let title = payload["title"] as? String ?? ""
            DispatchQueue.main.async { self.onSetTitle(title) }
            replyHandler(nil, nil)

        case "close":
            DispatchQueue.main.async { self.onClose() }
            replyHandler(nil, nil)

        case "camera.scanDocument":
            Task {
                if let images = await onScanDocument() {
                    replyHandler(images, nil)
                } else {
                    replyHandler(nil, "cancelled")
                }
            }

        case "photos.pick":
            let max = payload["max"] as? Int ?? 1
            Task {
                if let images = await onPickPhotos(max) {
                    replyHandler(images, nil)
                } else {
                    replyHandler(nil, "cancelled")
                }
            }

        default:
            replyHandler(nil, "unknown bridge call: \(type)")
        }
    }
}
