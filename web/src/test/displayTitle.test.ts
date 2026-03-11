import { describe, it, expect } from 'vitest';
import { displayTitle } from '../utils/displayTitle';
import type { DataObject } from '../api/types';

/** Helper to create a minimal DataObject with given attributes. */
function makeObj(attributes: Record<string, unknown>): DataObject {
  return {
    id: 'test-id',
    tool_id: 'tool-1',
    attributes,
    status: 'active',
    created_at: '2026-01-01T00:00:00Z',
    updated_at: '2026-01-01T00:00:00Z',
  };
}

describe('displayTitle', () => {
  it('returns content field when available', () => {
    const obj = makeObj({ content: 'Buy groceries' });
    expect(displayTitle(obj)).toBe('Buy groceries');
  });

  it('returns title when content is missing', () => {
    const obj = makeObj({ title: 'My Document' });
    expect(displayTitle(obj)).toBe('My Document');
  });

  it('returns full_name when content and title are missing', () => {
    const obj = makeObj({ full_name: 'John Doe' });
    expect(displayTitle(obj)).toBe('John Doe');
  });

  it('returns cert_type as last candidate', () => {
    const obj = makeObj({ cert_type: 'Passport' });
    expect(displayTitle(obj)).toBe('Passport');
  });

  it('prefers content over title (priority order)', () => {
    const obj = makeObj({ content: 'Primary', title: 'Secondary' });
    expect(displayTitle(obj)).toBe('Primary');
  });

  it('returns Untitled when no candidates are present', () => {
    const obj = makeObj({});
    expect(displayTitle(obj)).toBe('Untitled');
  });

  it('returns Untitled when all candidates are null', () => {
    const obj = makeObj({ content: null, title: null });
    expect(displayTitle(obj)).toBe('Untitled');
  });

  it('returns Untitled when all candidates are empty strings', () => {
    const obj = makeObj({ content: '', title: '' });
    expect(displayTitle(obj)).toBe('Untitled');
  });

  it('skips strings longer than 200 chars (base64 detection)', () => {
    const longString = 'A'.repeat(201);
    const obj = makeObj({ content: longString, title: 'Fallback Title' });
    expect(displayTitle(obj)).toBe('Fallback Title');
  });

  it('returns Untitled when all candidates exceed 200 chars', () => {
    const longString = 'B'.repeat(250);
    const obj = makeObj({ content: longString, title: longString });
    expect(displayTitle(obj)).toBe('Untitled');
  });

  it('converts non-string values to string', () => {
    const obj = makeObj({ content: 42 });
    expect(displayTitle(obj)).toBe('42');
  });

  it('accepts exactly 200-char strings', () => {
    const exact200 = 'C'.repeat(200);
    const obj = makeObj({ content: exact200 });
    expect(displayTitle(obj)).toBe(exact200);
  });
});
