import type { ColumnSpec, ResourceSpec } from "./meta";
import { editableColumns } from "./meta";

/** Build the initial form state for a row (string-ish values for inputs). */
export function rowToFormValues(
  spec: ResourceSpec,
  row: Record<string, any> | undefined,
): Record<string, any> {
  const out: Record<string, any> = {};
  for (const col of editableColumns(spec)) {
    const v = row?.[col.name];
    if (col.type === "bool") {
      out[col.name] = !!v;
    } else if (col.type === "json") {
      out[col.name] = v == null ? "" : JSON.stringify(v, null, 2);
    } else {
      out[col.name] = v == null ? "" : String(v);
    }
  }
  return out;
}

/**
 * Convert form state into the JSON payload sent to the API. Empty non-bool
 * fields are omitted (so DB defaults / nulls apply); JSON fields are parsed.
 * Throws an Error (with the offending column) if a JSON field is invalid.
 */
export function buildPayload(
  spec: ResourceSpec,
  values: Record<string, any>,
): Record<string, any> {
  const payload: Record<string, any> = {};
  for (const col of editableColumns(spec)) {
    const raw = values[col.name];
    if (col.type === "bool") {
      payload[col.name] = !!raw;
      continue;
    }
    if (raw === undefined || raw === "") continue;
    if (col.type === "json") {
      try {
        payload[col.name] = JSON.parse(raw);
      } catch {
        throw new Error(`字段 "${col.name}" 不是合法 JSON`);
      }
    } else {
      payload[col.name] = raw;
    }
  }
  return payload;
}

interface FieldProps {
  col: ColumnSpec;
  value: any;
  onChange: (value: any) => void;
}

/** Renders one input appropriate to the column type. */
export function Field({ col, value, onChange }: FieldProps) {
  const label = (
    <label style={{ display: "block", fontWeight: 600, marginBottom: 4 }}>
      {col.name}
      <span style={{ color: "#9ca3af", fontWeight: 400 }}> · {col.type}</span>
    </label>
  );

  if (col.type === "bool") {
    return (
      <div style={{ marginBottom: 14 }}>
        <label style={{ fontWeight: 600 }}>
          <input
            type="checkbox"
            checked={!!value}
            onChange={(e) => onChange(e.target.checked)}
            style={{ marginRight: 8 }}
          />
          {col.name}
        </label>
      </div>
    );
  }

  if (col.type === "json") {
    return (
      <div style={{ marginBottom: 14 }}>
        {label}
        <textarea
          name={col.name}
          value={value ?? ""}
          onChange={(e) => onChange(e.target.value)}
          rows={4}
          style={{ width: "100%", fontFamily: "monospace", boxSizing: "border-box" }}
          placeholder="JSON, e.g. {}"
        />
      </div>
    );
  }

  return (
    <div style={{ marginBottom: 14 }}>
      {label}
      <input
        name={col.name}
        value={value ?? ""}
        onChange={(e) => onChange(e.target.value)}
        style={{ width: "100%", padding: "6px 8px", boxSizing: "border-box" }}
      />
    </div>
  );
}
