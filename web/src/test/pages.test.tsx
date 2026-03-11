import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';

// Mock the API module
vi.mock('../api', () => ({
  auth: { login: vi.fn(), logout: vi.fn() },
  tools: { listTools: vi.fn(), getTool: vi.fn() },
  dataObjects: {
    listDataObjects: vi.fn(),
    getDataObject: vi.fn(),
    updateDataObject: vi.fn(),
    deleteDataObject: vi.fn(),
    searchDataObjects: vi.fn(),
  },
  rawInputs: { createRawInput: vi.fn() },
  pipelines: { listPipelines: vi.fn(), getPipeline: vi.fn() },
  reminders: {
    listReminders: vi.fn(),
    createReminder: vi.fn(),
    updateReminder: vi.fn(),
    deleteReminder: vi.fn(),
    dismissReminder: vi.fn(),
  },
  categories: {
    listCategories: vi.fn(),
    createCategory: vi.fn(),
    updateCategory: vi.fn(),
    deleteCategory: vi.fn(),
  },
  files: { uploadFile: vi.fn(), getFileUrl: vi.fn((id: string) => `/api/files/${id}`) },
}));

// Mock useWebSocket to prevent actual WebSocket connections
vi.mock('../hooks/useWebSocket', () => ({
  useWebSocket: vi.fn(() => ({ connected: false, send: vi.fn() })),
}));

import { tools as toolsApi } from '../api';
import { AuthProvider } from '../contexts/AuthContext';

const mockedListTools = vi.mocked(toolsApi.listTools);

describe('LoginPage', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('renders login form with username, password, and submit button', async () => {
    const { default: LoginPage } = await import('../pages/LoginPage');

    render(
      <MemoryRouter>
        <AuthProvider>
          <LoginPage />
        </AuthProvider>
      </MemoryRouter>,
    );

    expect(screen.getByLabelText(/username/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /sign in/i })).toBeInTheDocument();
    expect(screen.getByText('Lifly')).toBeInTheDocument();
  });
});

describe('HomePage', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it('renders tool list when tools are loaded', async () => {
    mockedListTools.mockResolvedValueOnce([
      {
        id: 't1',
        name: 'Todo List',
        slug: 'todo-list',
        description: 'Manage your tasks',
        icon: '',
        config: {},
        created_at: '2026-01-01T00:00:00Z',
        updated_at: '2026-01-01T00:00:00Z',
      },
    ]);

    const { default: HomePage } = await import('../pages/HomePage');

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(await screen.findByText('Todo List')).toBeInTheDocument();
    expect(screen.getByText('Manage your tasks')).toBeInTheDocument();
  });

  it('renders empty state when no tools exist', async () => {
    mockedListTools.mockResolvedValueOnce([]);

    const { default: HomePage } = await import('../pages/HomePage');

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(await screen.findByText(/no tools available/i)).toBeInTheDocument();
  });
});

describe('SearchPage', () => {
  it('renders search input and button', async () => {
    const { default: SearchPage } = await import('../pages/SearchPage');

    render(
      <MemoryRouter>
        <SearchPage />
      </MemoryRouter>,
    );

    expect(screen.getByPlaceholderText(/search data objects/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /search/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Search' })).toBeInTheDocument();
  });
});
