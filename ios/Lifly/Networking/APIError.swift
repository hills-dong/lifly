import Foundation

enum APIError: LocalizedError, Equatable {
    case invalidBaseURL
    case unauthorized
    case server(code: Int, message: String)
    case decoding(String)
    case network(String)

    var errorDescription: String? {
        switch self {
        case .invalidBaseURL:
            return "服务器地址无效，请在登录页检查。"
        case .unauthorized:
            return "登录已过期，请重新登录。"
        case .server(_, let message):
            return message
        case .decoding(let detail):
            return "返回数据解析失败：\(detail)"
        case .network(let detail):
            return "网络请求失败：\(detail)"
        }
    }
}
