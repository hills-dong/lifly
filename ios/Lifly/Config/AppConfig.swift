import Foundation

/// App-wide configuration backed by UserDefaults.
/// The backend base URL is editable at runtime (login screen / settings)
/// because Lifly is self-hosted and each user points at their own server.
enum AppConfig {
    private static let baseURLKey = "lifly.baseURL"
    private static let webBaseURLKey = "lifly.webBaseURL"

    /// Sensible default for a locally running docker-compose backend.
    static let defaultBaseURL = "http://localhost:9527"

    /// API base URL (backend). All REST calls go here.
    static var baseURL: String {
        get { UserDefaults.standard.string(forKey: baseURLKey) ?? defaultBaseURL }
        set { UserDefaults.standard.set(newValue.trimmedTrailingSlash, forKey: baseURLKey) }
    }

    /// Base URL where tool web bundles are served. In production this equals the
    /// backend (it serves the built web). For local dev it can point at a vite
    /// server. Falls back to `baseURL` when unset.
    static var webBaseURL: String {
        get { UserDefaults.standard.string(forKey: webBaseURLKey) ?? baseURL }
        set { UserDefaults.standard.set(newValue.trimmedTrailingSlash, forKey: webBaseURLKey) }
    }
}

extension String {
    var trimmedTrailingSlash: String {
        var s = trimmingCharacters(in: .whitespacesAndNewlines)
        while s.hasSuffix("/") { s.removeLast() }
        return s
    }
}
