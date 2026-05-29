import SwiftUI

/// Home screen: a dynamic list of the user's tools fetched from the backend.
/// New tools (including ones created at runtime) appear on refresh — no app
/// update needed. Tapping a tool opens its web UI in a WKWebView host.
struct ToolCatalogView: View {
    @State private var registry = ToolRegistry()

    var body: some View {
        NavigationStack {
            Group {
                if !registry.isLoaded {
                    if let error = registry.loadError {
                        VStack(spacing: 12) {
                            Text(error).font(.footnote).foregroundStyle(.red)
                                .multilineTextAlignment(.center).padding(.horizontal)
                            Button("重试") { Task { await registry.load() } }
                        }
                    } else {
                        ProgressView("加载工具…")
                    }
                } else if registry.tools.isEmpty {
                    ContentUnavailableView("还没有工具", systemImage: "square.grid.2x2",
                        description: Text("在 Web/桌面端创建工具后，下拉刷新即可在这里看到。"))
                } else {
                    List(registry.tools) { tool in
                        NavigationLink(value: tool) {
                            ToolCatalogRow(tool: tool)
                        }
                    }
                }
            }
            .navigationTitle("工具")
            .navigationDestination(for: Tool.self) { tool in
                ToolHostScreen(tool: tool)
            }
            .refreshable { await registry.load() }
            .task { if !registry.isLoaded { await registry.load() } }
        }
    }
}

private struct ToolCatalogRow: View {
    let tool: Tool

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: icon)
                .font(.title3)
                .frame(width: 32, height: 32)
                .foregroundStyle(Color.accentColor)
            VStack(alignment: .leading, spacing: 2) {
                Text(tool.name).font(.body)
                if let desc = tool.description, !desc.isEmpty {
                    Text(desc).font(.caption).foregroundStyle(.secondary).lineLimit(1)
                }
            }
        }
        .padding(.vertical, 2)
    }

    private var icon: String {
        let s = (tool.name + " " + (tool.description ?? "")).lowercased()
        if s.contains("todo") || s.contains("待办") { return "checklist" }
        if s.contains("证件") || s.contains("document") { return "doc.text.image" }
        return "square.grid.2x2"
    }
}
