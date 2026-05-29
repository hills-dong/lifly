import Foundation

/// Thin async wrapper over the Lifly REST endpoints used by the app.
enum LiflyAPI {
    // Tools
    static func listTools() async throws -> [Tool] {
        try await APIClient.shared.get("/api/tools")
    }

    // Data objects
    static func listDataObjects(toolId: String, status: String? = "active") async throws -> [DataObject] {
        var query = [URLQueryItem(name: "tool_id", value: toolId)]
        if let status { query.append(URLQueryItem(name: "status", value: status)) }
        let payload: ListPayload<DataObject> = try await APIClient.shared.get("/api/data-objects", query: query)
        return payload.items
    }

    static func getDataObject(id: String) async throws -> DataObject {
        try await APIClient.shared.get("/api/data-objects/\(id)")
    }

    static func updateDataObject(id: String, body: UpdateDataObjectRequest) async throws -> DataObject {
        try await APIClient.shared.put("/api/data-objects/\(id)", body: body)
    }

    static func deleteDataObject(id: String) async throws {
        _ = try await APIClient.shared.delete("/api/data-objects/\(id)")
    }

    // Raw input (collection)
    static func submitRawInput(_ request: CreateRawInputRequest) async throws -> RawInputResponse {
        try await APIClient.shared.post("/api/raw-inputs", body: request)
    }

    // Pipeline status (for polling after submit)
    static func pipelineStatus(id: String) async throws -> PipelineStatus {
        try await APIClient.shared.get("/api/pipelines/\(id)")
    }
}
