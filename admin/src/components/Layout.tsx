import { useGetIdentity, useLogout } from "@refinedev/core";
import { Link, Outlet, useParams } from "react-router-dom";

import { useMeta } from "../meta";

const navLink: React.CSSProperties = {
  color: "#e5e7eb",
  textDecoration: "none",
  padding: "6px 8px",
  borderRadius: 4,
  fontSize: 14,
};

export function Layout() {
  const { list } = useMeta();
  const { mutate: logout } = useLogout();
  const { data: identity } = useGetIdentity<{ name?: string }>();
  const { resource } = useParams();

  return (
    <div style={{ display: "flex", minHeight: "100vh", fontFamily: "system-ui" }}>
      <aside
        style={{
          width: 220,
          background: "#111827",
          color: "#e5e7eb",
          padding: 16,
          boxSizing: "border-box",
        }}
      >
        <h2 style={{ fontSize: 16, margin: "0 0 4px" }}>Lifly Admin</h2>
        <div style={{ fontSize: 12, opacity: 0.7, marginBottom: 16 }}>{identity?.name}</div>
        <nav style={{ display: "flex", flexDirection: "column", gap: 2 }}>
          <Link to="/" style={navLink}>
            ◆ Dashboard
          </Link>
          {list.map((r) => (
            <Link
              key={r.name}
              to={`/r/${r.name}`}
              style={{
                ...navLink,
                background: resource === r.name ? "#2563eb" : "transparent",
              }}
            >
              {r.name}
            </Link>
          ))}
        </nav>
        <button
          onClick={() => logout()}
          style={{
            marginTop: 20,
            width: "100%",
            padding: "8px",
            background: "#374151",
            color: "#e5e7eb",
            border: "none",
            borderRadius: 4,
            cursor: "pointer",
          }}
        >
          退出登录
        </button>
      </aside>
      <main style={{ flex: 1, padding: 24, background: "#f9fafb", overflow: "auto" }}>
        <Outlet />
      </main>
    </div>
  );
}
