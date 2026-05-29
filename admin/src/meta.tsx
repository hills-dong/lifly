import { createContext, useContext, useEffect, useState } from "react";

import { http } from "./http";

export type ColType =
  | "uuid"
  | "text"
  | "int"
  | "big_int"
  | "bool"
  | "json"
  | "timestamp"
  | "vector";

export interface ColumnSpec {
  name: string;
  type: ColType;
  readonly: boolean;
  hidden: boolean;
}

export interface ResourceSpec {
  name: string;
  pk: string;
  columns: ColumnSpec[];
}

interface MetaState {
  resources: Record<string, ResourceSpec>;
  list: ResourceSpec[];
  loading: boolean;
}

const MetaContext = createContext<MetaState>({
  resources: {},
  list: [],
  loading: true,
});

export const useMeta = () => useContext(MetaContext);

/** Convenience helpers over a resource spec. */
export const visibleColumns = (r: ResourceSpec) => r.columns.filter((c) => !c.hidden);
export const editableColumns = (r: ResourceSpec) =>
  r.columns.filter((c) => !c.hidden && !c.readonly);

/**
 * Loads the resource registry from `/api/admin/meta` once (after auth) and
 * exposes it to the generic pages. Renders a splash until it resolves.
 */
export function MetaProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<MetaState>({
    resources: {},
    list: [],
    loading: true,
  });

  useEffect(() => {
    http
      .get("/api/admin/meta")
      .then((res) => {
        const list: ResourceSpec[] = res.data?.data?.resources ?? [];
        const resources: Record<string, ResourceSpec> = {};
        list.forEach((r) => {
          resources[r.name] = r;
        });
        setState({ resources, list, loading: false });
      })
      .catch(() => setState((s) => ({ ...s, loading: false })));
  }, []);

  if (state.loading) {
    return <div style={{ padding: 24, fontFamily: "system-ui" }}>加载数据结构…</div>;
  }

  return <MetaContext.Provider value={state}>{children}</MetaContext.Provider>;
}
