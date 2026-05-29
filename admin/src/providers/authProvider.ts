import type { AuthProvider } from "@refinedev/core";

import { TOKEN_KEY, http } from "../http";

/**
 * Config-based admin auth. Talks to `/api/admin/login` and stores the issued
 * admin JWT in localStorage. Independent of the user account system.
 */
export const authProvider: AuthProvider = {
  login: async ({ username, password }) => {
    try {
      const res = await http.post("/api/admin/login", { username, password });
      const token = res.data?.data?.token as string | undefined;
      if (token) {
        localStorage.setItem(TOKEN_KEY, token);
        return { success: true, redirectTo: "/" };
      }
    } catch {
      // fall through to the generic failure below
    }
    return {
      success: false,
      error: { name: "登录失败", message: "用户名或密码错误" },
    };
  },

  logout: async () => {
    localStorage.removeItem(TOKEN_KEY);
    return { success: true, redirectTo: "/login" };
  },

  check: async () => {
    const token = localStorage.getItem(TOKEN_KEY);
    return token
      ? { authenticated: true }
      : { authenticated: false, redirectTo: "/login" };
  },

  getIdentity: async () => {
    try {
      const res = await http.get("/api/admin/me");
      return { name: res.data?.data?.username as string };
    } catch {
      return null;
    }
  },

  onError: async (error) => {
    if (error?.response?.status === 401) {
      localStorage.removeItem(TOKEN_KEY);
      return { logout: true, redirectTo: "/login" };
    }
    return {};
  },

  getPermissions: async () => null,
};
