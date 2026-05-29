import SwiftUI

struct SettingsView: View {
    @Environment(AuthStore.self) private var auth

    var body: some View {
        NavigationStack {
            Form {
                Section("当前用户") {
                    LabeledContent("用户名", value: auth.currentUser?.username ?? "-")
                    if let name = auth.currentUser?.displayName, !name.isEmpty {
                        LabeledContent("显示名", value: name)
                    }
                }
                Section("服务器") {
                    LabeledContent("地址", value: auth.serverURL)
                }
                Section {
                    Button(role: .destructive) {
                        auth.signOut()
                    } label: {
                        Text("退出登录")
                    }
                }
                Section {
                    LabeledContent("版本", value: appVersion)
                }
            }
            .navigationTitle("设置")
        }
    }

    private var appVersion: String {
        let v = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String ?? "0"
        let b = Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "0"
        return "\(v) (\(b))"
    }
}
