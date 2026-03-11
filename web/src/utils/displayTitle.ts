import type { DataObject } from '../api/types';

/** Extract a display title from a data object's attributes. */
export function displayTitle(obj: DataObject): string {
  const a = obj.attributes;
  // Prefer content (original user input), then structured fields; skip base64-like data
  const candidates = [a?.content, a?.title, a?.full_name, a?.cert_type];
  for (const c of candidates) {
    if (c != null) {
      const s = String(c);
      if (s.length > 0 && s.length <= 200) return s;
    }
  }
  return 'Untitled';
}
