import SwiftUI
import WebKit

/// Hosts a tool by reparenting the shared persistent WebView into this screen and
/// switching it to the given tool. JS is parsed once (in WebRuntime), so opening
/// a tool is near-instant after the first load.
struct ToolHostScreen: View {
    let tool: Tool
    @Environment(\.dismiss) private var dismiss
    private let runtime = WebRuntime.shared

    var body: some View {
        WebHostContainer()
            .ignoresSafeArea(.container, edges: .bottom)
            .overlay {
                if !runtime.isReady {
                    ProgressView()
                        .controlSize(.large)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .background(Color(.systemBackground))
                }
            }
            .navigationTitle(runtime.title)
            .navigationBarTitleDisplayMode(.inline)
            .onAppear {
                runtime.onClose = { dismiss() }
                runtime.open(tool)
            }
    }
}

/// Reparents the shared WebView into this screen's container view.
private struct WebHostContainer: UIViewRepresentable {
    func makeUIView(context: Context) -> UIView { UIView() }

    func updateUIView(_ container: UIView, context: Context) {
        WebRuntime.shared.ensureLoaded()
        guard let webView = WebRuntime.shared.webView else { return }
        if webView.superview !== container {
            webView.removeFromSuperview()
            webView.translatesAutoresizingMaskIntoConstraints = false
            container.addSubview(webView)
            NSLayoutConstraint.activate([
                webView.topAnchor.constraint(equalTo: container.topAnchor),
                webView.leadingAnchor.constraint(equalTo: container.leadingAnchor),
                webView.trailingAnchor.constraint(equalTo: container.trailingAnchor),
                webView.bottomAnchor.constraint(equalTo: container.bottomAnchor),
            ])
        }
    }
}
