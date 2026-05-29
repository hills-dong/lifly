import type { DataProvider } from "@refinedev/core";

import { http } from "../http";

const BASE = "/api/admin/data";

/** Pull a 1-based page number out of Refine's pagination (v4/v5 compatible). */
function pageOf(pagination: any): number {
  return pagination?.currentPage ?? pagination?.current ?? 1;
}

/**
 * Custom data provider speaking the Lifly admin API's envelope
 * (`{ code, data, message }`) and the generic CRUD routes under `/api/admin/data`.
 */
export const dataProvider: DataProvider = {
  getList: async ({ resource, pagination, sorters, filters }) => {
    const params: Record<string, unknown> = {
      page: pageOf(pagination),
      per_page: pagination?.pageSize ?? 20,
    };

    const sorter = sorters?.[0];
    if (sorter) {
      params.sort = sorter.field;
      params.order = sorter.order;
    }

    (filters ?? []).forEach((f: any) => {
      if (f.field && f.value !== undefined && f.value !== "") {
        params[f.field] = f.value;
      }
    });

    const res = await http.get(`${BASE}/${resource}`, { params });
    const data = res.data?.data ?? {};
    return { data: data.items ?? [], total: data.total ?? 0 };
  },

  getOne: async ({ resource, id }) => {
    const res = await http.get(`${BASE}/${resource}/${id}`);
    return { data: res.data?.data };
  },

  create: async ({ resource, variables }) => {
    const res = await http.post(`${BASE}/${resource}`, variables);
    return { data: res.data?.data };
  },

  update: async ({ resource, id, variables }) => {
    const res = await http.put(`${BASE}/${resource}/${id}`, variables);
    return { data: res.data?.data };
  },

  deleteOne: async ({ resource, id }) => {
    const res = await http.delete(`${BASE}/${resource}/${id}`);
    return { data: res.data?.data };
  },

  getApiUrl: () => BASE,
};
