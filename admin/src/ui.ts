import type { CSSProperties } from "react";

export const btnPrimary: CSSProperties = {
  padding: "8px 14px",
  background: "#2563eb",
  color: "#fff",
  border: "none",
  borderRadius: 4,
  cursor: "pointer",
  textDecoration: "none",
  fontSize: 14,
};

export const btnSecondary: CSSProperties = {
  padding: "8px 14px",
  background: "#fff",
  color: "#374151",
  border: "1px solid #d1d5db",
  borderRadius: 4,
  cursor: "pointer",
  fontSize: 14,
};

export const linkBtn: CSSProperties = {
  background: "none",
  border: "none",
  color: "#dc2626",
  cursor: "pointer",
  padding: 0,
  font: "inherit",
};

export const th: CSSProperties = {
  textAlign: "left",
  padding: "8px 10px",
  borderBottom: "2px solid #e5e7eb",
  whiteSpace: "nowrap",
  color: "#374151",
  position: "sticky",
  top: 0,
  background: "#f9fafb",
};

export const td: CSSProperties = {
  padding: "8px 10px",
  borderBottom: "1px solid #f1f5f9",
  verticalAlign: "top",
  maxWidth: 320,
  overflow: "hidden",
  textOverflow: "ellipsis",
};
