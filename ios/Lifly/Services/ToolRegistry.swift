import Foundation
import Observation

/// Loads the user's tools once and resolves the ids the app's features need.
/// Falls back to the known system seed ids if the list can't be loaded or the
/// names don't match.
@MainActor
@Observable
final class ToolRegistry {
    private(set) var tools: [Tool] = []
    private(set) var isLoaded = false
    var loadError: String?

    // System seed ids from the backend migrations.
    static let todoSeedId = "00000000-0000-0000-0000-000000000201"
    static let documentSeedId = "00000000-0000-0000-0000-000000000202"

    func load() async {
        do {
            tools = try await LiflyAPI.listTools()
            loadError = nil
            isLoaded = true
        } catch {
            loadError = (error as? APIError)?.errorDescription ?? error.localizedDescription
        }
    }

    private func id(matchingNames names: [String], fallback: String) -> String {
        for name in names {
            if let tool = tools.first(where: { $0.name == name }) {
                return tool.id
            }
        }
        return fallback
    }

    var todoToolId: String {
        id(matchingNames: ["Todo List", "Todo", "待办"], fallback: Self.todoSeedId)
    }

    var documentToolId: String {
        id(matchingNames: ["证件管理", "ID Document", "Documents"], fallback: Self.documentSeedId)
    }
}
