import SwiftUI

struct RootView: View {
    @Environment(AuthStore.self) private var auth
    @State private var didBootstrap = false

    var body: some View {
        Group {
            if auth.isAuthenticated {
                MainTabView()
            } else {
                LoginView()
            }
        }
        .task {
            guard !didBootstrap else { return }
            didBootstrap = true
            await auth.bootstrap()
        }
    }
}
