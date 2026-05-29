/// <reference types="node" />
import { describe, it, expect } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import GrowthView from '../embed/GrowthView';
import { WHO_STANDARDS } from '../embed/growthStandards';
import type { DataObject } from '../api/types';

// Load the records straight from the seed migration so the test exercises the
// exact data that ships to production.
function loadSeededRecords(): DataObject[] {
  const path = resolve(process.cwd(), '../server/migrations/20260311000005_seed_growth_record.sql');
  const sql = readFileSync(path, 'utf8');
  const re = /\('([0-9a-f-]+)', '00000000-0000-0000-0000-000000000203', '(\{.*?\})'::jsonb/g;
  const items: DataObject[] = [];
  for (const m of sql.matchAll(re)) {
    items.push({
      id: m[1],
      tool_id: '00000000-0000-0000-0000-000000000203',
      attributes: JSON.parse(m[2]),
      status: 'active',
      created_at: '',
      updated_at: '',
    });
  }
  return items;
}

const records = loadSeededRecords();

describe('WHO_STANDARDS data module', () => {
  it('covers months 0–60 for both sexes and metrics', () => {
    for (const sex of ['male', 'female'] as const) {
      for (const metric of ['height', 'weight'] as const) {
        const band = WHO_STANDARDS[sex][metric];
        expect(band[0].m).toBe(0);
        expect(band[band.length - 1].m).toBe(60);
        expect(band.length).toBe(61);
        // percentiles are ordered p3 < p50 < p97 at every age
        for (const b of band) {
          expect(b.p3).toBeLessThan(b.p50);
          expect(b.p50).toBeLessThan(b.p97);
        }
      }
    }
  });
});

describe('GrowthView with seeded records', () => {
  it('parses all 41 records from the migration', () => {
    expect(records.length).toBe(41);
  });

  it('shows the latest measurement in the summary', () => {
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    const summary = document.querySelector('.growth-summary') as HTMLElement;
    expect(summary).toBeTruthy();
    expect(within(summary).getByText('115')).toBeInTheDocument();
    expect(within(summary).getByText('17')).toBeInTheDocument();
    expect(within(summary).getByText('5岁8月6天')).toBeInTheDocument();
  });

  it('renders every record as a list row, newest first', () => {
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    const rows = document.querySelectorAll('.growth-row');
    expect(rows.length).toBe(41);
    // First row is the newest date.
    expect(within(rows[0] as HTMLElement).getByText('2025-11-19')).toBeInTheDocument();
    // Last row is the earliest.
    expect(within(rows[40] as HTMLElement).getByText('2020-04-03')).toBeInTheDocument();
  });

  it('shows an em-dash for a record missing height', () => {
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    // 2022-03-22 has only weight (10.8), no height.
    const rows = Array.from(document.querySelectorAll('.growth-row')) as HTMLElement[];
    const row = rows.find((r) => within(r).queryByText('2022-03-22'));
    expect(row).toBeTruthy();
    const heightCell = row!.querySelectorAll('.growth-metric')[0];
    expect(heightCell.querySelector('.growth-metric-val')?.textContent).toBe('—');
  });

  it('classifies measurements against WHO bands (chips render)', () => {
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    // At least some normal/low chips should appear given this child trends low.
    expect(document.querySelectorAll('.growth-chip').length).toBeGreaterThan(0);
  });

  it('renders the height curve with percentile bands and the child line', async () => {
    const user = userEvent.setup();
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    await user.click(screen.getByRole('button', { name: '身高曲线' }));

    const svg = document.querySelector('svg.growth-chart');
    expect(svg).toBeTruthy();
    // 5 percentile lines (P3/P15/P50/P85/P97).
    expect(svg!.querySelectorAll('.growth-pline').length).toBe(5);
    // The child's connected curve.
    expect(svg!.querySelector('.growth-childline')).toBeTruthy();
    // One dot per height measurement (28 records have a height value).
    const heightCount = records.filter((r) => r.attributes.height_cm != null).length;
    expect(svg!.querySelectorAll('.growth-dot').length).toBe(heightCount);
  });

  it('switches to the weight curve and honors the sex toggle', async () => {
    const user = userEvent.setup();
    render(<GrowthView items={records} toolId="00000000-0000-0000-0000-000000000203" onChanged={() => {}} />);
    await user.click(screen.getByRole('button', { name: '体重曲线' }));
    expect(document.querySelector('svg.growth-chart')).toBeTruthy();

    // Toggle to girl — chart should still render.
    await user.click(screen.getByRole('button', { name: '女孩' }));
    const svg = document.querySelector('svg.growth-chart');
    expect(svg).toBeTruthy();
    const weightCount = records.filter((r) => r.attributes.weight_kg != null).length;
    expect(svg!.querySelectorAll('.growth-dot').length).toBe(weightCount);
  });
});
