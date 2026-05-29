import Foundation

struct LoginRequest: Encodable {
    let username: String
    let password: String
}

struct LoginResponse: Decodable {
    let token: String
    let user: UserProfile
}

struct UserProfile: Decodable, Identifiable {
    let id: String
    let username: String
    let displayName: String?

    enum CodingKeys: String, CodingKey {
        case id
        case username
        case displayName = "display_name"
    }
}
