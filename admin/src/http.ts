import axios from "axios";

/** LocalStorage key holding the admin JWT. */
export const TOKEN_KEY = "lifly-admin-token";

/**
 * Shared axios instance. Requests are relative (`/api/...`) so they work both
 * behind the Vite dev proxy and when the build is served by the backend.
 */
export const http = axios.create();

http.interceptors.request.use((config) => {
  const token = localStorage.getItem(TOKEN_KEY);
  if (token) {
    config.headers = config.headers ?? {};
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
