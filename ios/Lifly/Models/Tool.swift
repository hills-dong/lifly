import Foundation

struct Tool: Decodable, Identifiable, Hashable {
    let id: String
    let name: String
    let description: String?
    let status: String?
    let currentVersionId: String?

    enum CodingKeys: String, CodingKey {
        case id
        case name
        case description
        case status
        case currentVersionId = "current_version_id"
    }
}
