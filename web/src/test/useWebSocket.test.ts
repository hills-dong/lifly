import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useWebSocket, type WSMessage } from '../hooks/useWebSocket';

// --- Mock WebSocket ---

class MockWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;

  static instances: MockWebSocket[] = [];
  url: string;
  readyState: number = MockWebSocket.CONNECTING;
  onopen: ((ev: Event) => void) | null = null;
  onclose: ((ev: CloseEvent) => void) | null = null;
  onmessage: ((ev: MessageEvent) => void) | null = null;
  onerror: ((ev: Event) => void) | null = null;
  send = vi.fn();
  close = vi.fn();

  constructor(url: string) {
    this.url = url;
    MockWebSocket.instances.push(this);
  }

  /** Simulate server opening connection */
  simulateOpen() {
    this.readyState = MockWebSocket.OPEN;
    this.onopen?.(new Event('open'));
  }

  /** Simulate server sending a message */
  simulateMessage(data: string) {
    this.onmessage?.(new MessageEvent('message', { data }));
  }

  /** Simulate connection close */
  simulateClose() {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.({} as CloseEvent);
  }
}

describe('useWebSocket', () => {
  beforeEach(() => {
    MockWebSocket.instances = [];
    vi.stubGlobal('WebSocket', MockWebSocket);
    localStorage.clear();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it('connects to WebSocket with correct URL including token', () => {
    localStorage.setItem('token', 'test-jwt');

    renderHook(() => useWebSocket());

    expect(MockWebSocket.instances).toHaveLength(1);
    const ws = MockWebSocket.instances[0];
    expect(ws.url).toContain('/api/ws');
    expect(ws.url).toContain('token=test-jwt');
  });

  it('connects without token param when no token in localStorage', () => {
    renderHook(() => useWebSocket());

    const ws = MockWebSocket.instances[0];
    expect(ws.url).not.toContain('token=');
  });

  it('sets connected=true on open and connected=false on close', () => {
    const { result } = renderHook(() => useWebSocket());
    expect(result.current.connected).toBe(false);

    act(() => {
      MockWebSocket.instances[0].simulateOpen();
    });
    expect(result.current.connected).toBe(true);

    act(() => {
      MockWebSocket.instances[0].simulateClose();
    });
    expect(result.current.connected).toBe(false);
  });

  it('calls onMessage callback with parsed JSON message', () => {
    const handler = vi.fn();
    renderHook(() => useWebSocket(handler));

    const ws = MockWebSocket.instances[0];
    act(() => ws.simulateOpen());

    const msg: WSMessage = { type: 'pipeline_complete', payload: { id: '123' } };
    act(() => ws.simulateMessage(JSON.stringify(msg)));

    expect(handler).toHaveBeenCalledTimes(1);
    expect(handler).toHaveBeenCalledWith(msg);
  });

  it('ignores non-JSON messages without throwing', () => {
    const handler = vi.fn();
    renderHook(() => useWebSocket(handler));

    const ws = MockWebSocket.instances[0];
    act(() => ws.simulateOpen());
    act(() => ws.simulateMessage('not json'));

    expect(handler).not.toHaveBeenCalled();
  });

  it('sends messages when connected', () => {
    const { result } = renderHook(() => useWebSocket());
    const ws = MockWebSocket.instances[0];

    act(() => ws.simulateOpen());

    const msg: WSMessage = { type: 'ping', payload: {} };
    act(() => result.current.send(msg));

    expect(ws.send).toHaveBeenCalledWith(JSON.stringify(msg));
  });

  it('does not send when WebSocket is not open', () => {
    const { result } = renderHook(() => useWebSocket());
    const ws = MockWebSocket.instances[0];
    // readyState is CONNECTING, not OPEN

    const msg: WSMessage = { type: 'ping', payload: {} };
    act(() => result.current.send(msg));

    expect(ws.send).not.toHaveBeenCalled();
  });

  it('reconnects after connection close with 3s delay', () => {
    renderHook(() => useWebSocket());

    expect(MockWebSocket.instances).toHaveLength(1);

    act(() => {
      MockWebSocket.instances[0].simulateClose();
    });

    // Not yet reconnected
    expect(MockWebSocket.instances).toHaveLength(1);

    // Advance timer by 3 seconds
    act(() => {
      vi.advanceTimersByTime(3000);
    });

    expect(MockWebSocket.instances).toHaveLength(2);
  });
});
