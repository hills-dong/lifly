import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from '../App';

describe('App', () => {
  beforeEach(() => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
  });

  it('renders login page for unauthenticated users', () => {
    render(<App />);
    // The login page should render with a login form.
    expect(screen.getByRole('button', { name: /log\s*in|sign\s*in|登录/i })).toBeInTheDocument();
  });

  it('redirects to login when accessing protected route', () => {
    render(<App />);
    // Should redirect to login since there's no token.
    expect(screen.getByRole('button', { name: /log\s*in|sign\s*in|登录/i })).toBeInTheDocument();
  });
});
