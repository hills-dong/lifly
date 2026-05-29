import Foundation

/// A tool's produced business data. The meaningful fields live in `attributes`,
/// whose shape is defined by the tool's data_schema. Top-level fields are kept
/// optional because the exact response DTO varies by backend version.
struct DataObject: Decodable, Identifiable {
    let id: String
    let toolId: String?
    let title: String?
    let status: String?
    let attributes: [String: JSONValue]
    let createdAt: String?
    let updatedAt: String?

    enum CodingKeys: String, CodingKey {
        case id
        case toolId = "tool_id"
        case title
        case status
        case attributes
        case createdAt = "created_at"
        case updatedAt = "updated_at"
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        id = try c.decode(String.self, forKey: .id)
        toolId = try c.decodeIfPresent(String.self, forKey: .toolId)
        title = try c.decodeIfPresent(String.self, forKey: .title)
        status = try c.decodeIfPresent(String.self, forKey: .status)
        attributes = try c.decodeIfPresent([String: JSONValue].self, forKey: .attributes) ?? [:]
        createdAt = try c.decodeIfPresent(String.self, forKey: .createdAt)
        updatedAt = try c.decodeIfPresent(String.self, forKey: .updatedAt)
    }
}

// MARK: - Request bodies

struct CreateRawInputRequest: Encodable {
    let toolId: String
    let inputType: String
    let rawContent: String
    let metadata: [String: JSONValue]?

    enum CodingKeys: String, CodingKey {
        case toolId = "tool_id"
        case inputType = "input_type"
        case rawContent = "raw_content"
        case metadata
    }
}

struct UpdateDataObjectRequest: Encodable {
    let title: String?
    let attributes: [String: JSONValue]?
    let status: String?
}

/// Returned by POST /api/raw-inputs — carries the pipeline id so we can poll.
struct RawInputResponse: Decodable {
    let id: String
    let processingStatus: String?
    let pipelineId: String?

    enum CodingKeys: String, CodingKey {
        case id
        case processingStatus = "processing_status"
        case pipelineId = "pipeline_id"
    }
}

struct PipelineStatus: Decodable {
    let id: String
    let status: String

    enum CodingKeys: String, CodingKey {
        case id
        case status
    }
}
