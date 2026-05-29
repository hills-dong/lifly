import { Link } from "react-router-dom";

import { useMeta } from "../meta";

export function Home() {
  const { list } = useMeta();
  return (
    <div>
      <h1 style={{ fontSize: 22 }}>数据表 ({list.length})</h1>
      <p style={{ color: "#6b7280" }}>选择一张表进行查看 / 编辑。直写数据库，请谨慎操作。</p>
      <div
        style={{
          display: "grid",
          gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))",
          gap: 12,
          marginTop: 16,
        }}
      >
        {list.map((r) => (
          <Link
            key={r.name}
            to={`/r/${r.name}`}
            style={{
              display: "block",
              padding: 16,
              background: "#fff",
              border: "1px solid #e5e7eb",
              borderRadius: 8,
              textDecoration: "none",
              color: "#111827",
            }}
          >
            <div style={{ fontWeight: 600 }}>{r.name}</div>
            <div style={{ fontSize: 12, color: "#9ca3af" }}>{r.columns.length} 列</div>
          </Link>
        ))}
      </div>
    </div>
  );
}
