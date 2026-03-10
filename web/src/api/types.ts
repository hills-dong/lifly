// API response wrapper
export interface ApiResponse<T> {
  code: number;
  data: T;
  message: string;
}

// --- Domain Models ---

export interface User {
  id: string;
  username: string;
  email: string;
  display_name: string;
  created_at: string;
  updated_at: string;
}

export interface Device {
  id: string;
  user_id: string;
  name: string;
  type: string;
  token: string;
  last_seen_at: string;
  created_at: string;
}

export interface Tool {
  id: string;
  name: string;
  slug: string;
  description: string;
  icon: string;
  config: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface ToolVersion {
  id: string;
  tool_id: string;
  version: string;
  changelog: string;
  created_at: string;
}

export interface DataObject {
  id: string;
  tool_id: string;
  user_id: string;
  type: string;
  title: string;
  attributes: Record<string, unknown>;
  status: string;
  created_at: string;
  updated_at: string;
}

export interface FileStorage {
  id: string;
  data_object_id: string;
  filename: string;
  content_type: string;
  size: number;
  storage_path: string;
  created_at: string;
}

export interface Category {
  id: string;
  user_id: string;
  name: string;
  color: string;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface Reminder {
  id: string;
  user_id: string;
  data_object_id: string | null;
  title: string;
  description: string;
  trigger_at: string;
  repeat_rule: string;
  status: 'pending' | 'triggered' | 'dismissed';
  created_at: string;
  updated_at: string;
}

export interface Pipeline {
  id: string;
  name: string;
  tool_id: string;
  steps: PipelineStep[];
  created_at: string;
  updated_at: string;
}

export interface PipelineStep {
  name: string;
  type: string;
  config: Record<string, unknown>;
}

export interface PipelineRun {
  id: string;
  pipeline_id: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  input: Record<string, unknown>;
  output: Record<string, unknown>;
  error: string;
  started_at: string;
  completed_at: string;
  created_at: string;
}

// --- Request Types ---

export interface LoginRequest {
  username: string;
  password: string;
}

export interface LoginResponse {
  token: string;
  user: User;
}

export interface CreateRawInputRequest {
  tool_id: string;
  type: string;
  content: string;
  metadata?: Record<string, unknown>;
}

export interface CreateReminderRequest {
  title: string;
  description?: string;
  trigger_at: string;
  repeat_rule?: string;
  data_object_id?: string;
}

export interface UpdateReminderRequest {
  title?: string;
  description?: string;
  trigger_at?: string;
  repeat_rule?: string;
  status?: string;
}

export interface UpdateDataObjectRequest {
  title?: string;
  attributes?: Record<string, unknown>;
  status?: string;
}

export interface CreateCategoryRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateCategoryRequest {
  name?: string;
  color?: string;
  parent_id?: string;
}

export interface SearchRequest {
  query: string;
  tool_id?: string;
  type?: string;
  limit?: number;
  offset?: number;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}
