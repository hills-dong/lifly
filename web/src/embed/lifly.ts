// JS side of the native bridge. The iOS shell injects `window.lifly` at document
// start. In a plain browser it is absent, and we fall back to web behavior.

export interface BridgeApiReply {
  status: number;
  body: unknown; // the {code,data,message} envelope
}

export interface NativeBridge {
  isNative: true;
  platform: string;
  getContext(): Promise<BridgeContext>;
  api: {
    request(method: string, path: string, body?: unknown): Promise<BridgeApiReply>;
  };
  camera: { scanDocument(): Promise<string[]> }; // base64 JPEGs
  photos: { pick(opts?: { max?: number }): Promise<string[]> }; // base64 JPEGs
  setTitle(title: string): void;
  close(): void;
}

export interface BridgeContext {
  toolId?: string;
  toolName?: string;
  toolDescription?: string;
  userId?: string;
  platform: string;
  appVersion?: string;
  locale?: string;
}

declare global {
  interface Window {
    lifly?: NativeBridge;
  }
}

export const nativeBridge: NativeBridge | null =
  typeof window !== 'undefined' && window.lifly?.isNative ? window.lifly : null;

export const isNative = !!nativeBridge;

/** Native shell context (tool id/name, platform). Empty-ish in a browser. */
export async function getContext(): Promise<BridgeContext> {
  if (nativeBridge) return nativeBridge.getContext();
  return { platform: 'web' };
}

/**
 * URL for an authenticated file (image) usable in <img src>. In the native shell
 * the cache proxy injects the bearer token, so no token is needed in the URL; in a
 * browser we append ?token= (the file endpoint accepts it for <img> tags).
 */
export function fileURL(fileId: string): string {
  if (nativeBridge) return `/api/files/${fileId}`;
  const token = (typeof localStorage !== 'undefined' && localStorage.getItem('token')) || '';
  return `/api/files/${fileId}?token=${encodeURIComponent(token)}`;
}

/** Set the screen title shown by the native shell (no-op-ish on web). */
export function setTitle(title: string) {
  if (nativeBridge) nativeBridge.setTitle(title);
  else document.title = title;
}

/** Dismiss the current tool and return to the native catalog (web: history back). */
export function close() {
  if (nativeBridge) nativeBridge.close();
  else window.history.back();
}

/**
 * Capture document photos. Uses VisionKit via native; on web falls back to a
 * file picker and returns base64 JPEG strings.
 */
export async function captureImages(opts?: { camera?: boolean; max?: number }): Promise<string[]> {
  if (nativeBridge) {
    return opts?.camera ? nativeBridge.camera.scanDocument() : nativeBridge.photos.pick({ max: opts?.max });
  }
  return webFilePick(opts?.max ?? 1);
}

function webFilePick(max: number): Promise<string[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/*';
    if (max > 1) input.multiple = true;
    input.onchange = async () => {
      const files = Array.from(input.files ?? []).slice(0, max);
      const out = await Promise.all(files.map(fileToBase64));
      resolve(out);
    };
    input.click();
  });
}

function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const result = reader.result as string;
      const comma = result.indexOf(',');
      resolve(comma >= 0 ? result.slice(comma + 1) : result);
    };
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
