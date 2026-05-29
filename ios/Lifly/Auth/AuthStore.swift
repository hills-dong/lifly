import Foundation
import Observation

@MainActor
@Observable
final class AuthStore {
    enum State {
        case unauthenticated
        case authenticated(UserProfile)
    }

    private(set) var state: State = .unauthenticated
    var serverURL: String = AppConfig.baseURL
    var isLoggingIn = false
    var loginError: String?

    private var token: String?

    var currentUser: UserProfile? {
        if case .authenticated(let user) = state { return user }
        return nil
    }

    var isAuthenticated: Bool {
        if case .authenticated = state { return true }
        return false
    }

    init() {
        APIClient.shared.tokenProvider = { [weak self] in self?.token }
        APIClient.shared.onUnauthorized = { [weak self] in
            Task { @MainActor in self?.signOut() }
        }
    }

    /// Restore a prior session if a token is stored. Validates by fetching the profile.
    func bootstrap() async {
        guard let stored = KeychainHelper.readToken() else { return }
        token = stored
        do {
            let profile: UserProfile = try await APIClient.shared.get("/api/user/profile")
            state = .authenticated(profile)
        } catch {
            // Token invalid/expired or server unreachable — stay logged out.
            token = nil
            KeychainHelper.deleteToken()
        }
    }

    func login(username: String, password: String) async {
        loginError = nil
        isLoggingIn = true
        defer { isLoggingIn = false }

        AppConfig.baseURL = serverURL
        serverURL = AppConfig.baseURL

        do {
            let response: LoginResponse = try await APIClient.shared.post(
                "/api/auth/login",
                body: LoginRequest(username: username, password: password)
            )
            token = response.token
            KeychainHelper.saveToken(response.token)
            state = .authenticated(response.user)
        } catch {
            loginError = (error as? APIError)?.errorDescription ?? error.localizedDescription
        }
    }

    func signOut() {
        token = nil
        KeychainHelper.deleteToken()
        state = .unauthenticated
        Task { _ = try? await APIClient.shared.postNoBody("/api/auth/logout") as EmptyResponse }
    }
}
