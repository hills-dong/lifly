import Foundation

/// Envelope every Lifly endpoint wraps responses in: `{ code, data, message }`.
/// Success is `code == 0` or `code == 200`.
private struct APIEnvelope<T: Decodable>: Decodable {
    let code: Int
    let data: T?
    let message: String?
}

/// Placeholder for endpoints whose `data` is null (e.g. logout, delete).
struct EmptyResponse: Decodable {}

/// A list payload that tolerates either a bare array (`[...]`) or a paginated
/// wrapper (`{ items: [...], total, limit, offset }`). The backend uses both
/// shapes across endpoints, so we accept either.
struct ListPayload<Element: Decodable>: Decodable {
    let items: [Element]

    init(from decoder: Decoder) throws {
        if let single = try? decoder.singleValueContainer(),
           let array = try? single.decode([Element].self) {
            items = array
            return
        }
        let keyed = try decoder.container(keyedBy: CodingKeys.self)
        items = try keyed.decodeIfPresent([Element].self, forKey: .items) ?? []
    }

    private enum CodingKeys: String, CodingKey { case items }
}

final class APIClient: @unchecked Sendable {
    static let shared = APIClient()

    /// Supplies the current bearer token. Set by AuthStore.
    var tokenProvider: () -> String? = { nil }
    /// Invoked when the server returns 401 so the app can sign the user out.
    var onUnauthorized: () -> Void = {}

    private let session: URLSession

    private init() {
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 20
        config.timeoutIntervalForResource = 40
        // Fail fast when the self-hosted server is unreachable instead of
        // waiting against the multi-day resource timeout.
        config.waitsForConnectivity = false
        session = URLSession(configuration: config)
    }

    private var decoder: JSONDecoder { JSONDecoder() }

    // MARK: - Public verbs

    func get<T: Decodable>(_ path: String, query: [URLQueryItem] = []) async throws -> T {
        try await send(method: "GET", path: path, query: query, body: Optional<Int>.none)
    }

    func post<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        try await send(method: "POST", path: path, query: [], body: body)
    }

    func postNoBody<T: Decodable>(_ path: String) async throws -> T {
        try await send(method: "POST", path: path, query: [], body: Optional<Int>.none)
    }

    func put<T: Decodable, B: Encodable>(_ path: String, body: B) async throws -> T {
        try await send(method: "PUT", path: path, query: [], body: body)
    }

    @discardableResult
    func delete(_ path: String) async throws -> EmptyResponse {
        try await send(method: "DELETE", path: path, query: [], body: Optional<Int>.none)
    }

    // MARK: - Raw passthrough (for the web bridge)

    /// Performs a request on behalf of an in-WebView tool. The `path` may already
    /// include a query string. The bearer token is injected here so it never has
    /// to be exposed to JavaScript. Returns the HTTP status and the parsed JSON
    /// body (the `{code,data,message}` envelope) as a Foundation object.
    func rawRequest(method: String, path: String, body: Any?) async throws -> (status: Int, json: Any) {
        guard let url = URL(string: AppConfig.baseURL + path) else {
            throw APIError.invalidBaseURL
        }
        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        if let token = tokenProvider() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        if let body, !(body is NSNull) {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = try JSONSerialization.data(withJSONObject: body)
        }

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.network(error.localizedDescription)
        }
        let status = (response as? HTTPURLResponse)?.statusCode ?? 0
        if status == 401 { onUnauthorized() }

        let json: Any = data.isEmpty ? NSNull() : ((try? JSONSerialization.jsonObject(with: data)) ?? NSNull())
        return (status, json)
    }

    // MARK: - Core

    private func send<T: Decodable, B: Encodable>(
        method: String,
        path: String,
        query: [URLQueryItem],
        body: B?
    ) async throws -> T {
        guard var components = URLComponents(string: AppConfig.baseURL) else {
            throw APIError.invalidBaseURL
        }
        components.path += path
        if !query.isEmpty { components.queryItems = query }
        guard let url = components.url else { throw APIError.invalidBaseURL }

        var request = URLRequest(url: url)
        request.httpMethod = method
        request.setValue("application/json", forHTTPHeaderField: "Accept")
        if let token = tokenProvider() {
            request.setValue("Bearer \(token)", forHTTPHeaderField: "Authorization")
        }
        if let body {
            request.setValue("application/json", forHTTPHeaderField: "Content-Type")
            request.httpBody = try JSONEncoder().encode(body)
        }

        let data: Data
        let response: URLResponse
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.network(error.localizedDescription)
        }

        guard let http = response as? HTTPURLResponse else {
            throw APIError.network("无效的响应")
        }
        if http.statusCode == 401 {
            onUnauthorized()
            throw APIError.unauthorized
        }

        return try decodeEnvelope(data, status: http.statusCode)
    }

    private func decodeEnvelope<T: Decodable>(_ data: Data, status: Int) throws -> T {
        // Try the standard envelope first.
        if let envelope = try? decoder.decode(APIEnvelope<T>.self, from: data) {
            if envelope.code == 0 || envelope.code == 200 {
                if let payload = envelope.data {
                    return payload
                }
                if let empty = EmptyResponse() as? T {
                    return empty
                }
                // data was null but caller expects a value.
                throw APIError.decoding("响应缺少 data 字段")
            }
            throw APIError.server(code: envelope.code, message: envelope.message ?? "请求失败")
        }

        // Some endpoints (or proxies) may return the raw object without an envelope.
        if status >= 200, status < 300, let raw = try? decoder.decode(T.self, from: data) {
            return raw
        }

        let snippet = String(data: data, encoding: .utf8)?.prefix(200) ?? ""
        throw APIError.decoding(String(snippet))
    }
}
