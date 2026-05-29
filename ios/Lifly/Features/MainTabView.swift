import SwiftUI

struct MainTabView: View {
    var body: some View {
        TabView {
            ToolCatalogView()
                .tabItem { Label("工具", systemImage: "square.grid.2x2") }

            SettingsView()
                .tabItem { Label("设置", systemImage: "gearshape") }
        }
    }
}
