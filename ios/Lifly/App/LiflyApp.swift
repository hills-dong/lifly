import SwiftUI

@main
struct LiflyApp: App {
    @State private var auth = AuthStore()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environment(auth)
        }
    }
}
