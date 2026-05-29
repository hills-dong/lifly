import { useCreate } from "@refinedev/core";
import { useState } from "react";
import { useNavigate, useParams } from "react-router-dom";

import { Field, buildPayload, rowToFormValues } from "../fields";
import { editableColumns, useMeta } from "../meta";
import { btnPrimary, btnSecondary } from "../ui";

export function ResourceCreate() {
  const { resource } = useParams();
  const { resources } = useMeta();
  const spec = resource ? resources[resource] : undefined;
  const navigate = useNavigate();

  const { mutate: create, mutation } = useCreate();
  const [values, setValues] = useState<Record<string, any>>(() =>
    spec ? rowToFormValues(spec, undefined) : {},
  );
  const [error, setError] = useState("");

  if (!spec) return <div>未知资源：{resource}</div>;

  const submit = (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    let payload: Record<string, any>;
    try {
      payload = buildPayload(spec, values);
    } catch (err: any) {
      setError(err.message);
      return;
    }
    create(
      { resource: resource!, values: payload },
      {
        onSuccess: () => navigate(`/r/${resource}`),
        onError: () => setError("创建失败（可能缺少必填字段或违反约束）"),
      },
    );
  };

  return (
    <div style={{ maxWidth: 640 }}>
      <h1 style={{ fontSize: 20 }}>新建 {resource}</h1>
      <form onSubmit={submit}>
        {editableColumns(spec).map((col) => (
          <Field
            key={col.name}
            col={col}
            value={values[col.name]}
            onChange={(v) => setValues((s) => ({ ...s, [col.name]: v }))}
          />
        ))}
        {error && (
          <div data-testid="form-error" style={{ color: "#dc2626", marginBottom: 12 }}>
            {error}
          </div>
        )}
        <div style={{ display: "flex", gap: 8 }}>
          <button type="submit" disabled={mutation.isPending} style={btnPrimary}>
            {mutation.isPending ? "创建中…" : "创建"}
          </button>
          <button type="button" onClick={() => navigate(`/r/${resource}`)} style={btnSecondary}>
            取消
          </button>
        </div>
      </form>
    </div>
  );
}
