import { useDelete, useList } from "@refinedev/core";
import { useState } from "react";
import { Link, useParams } from "react-router-dom";

import { useMeta, visibleColumns } from "../meta";
import { btnPrimary, linkBtn, td, th } from "../ui";

const PAGE_SIZE = 20;

function renderCell(v: any) {
  if (v === null || v === undefined) return <span style={{ color: "#cbd5e1" }}>—</span>;
  if (typeof v === "object") {
    const s = JSON.stringify(v);
    return <code style={{ fontSize: 12 }}>{s.length > 60 ? s.slice(0, 60) + "…" : s}</code>;
  }
  const s = String(v);
  return s.length > 80 ? s.slice(0, 80) + "…" : s;
}

export function ResourceList() {
  const { resource } = useParams();
  const { resources } = useMeta();
  const spec = resource ? resources[resource] : undefined;
  const [page, setPage] = useState(1);

  const { result, query } = useList({
    resource,
    pagination: { currentPage: page, pageSize: PAGE_SIZE },
    queryOptions: { enabled: !!resource },
  });
  const { mutate: del } = useDelete();

  if (!spec) return <div>未知资源：{resource}</div>;

  const rows = (result?.data ?? []) as Record<string, any>[];
  const total = result?.total ?? 0;
  const cols = visibleColumns(spec);
  const pk = spec.pk;
  const pages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  return (
    <div>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <h1 style={{ fontSize: 22 }}>
          {resource}{" "}
          <span data-testid="total" style={{ fontSize: 14, color: "#6b7280" }}>
            共 {total} 行
          </span>
        </h1>
        <Link to={`/r/${resource}/create`} style={btnPrimary}>
          + 新建
        </Link>
      </div>

      {query.isLoading ? (
        <p>加载中…</p>
      ) : (
        <div
          style={{
            overflowX: "auto",
            background: "#fff",
            border: "1px solid #e5e7eb",
            borderRadius: 8,
            marginTop: 12,
            maxHeight: "70vh",
          }}
        >
          <table data-testid="resource-table" style={{ borderCollapse: "collapse", width: "100%", fontSize: 13 }}>
            <thead>
              <tr>
                {cols.map((c) => (
                  <th key={c.name} style={th}>
                    {c.name}
                  </th>
                ))}
                <th style={th}>操作</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={String(row[pk])} data-testid="resource-row">
                  {cols.map((c) => (
                    <td key={c.name} style={td}>
                      {renderCell(row[c.name])}
                    </td>
                  ))}
                  <td style={{ ...td, whiteSpace: "nowrap" }}>
                    <Link to={`/r/${resource}/${row[pk]}`}>编辑</Link>
                    {" · "}
                    <button
                      style={linkBtn}
                      onClick={() => {
                        if (confirm("确认删除该行？此操作直接作用于数据库。")) {
                          del(
                            { resource: resource!, id: row[pk] },
                            { onSuccess: () => query.refetch() },
                          );
                        }
                      }}
                    >
                      删除
                    </button>
                  </td>
                </tr>
              ))}
              {rows.length === 0 && (
                <tr>
                  <td colSpan={cols.length + 1} style={{ ...td, color: "#9ca3af" }}>
                    无数据
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      <div style={{ marginTop: 12, display: "flex", gap: 8, alignItems: "center" }}>
        <button disabled={page <= 1} onClick={() => setPage((p) => p - 1)}>
          上一页
        </button>
        <span>
          第 {page} / {pages} 页
        </span>
        <button disabled={page >= pages} onClick={() => setPage((p) => p + 1)}>
          下一页
        </button>
      </div>
    </div>
  );
}
