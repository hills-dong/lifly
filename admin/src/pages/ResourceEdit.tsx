import { useOne, useUpdate } from "@refinedev/core";
import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router-dom";

import { Field, buildPayload, rowToFormValues } from "../fields";
import { editableColumns, useMeta } from "../meta";
import { btnPrimary, btnSecondary } from "../ui";

export function ResourceEdit() {
  const { resource, id } = useParams();
  const { resources } = useMeta();
  const spec = resource ? resources[resource] : undefined;
  const navigate = useNavigate();

  const { result, query } = useOne({
    resource,
    id,
    queryOptions: { enabled: !!resource && !!id },
  });
  const { mutate: update, mutation } = useUpdate();

  const [values, setValues] = useState<Record<string, any>>({});
  const [error, setError] = useState("");

  useEffect(() => {
    if (spec && result) setValues(rowToFormValues(spec, result as Record<string, any>));
  }, [spec, result]);

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
    update(
      { resource: resource!, id: id!, values: payload },
      {
        onSuccess: () => navigate(`/r/${resource}`),
        onError: () => setError("保存失败"),
      },
    );
  };

  return (
    <div style={{ maxWidth: 640 }}>
      <h1 style={{ fontSize: 20 }}>编辑 {resource}</h1>
      <div style={{ fontSize: 12, color: "#9ca3af", marginBottom: 16 }}>
        {spec.pk}: {id}
      </div>
      {query.isLoading ? (
        <p>加载中…</p>
      ) : (
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
              {mutation.isPending ? "保存中…" : "保存"}
            </button>
            <button type="button" onClick={() => navigate(`/r/${resource}`)} style={btnSecondary}>
              取消
            </button>
          </div>
        </form>
      )}
    </div>
  );
}
