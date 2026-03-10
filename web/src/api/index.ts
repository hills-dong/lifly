import client from './client';
import type {
  LoginRequest,
  LoginResponse,
  Tool,
  DataObject,
  UpdateDataObjectRequest,
  CreateRawInputRequest,
  Pipeline,
  PipelineRun,
  Reminder,
  CreateReminderRequest,
  UpdateReminderRequest,
  Category,
  CreateCategoryRequest,
  UpdateCategoryRequest,
  FileStorage,
  PaginatedResponse,
} from './types';

// --- Auth ---

export const auth = {
  login: (data: LoginRequest) =>
    client.post<LoginResponse>('/api/auth/login', data).then((r) => r.data),

  logout: () => client.post('/api/auth/logout').then((r) => r.data),
};

// --- Tools ---

export const tools = {
  listTools: () =>
    client.get<Tool[]>('/api/tools').then((r) => r.data),

  getTool: (id: string) =>
    client.get<Tool>(`/api/tools/${id}`).then((r) => r.data),
};

// --- Data Objects ---

export const dataObjects = {
  listDataObjects: (params?: { tool_id?: string; type?: string; status?: string; limit?: number; offset?: number }) =>
    client.get<PaginatedResponse<DataObject>>('/api/data-objects', { params }).then((r) => r.data),

  getDataObject: (id: string) =>
    client.get<DataObject>(`/api/data-objects/${id}`).then((r) => r.data),

  updateDataObject: (id: string, data: UpdateDataObjectRequest) =>
    client.put<DataObject>(`/api/data-objects/${id}`, data).then((r) => r.data),

  deleteDataObject: (id: string) =>
    client.delete(`/api/data-objects/${id}`).then((r) => r.data),

  searchDataObjects: (params: { query: string; tool_id?: string; type?: string; limit?: number; offset?: number }) =>
    client.get<PaginatedResponse<DataObject>>('/api/data-objects/search', { params }).then((r) => r.data),
};

// --- Raw Inputs ---

export const rawInputs = {
  createRawInput: (data: CreateRawInputRequest) =>
    client.post<PipelineRun>('/api/raw-inputs', data).then((r) => r.data),
};

// --- Pipelines ---

export const pipelines = {
  listPipelines: (params?: { tool_id?: string }) =>
    client.get<Pipeline[]>('/api/pipelines', { params }).then((r) => r.data),

  getPipeline: (id: string) =>
    client.get<Pipeline>(`/api/pipelines/${id}`).then((r) => r.data),
};

// --- Reminders ---

export const reminders = {
  listReminders: (params?: { status?: string }) =>
    client.get<Reminder[]>('/api/reminders', { params }).then((r) => r.data),

  createReminder: (data: CreateReminderRequest) =>
    client.post<Reminder>('/api/reminders', data).then((r) => r.data),

  updateReminder: (id: string, data: UpdateReminderRequest) =>
    client.put<Reminder>(`/api/reminders/${id}`, data).then((r) => r.data),

  deleteReminder: (id: string) =>
    client.delete(`/api/reminders/${id}`).then((r) => r.data),

  dismissReminder: (id: string) =>
    client.post(`/api/reminders/${id}/dismiss`).then((r) => r.data),
};

// --- Categories ---

export const categories = {
  listCategories: () =>
    client.get<Category[]>('/api/categories').then((r) => r.data),

  createCategory: (data: CreateCategoryRequest) =>
    client.post<Category>('/api/categories', data).then((r) => r.data),

  updateCategory: (id: string, data: UpdateCategoryRequest) =>
    client.put<Category>(`/api/categories/${id}`, data).then((r) => r.data),

  deleteCategory: (id: string) =>
    client.delete(`/api/categories/${id}`).then((r) => r.data),
};

// --- Files ---

export const files = {
  uploadFile: (dataObjectId: string, file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    return client
      .post<FileStorage>(`/api/data-objects/${dataObjectId}/files`, formData, {
        headers: { 'Content-Type': 'multipart/form-data' },
      })
      .then((r) => r.data);
  },

  getFileUrl: (fileId: string) => `/api/files/${fileId}/download`,
};
