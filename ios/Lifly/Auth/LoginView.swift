import SwiftUI

struct LoginView: View {
    @Environment(AuthStore.self) private var auth

    @State private var username = "admin"
    @State private var password = ""

    var body: some View {
        @Bindable var auth = auth
        NavigationStack {
            Form {
                Section("服务器") {
                    TextField("http://192.168.x.x:9527", text: $auth.serverURL)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                        .keyboardType(.URL)
                }
                Section("账号") {
                    TextField("用户名", text: $username)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                    SecureField("密码", text: $password)
                }
                if let error = auth.loginError {
                    Section {
                        Text(error).foregroundStyle(.red).font(.footnote)
                    }
                }
                Section {
                    Button {
                        Task { await auth.login(username: username, password: password) }
                    } label: {
                        HStack {
                            Spacer()
                            if auth.isLoggingIn { ProgressView() } else { Text("登录") }
                            Spacer()
                        }
                    }
                    .disabled(auth.isLoggingIn || username.isEmpty || password.isEmpty || auth.serverURL.isEmpty)
                }
            }
            .navigationTitle("登录 Lifly")
        }
    }
}
