import axios, { type AxiosAdapter, type AxiosResponse } from 'axios';
import type { ApiResponse } from './types';
import { nativeBridge } from '../embed/lifly';

const client = axios.create({
  baseURL: import.meta.env.VITE_API_URL || '',
  headers: {
    'Content-Type': 'application/json',
  },
});

// When running inside the native iOS shell, route every request through the
// native bridge. Native performs the real HTTP call and injects the JWT, so the
// token never lives in JS. The path (+ query) is forwarded; native prepends its
// own configured API base URL.
if (nativeBridge) {
  const bridge = nativeBridge;
  const bridgeAdapter: AxiosAdapter = async (config) => {
    const method = (config.method || 'get').toUpperCase();

    let path = config.url || '';
    if (config.params) {
      const qs = new URLSearchParams();
      for (const [k, v] of Object.entries(config.params as Record<string, unknown>)) {
        if (v !== undefined && v !== null) qs.append(k, String(v));
      }
      const q = qs.toString();
      if (q) path += (path.includes('?') ? '&' : '?') + q;
    }

    if (config.data instanceof FormData) {
      throw new Error('multipart upload not yet supported via native bridge');
    }

    let body: unknown = config.data;
    if (typeof body === 'string') {
      try { body = JSON.parse(body); } catch { /* leave as-is */ }
    }

    const reply = await bridge.api.request(method, path, body);
    return {
      data: reply.body,
      status: reply.status,
      statusText: '',
      headers: {},
      config,
      request: null,
    } as AxiosResponse;
  };
  client.defaults.adapter = bridgeAdapter;
}

// Attach auth token (web/browser mode only; ignored under the native bridge).
client.interceptors.request.use((config) => {
  const token = localStorage.getItem('token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Unwrap {code, data, message} response format
client.interceptors.response.use(
  (response) => {
    const body = response.data as ApiResponse<unknown>;
    if (body && typeof body === 'object' && 'code' in body) {
      if (body.code !== 0 && body.code !== 200) {
        return Promise.reject(new Error(body.message || 'Request failed'));
      }
      response.data = body.data;
    }
    return response;
  },
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      // In native shell there is no /login route; let the shell handle auth.
      if (!nativeBridge) {
        window.location.href = '/login';
      }
    }
    const message =
      error.response?.data?.message || error.message || 'Network error';
    return Promise.reject(new Error(message));
  }
);

export default client;
