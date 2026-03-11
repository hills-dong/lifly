import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, screen, act } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AuthProvider, useAuth } from '../contexts/AuthContext';

// Mock the API module
vi.mock('../api', () => ({
  auth: {
    login: vi.fn(),
    logout: vi.fn(),
  },
}));

import { auth } from '../api';

const mockedLogin = vi.mocked(auth.login);
const mockedLogout = vi.mocked(auth.logout);

/** Helper component that exposes AuthContext values for testing. */
function AuthConsumer({ onRender }: { onRender?: (ctx: ReturnType<typeof useAuth>) => void }) {
  const ctx = useAuth();
  onRender?.(ctx);
  return (
    <div>
      <span data-testid="authenticated">{String(ctx.isAuthenticated)}</span>
      <span data-testid="user">{ctx.user ? ctx.user.username : 'none'}</span>
      <span data-testid="token">{ctx.token ?? 'none'}</span>
      <button onClick={() => ctx.login({ username: 'admin', password: 'admin123' })}>Login</button>
      <button onClick={() => ctx.logout()}>Logout</button>
    </div>
  );
}

describe('AuthContext', () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });

  it('initial state is unauthenticated when localStorage is empty', () => {
    render(
      <AuthProvider>
        <AuthConsumer />
      </AuthProvider>,
    );
    expect(screen.getByTestId('authenticated')).toHaveTextContent('false');
    expect(screen.getByTestId('user')).toHaveTextContent('none');
    expect(screen.getByTestId('token')).toHaveTextContent('none');
  });

  it('reads initial state from localStorage', () => {
    localStorage.setItem('token', 'saved-token');
    localStorage.setItem('user', JSON.stringify({ username: 'alice', id: '1', email: '', display_name: '', created_at: '', updated_at: '' }));

    render(
      <AuthProvider>
        <AuthConsumer />
      </AuthProvider>,
    );
    expect(screen.getByTestId('authenticated')).toHaveTextContent('true');
    expect(screen.getByTestId('user')).toHaveTextContent('alice');
    expect(screen.getByTestId('token')).toHaveTextContent('saved-token');
  });

  it('login stores token and user in state and localStorage', async () => {
    const fakeUser = { id: '1', username: 'admin', email: 'a@b.c', display_name: 'Admin', created_at: '', updated_at: '' };
    mockedLogin.mockResolvedValueOnce({ token: 'jwt-123', user: fakeUser });

    render(
      <AuthProvider>
        <AuthConsumer />
      </AuthProvider>,
    );

    await act(async () => {
      await userEvent.click(screen.getByText('Login'));
    });

    expect(mockedLogin).toHaveBeenCalledWith({ username: 'admin', password: 'admin123' });
    expect(screen.getByTestId('authenticated')).toHaveTextContent('true');
    expect(screen.getByTestId('token')).toHaveTextContent('jwt-123');
    expect(screen.getByTestId('user')).toHaveTextContent('admin');
    expect(localStorage.getItem('token')).toBe('jwt-123');
    expect(JSON.parse(localStorage.getItem('user')!).username).toBe('admin');
  });

  it('logout clears token and user from state and localStorage', async () => {
    localStorage.setItem('token', 'old-token');
    localStorage.setItem('user', JSON.stringify({ username: 'admin', id: '1', email: '', display_name: '', created_at: '', updated_at: '' }));
    mockedLogout.mockResolvedValueOnce(undefined);

    render(
      <AuthProvider>
        <AuthConsumer />
      </AuthProvider>,
    );

    // Confirm initially authenticated
    expect(screen.getByTestId('authenticated')).toHaveTextContent('true');

    await act(async () => {
      await userEvent.click(screen.getByText('Logout'));
    });

    expect(screen.getByTestId('authenticated')).toHaveTextContent('false');
    expect(screen.getByTestId('user')).toHaveTextContent('none');
    expect(localStorage.getItem('token')).toBeNull();
    expect(localStorage.getItem('user')).toBeNull();
  });

  it('useAuth throws when used outside AuthProvider', () => {
    // Suppress React error boundary console output
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {});
    expect(() => render(<AuthConsumer />)).toThrow('useAuth must be used within an AuthProvider');
    spy.mockRestore();
  });
});
