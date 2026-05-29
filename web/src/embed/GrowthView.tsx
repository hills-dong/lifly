import { useMemo, useState } from 'react';
import type { DataObject } from '../api/types';
import { dataObjects as doApi } from '../api';
import { WHO_STANDARDS } from './growthStandards';
import type { GrowthBand, Metric, Sex } from './growthStandards';
import './embed.css';

const DAY = 86_400_000;
const MS_PER_MONTH = 30.4375 * DAY;

/** Derive the child's birth time (ms) from existing records: birth = date − age_months. */
function deriveBirthMs(items: DataObject[]): number | null {
  const births: number[] = [];
  for (const o of items) {
    const d = typeof o.attributes?.date === 'string' ? Date.parse(o.attributes.date as string) : NaN;
    const am = typeof o.attributes?.age_months === 'number' ? (o.attributes.age_months as number) : NaN;
    if (!Number.isNaN(d) && !Number.isNaN(am)) births.push(d - am * MS_PER_MONTH);
  }
  if (!births.length) return null;
  births.sort((a, b) => a - b);
  return births[Math.floor(births.length / 2)];
}

/** Human age label (5岁8月6天 / 3月25天 / 22天) from birth → measurement date. */
function ageLabelFrom(birthMs: number, dateMs: number): string {
  const b = new Date(birthMs);
  const d = new Date(dateMs);
  let years = d.getFullYear() - b.getFullYear();
  let months = d.getMonth() - b.getMonth();
  let days = d.getDate() - b.getDate();
  if (days < 0) {
    days += new Date(d.getFullYear(), d.getMonth(), 0).getDate();
    months -= 1;
  }
  if (months < 0) {
    months += 12;
    years -= 1;
  }
  if (years > 0) return `${years}岁${months}月${days}天`;
  if (months > 0) return `${months}月${days}天`;
  return `${days}天`;
}

function num(obj: DataObject, key: string): number | undefined {
  const v = obj.attributes?.[key];
  return typeof v === 'number' ? v : v != null && v !== '' ? Number(v) : undefined;
}
function str(obj: DataObject, key: string): string | undefined {
  const v = obj.attributes?.[key];
  return v == null ? undefined : String(v);
}

const SEX_KEY = 'lifly.growth.sex';
const PCTS = ['p97', 'p85', 'p50', 'p15', 'p3'] as const;

/** Linear-interpolate the WHO band values at an arbitrary age (months). Null past the table. */
function bandAt(bands: GrowthBand[], age: number): GrowthBand | null {
  if (age < 0 || age > bands[bands.length - 1].m) return null;
  const hi = bands.findIndex((b) => b.m >= age);
  if (hi < 0) return null;
  if (bands[hi].m === age || hi === 0) return bands[hi];
  const a = bands[hi - 1];
  const b = bands[hi];
  const t = (age - a.m) / (b.m - a.m);
  const lerp = (x: number, y: number) => x + (y - x) * t;
  return {
    m: age,
    p3: lerp(a.p3, b.p3),
    p15: lerp(a.p15, b.p15),
    p50: lerp(a.p50, b.p50),
    p85: lerp(a.p85, b.p85),
    p97: lerp(a.p97, b.p97),
  };
}

/** Classify a measurement against the WHO percentile bands shown on the chart. */
function classify(value: number, band: GrowthBand | null): { label: string; cls: string } | null {
  if (!band) return null;
  if (value < band.p3) return { label: '过低', cls: 'vlow' };
  if (value < band.p15) return { label: '偏低', cls: 'low' };
  if (value <= band.p85) return { label: '正常', cls: 'normal' };
  if (value <= band.p97) return { label: '偏高', cls: 'high' };
  return { label: '过高', cls: 'vhigh' };
}

interface ChartPoint {
  age: number;
  value: number;
}

function GrowthChart({
  metric,
  sex,
  points,
}: {
  metric: Metric;
  sex: Sex;
  points: ChartPoint[];
}) {
  const bands = WHO_STANDARDS[sex][metric];
  const W = 360;
  const H = 248;
  const padL = 30;
  const padR = 12;
  const padT = 10;
  const padB = 24;
  const plotW = W - padL - padR;
  const plotH = H - padT - padB;

  const dataMaxAge = points.length ? Math.max(...points.map((p) => p.age)) : 0;
  const xMax = Math.max(12, Math.ceil(Math.max(60, dataMaxAge) / 12) * 12);

  const yVals: number[] = [];
  for (const b of bands) {
    yVals.push(b.p3, b.p97);
  }
  for (const p of points) yVals.push(p.value);
  let yMin = Math.min(...yVals);
  let yMax = Math.max(...yVals);
  const pad = (yMax - yMin) * 0.06 || 1;
  yMin = Math.floor((yMin - pad) / (metric === 'weight' ? 1 : 5)) * (metric === 'weight' ? 1 : 5);
  yMax = Math.ceil((yMax + pad) / (metric === 'weight' ? 1 : 5)) * (metric === 'weight' ? 1 : 5);

  const x = (age: number) => padL + (age / xMax) * plotW;
  const y = (v: number) => padT + (1 - (v - yMin) / (yMax - yMin)) * plotH;

  const linePath = (key: keyof GrowthBand) =>
    bands.map((b, i) => `${i === 0 ? 'M' : 'L'}${x(b.m).toFixed(1)},${y(b[key] as number).toFixed(1)}`).join(' ');

  const areaPath = (top: keyof GrowthBand, bot: keyof GrowthBand) => {
    const t = bands.map((b) => `${x(b.m).toFixed(1)},${y(b[top] as number).toFixed(1)}`);
    const d = [...bands].reverse().map((b) => `${x(b.m).toFixed(1)},${y(b[bot] as number).toFixed(1)}`);
    return `M${t.join(' L')} L${d.join(' L')} Z`;
  };

  const sorted = [...points].sort((a, b) => a.age - b.age);
  const childPath = sorted
    .map((p, i) => `${i === 0 ? 'M' : 'L'}${x(p.age).toFixed(1)},${y(p.value).toFixed(1)}`)
    .join(' ');

  // X ticks every 12 months (years); Y ticks ~5 divisions.
  const xticks: number[] = [];
  for (let m = 0; m <= xMax; m += 12) xticks.push(m);
  const yStep = metric === 'weight' ? Math.max(2, Math.round((yMax - yMin) / 5)) : 10;
  const yticks: number[] = [];
  for (let v = yMin; v <= yMax + 0.001; v += yStep) yticks.push(v);

  const unit = metric === 'weight' ? 'kg' : 'cm';

  return (
    <svg className="growth-chart" viewBox={`0 0 ${W} ${H}`} preserveAspectRatio="xMidYMid meet" role="img">
      {/* gridlines */}
      {yticks.map((v) => (
        <line key={`gy${v}`} x1={padL} y1={y(v)} x2={W - padR} y2={y(v)} className="growth-grid" />
      ))}
      {/* percentile band shading */}
      <path d={areaPath('p97', 'p3')} className="growth-band-outer" />
      <path d={areaPath('p85', 'p15')} className="growth-band-inner" />
      {/* percentile lines */}
      {PCTS.map((k) => (
        <path key={k} d={linePath(k)} className={`growth-pline ${k === 'p50' ? 'p50' : ''}`} />
      ))}
      {/* percentile labels at right edge */}
      {PCTS.map((k) => {
        const last = bands[bands.length - 1];
        return (
          <text key={`lbl${k}`} x={x(last.m) + 2} y={y(last[k] as number) + 3} className="growth-pclabel">
            {k.slice(1)}
          </text>
        );
      })}
      {/* child curve */}
      {sorted.length > 1 && <path d={childPath} className="growth-childline" />}
      {sorted.map((p, i) => (
        <circle key={i} cx={x(p.age)} cy={y(p.value)} r={2.6} className="growth-dot" />
      ))}
      {/* axes */}
      <line x1={padL} y1={padT} x2={padL} y2={padT + plotH} className="growth-axis" />
      <line x1={padL} y1={padT + plotH} x2={W - padR} y2={padT + plotH} className="growth-axis" />
      {xticks.map((m) => (
        <text key={`xt${m}`} x={x(m)} y={H - 8} className="growth-tick" textAnchor="middle">
          {m === 0 ? '0' : `${m / 12}岁`}
        </text>
      ))}
      {yticks.map((v) => (
        <text key={`yt${v}`} x={padL - 4} y={y(v) + 3} className="growth-tick" textAnchor="end">
          {v}
        </text>
      ))}
      <text x={padL} y={padT - 1} className="growth-unit">
        {unit}
      </text>
    </svg>
  );
}

type Tab = 'list' | 'height' | 'weight';

export default function GrowthView({
  items,
  toolId,
  onChanged,
}: {
  items: DataObject[];
  toolId: string;
  onChanged: () => void | Promise<void>;
}) {
  const [tab, setTab] = useState<Tab>('list');
  const [sex, setSex] = useState<Sex>(
    () => (typeof localStorage !== 'undefined' && (localStorage.getItem(SEX_KEY) as Sex)) || 'male',
  );

  const setSexPersist = (s: Sex) => {
    setSex(s);
    try {
      localStorage.setItem(SEX_KEY, s);
    } catch {
      /* ignore */
    }
  };

  // Add-record form state.
  const today = new Date().toISOString().slice(0, 10);
  const [adding, setAdding] = useState(false);
  const [busy, setBusy] = useState(false);
  const [formErr, setFormErr] = useState('');
  const [fDate, setFDate] = useState(today);
  const [fHeight, setFHeight] = useState('');
  const [fWeight, setFWeight] = useState('');
  const [fBirth, setFBirth] = useState('');

  const birthMs = useMemo(() => deriveBirthMs(items), [items]);

  const submitAdd = async () => {
    setFormErr('');
    const dateMs = Date.parse(fDate);
    if (Number.isNaN(dateMs)) return setFormErr('请填写有效日期');
    const h = fHeight.trim() ? Number(fHeight) : undefined;
    const w = fWeight.trim() ? Number(fWeight) : undefined;
    if (h == null && w == null) return setFormErr('请至少填写身高或体重');
    if ((h != null && (Number.isNaN(h) || h <= 0)) || (w != null && (Number.isNaN(w) || w <= 0)))
      return setFormErr('身高/体重需为正数');
    let birth = birthMs;
    if (birth == null) {
      const bMs = Date.parse(fBirth);
      if (Number.isNaN(bMs)) return setFormErr('首次添加请填写宝宝出生日期');
      birth = bMs;
    }
    const ageMonths = Math.round(((dateMs - birth) / MS_PER_MONTH) * 100) / 100;
    if (ageMonths < 0) return setFormErr('测量日期早于出生日期');
    const attributes: Record<string, unknown> = {
      date: fDate,
      age_months: ageMonths,
      age_label: ageLabelFrom(birth, dateMs),
    };
    if (h != null) attributes.height_cm = h;
    if (w != null) attributes.weight_kg = w;
    setBusy(true);
    try {
      await doApi.createDataObject({ tool_id: toolId, attributes });
      setFHeight('');
      setFWeight('');
      setFDate(today);
      setAdding(false);
      await onChanged();
    } catch (e) {
      setFormErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const removeRecord = async (id: string) => {
    if (typeof window !== 'undefined' && !window.confirm('删除这条记录？')) return;
    try {
      await doApi.deleteDataObject(id);
      await onChanged();
    } catch (e) {
      setFormErr(e instanceof Error ? e.message : String(e));
    }
  };

  // Records sorted newest-first for the list.
  const records = useMemo(
    () =>
      [...items].sort((a, b) => (num(b, 'age_months') ?? 0) - (num(a, 'age_months') ?? 0)),
    [items],
  );

  const heightPoints = useMemo(
    () =>
      items
        .map((o) => ({ age: num(o, 'age_months'), value: num(o, 'height_cm') }))
        .filter((p): p is ChartPoint => p.age != null && p.value != null),
    [items],
  );
  const weightPoints = useMemo(
    () =>
      items
        .map((o) => ({ age: num(o, 'age_months'), value: num(o, 'weight_kg') }))
        .filter((p): p is ChartPoint => p.age != null && p.value != null),
    [items],
  );

  const latest = records[0];

  return (
    <div className="growth">
      {latest && (
        <div className="growth-summary">
          <div className="growth-sum-cell">
            <span className="growth-sum-val">{num(latest, 'height_cm') ?? '—'}</span>
            <span className="growth-sum-unit">cm</span>
            <span className="growth-sum-lbl">身高</span>
          </div>
          <div className="growth-sum-cell">
            <span className="growth-sum-val">{num(latest, 'weight_kg') ?? '—'}</span>
            <span className="growth-sum-unit">kg</span>
            <span className="growth-sum-lbl">体重</span>
          </div>
          <div className="growth-sum-cell">
            <span className="growth-sum-age">{str(latest, 'age_label') ?? ''}</span>
            <span className="growth-sum-lbl">{str(latest, 'date')}</span>
          </div>
        </div>
      )}

      <div className="growth-addbar">
        <button className="growth-addbtn" onClick={() => { setAdding((v) => !v); setFormErr(''); }}>
          {adding ? '取消' : '＋ 添加记录'}
        </button>
      </div>
      {adding && (
        <div className="growth-form">
          {formErr && <div className="growth-formerr">{formErr}</div>}
          <label className="growth-field">
            <span>日期</span>
            <input type="date" value={fDate} max={today} onChange={(e) => setFDate(e.target.value)} />
          </label>
          {birthMs == null && (
            <label className="growth-field">
              <span>出生日期</span>
              <input type="date" value={fBirth} max={today} onChange={(e) => setFBirth(e.target.value)} />
            </label>
          )}
          <label className="growth-field">
            <span>身高 cm</span>
            <input type="number" inputMode="decimal" step="0.1" value={fHeight} placeholder="选填" onChange={(e) => setFHeight(e.target.value)} />
          </label>
          <label className="growth-field">
            <span>体重 kg</span>
            <input type="number" inputMode="decimal" step="0.1" value={fWeight} placeholder="选填" onChange={(e) => setFWeight(e.target.value)} />
          </label>
          <button className="growth-savebtn" onClick={submitAdd} disabled={busy}>
            {busy ? '保存中…' : '保存'}
          </button>
        </div>
      )}

      <div className="growth-tabs">
        <button className={tab === 'list' ? 'on' : ''} onClick={() => setTab('list')}>
          记录列表
        </button>
        <button className={tab === 'height' ? 'on' : ''} onClick={() => setTab('height')}>
          身高曲线
        </button>
        <button className={tab === 'weight' ? 'on' : ''} onClick={() => setTab('weight')}>
          体重曲线
        </button>
      </div>

      {tab !== 'list' && (
        <>
          <div className="growth-sexrow">
            <span className="growth-sex-hint">百分位参照</span>
            <div className="growth-sextoggle">
              <button className={sex === 'male' ? 'on' : ''} onClick={() => setSexPersist('male')}>
                男孩
              </button>
              <button className={sex === 'female' ? 'on' : ''} onClick={() => setSexPersist('female')}>
                女孩
              </button>
            </div>
          </div>
          <GrowthChart
            metric={tab === 'height' ? 'height' : 'weight'}
            sex={sex}
            points={tab === 'height' ? heightPoints : weightPoints}
          />
          <p className="growth-note">阴影为 WHO 0–5 岁生长标准 P3–P97 区间，中线为 P50（中位数）。</p>
        </>
      )}

      {tab === 'list' && (
        <ul className="growth-list">
          {records.map((o) => {
            const h = num(o, 'height_cm');
            const w = num(o, 'weight_kg');
            const age = num(o, 'age_months');
            const hStatus = h != null && age != null ? classify(h, bandAt(WHO_STANDARDS[sex].height, age)) : null;
            const wStatus = w != null && age != null ? classify(w, bandAt(WHO_STANDARDS[sex].weight, age)) : null;
            return (
              <li key={o.id} className="growth-row">
                <div className="growth-row-head">
                  <span className="growth-row-date">{str(o, 'date')}</span>
                  <span className="growth-row-age">{str(o, 'age_label')}</span>
                  <button className="growth-rowdel" onClick={() => removeRecord(o.id)} aria-label="删除">
                    ✕
                  </button>
                </div>
                <div className="growth-row-vals">
                  <div className="growth-metric">
                    <span className="growth-metric-val">{h ?? '—'}</span>
                    <span className="growth-metric-unit">{h != null ? 'cm' : ''}</span>
                    <span className="growth-metric-lbl">身高</span>
                    {hStatus && <span className={`growth-chip ${hStatus.cls}`}>{hStatus.label}</span>}
                  </div>
                  <div className="growth-metric">
                    <span className="growth-metric-val">{w ?? '—'}</span>
                    <span className="growth-metric-unit">{w != null ? 'kg' : ''}</span>
                    <span className="growth-metric-lbl">体重</span>
                    {wStatus && <span className={`growth-chip ${wStatus.cls}`}>{wStatus.label}</span>}
                  </div>
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
