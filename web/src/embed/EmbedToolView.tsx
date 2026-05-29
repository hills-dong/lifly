import { useCallback, useEffect, useState } from 'react';
import { tools as toolsApi, dataObjects as doApi, rawInputs, pipelines, files as filesApi } from '../api';
import type { DataObject, Tool, FileStorage } from '../api/types';
import { displayTitle } from '../utils/displayTitle';
import { captureImages, fileURL, getContext, isNative, setTitle } from './lifly';
import GrowthView from './GrowthView';
import './embed.css';

type Kind = 'todo' | 'doc' | 'growth' | 'generic';

function kindOf(tool: Tool | null): Kind {
  if (!tool) return 'generic';
  const s = (tool.name + ' ' + (tool.description ?? '')).toLowerCase();
  if (s.includes('todo') || s.includes('待办')) return 'todo';
  if (s.includes('证件') || s.includes('document') || s.includes('id-doc')) return 'doc';
  if (s.includes('成长') || s.includes('growth')) return 'growth';
  return 'generic';
}

function attr(obj: DataObject, key: string): string | undefined {
  const v = obj.attributes?.[key];
  if (v == null) return undefined;
  const s = String(v);
  return s.length ? s : undefined;
}

function bestImage(list: FileStorage[]): FileStorage | undefined {
  const imgs = list.filter((f) => f.mime_type.startsWith('image/'));
  return imgs.find((f) => f.role === 'processed') || imgs.find((f) => f.role === 'original') || imgs[0];
}

async function waitForPipeline(pipelineId: string) {
  for (let i = 0; i < 30; i++) {
    try {
      const p = (await pipelines.getPipeline(pipelineId)) as unknown as { status?: string };
      if (p.status === 'completed' || p.status === 'failed') return;
    } catch {
      /* keep polling */
    }
    await new Promise((r) => setTimeout(r, 1500));
  }
}

export default function EmbedToolView({ toolId }: { toolId: string }) {
  const [tool, setTool] = useState<Tool | null>(null);
  const [items, setItems] = useState<DataObject[]>([]);
  const [docImages, setDocImages] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [text, setText] = useState('');
  const [expanded, setExpanded] = useState<string | null>(null);

  const kind = kindOf(tool);

  const loadItems = useCallback(async () => {
    if (!toolId) return;
    const list = await doApi.listDataObjects({ tool_id: toolId, status: 'active', limit: 200 });
    setItems(Array.isArray(list) ? list : []);
  }, [toolId]);

  useEffect(() => {
    if (!toolId) return;
    setLoading(true);
    (async () => {
      try {
        // In the native shell, the tool name/description come from the bridge
        // context (no network) so we skip the getTool round-trip.
        let t: Tool;
        if (isNative) {
          const ctx = await getContext();
          t = { id: toolId, name: ctx.toolName ?? '', description: ctx.toolDescription ?? '' } as Tool;
          // The growth tool needs its config (child birth date/sex), which the
          // bridge context doesn't carry — fetch it.
          const g = (t.name + ' ' + (t.description ?? '')).toLowerCase();
          if (g.includes('成长') || g.includes('growth')) {
            try {
              const full = await toolsApi.getTool(toolId);
              t = { ...t, config: full.config };
            } catch {
              /* keep base context */
            }
          }
        } else {
          t = await toolsApi.getTool(toolId);
        }
        setTool(t);
        setTitle(t.name);
        await loadItems();
        setError('');
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setLoading(false);
      }
    })();
  }, [toolId, loadItems]);

  // Load document thumbnails for the doc tool.
  useEffect(() => {
    if (kind !== 'doc' || items.length === 0) return;
    let cancelled = false;
    (async () => {
      const map: Record<string, string> = {};
      for (const it of items) {
        if (docImages[it.id]) continue;
        try {
          const fl = await filesApi.listByDataObject(it.id);
          const img = bestImage(Array.isArray(fl) ? fl : []);
          if (img) map[it.id] = fileURL(img.id);
        } catch {
          /* best-effort */
        }
      }
      if (!cancelled && Object.keys(map).length) {
        setDocImages((prev) => ({ ...prev, ...map }));
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [kind, items, docImages]);

  const submitText = async () => {
    const content = text.trim();
    if (!toolId || !content) return;
    setBusy(true);
    setError('');
    try {
      const res = await rawInputs.createRawInput({ tool_id: toolId, input_type: 'text', raw_content: content });
      if (res.pipeline_id) await waitForPipeline(res.pipeline_id);
      setText('');
      await loadItems();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const submitImage = async (camera: boolean) => {
    if (!toolId) return;
    const images = await captureImages({ camera });
    if (images.length === 0) return;
    setBusy(true);
    setError('');
    try {
      for (const base64 of images) {
        const res = await rawInputs.createRawInput({
          tool_id: toolId,
          input_type: 'image',
          raw_content: base64,
          metadata: { filename: 'document.jpg', content_type: 'image/jpeg' },
        });
        if (res.pipeline_id) await waitForPipeline(res.pipeline_id);
      }
      await loadItems();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  const toggleDone = async (obj: DataObject) => {
    const done = !obj.attributes?.done;
    setItems((prev) =>
      prev.map((o) => (o.id === obj.id ? { ...o, attributes: { ...o.attributes, done } } : o)),
    );
    try {
      await doApi.updateDataObject(obj.id, {
        attributes: { ...obj.attributes, done },
        status: done ? 'completed' : 'active',
      });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      await loadItems();
    }
  };

  const remove = async (obj: DataObject) => {
    setItems((prev) => prev.filter((o) => o.id !== obj.id));
    try {
      await doApi.deleteDataObject(obj.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      await loadItems();
    }
  };

  const saveProfile = async (p: { birth_date: string; sex: 'male' | 'female' }) => {
    const updated = await toolsApi.updateTool(toolId, { config: p });
    setTool((prev) => (prev ? { ...prev, config: updated.config } : prev));
  };

  if (loading) return <div className="embed-state">加载中…</div>;

  if (kind === 'growth') {
    const cfg = (tool?.config ?? {}) as { birth_date?: string; sex?: string };
    return (
      <div className="embed">
        {!isNative && tool && <h1 className="embed-title">{tool.name}</h1>}
        {error && <div className="embed-error">{error}</div>}
        <GrowthView
          items={items}
          toolId={toolId}
          birthDate={cfg.birth_date}
          sex={cfg.sex === 'female' ? 'female' : 'male'}
          onChanged={loadItems}
          onSaveProfile={saveProfile}
        />
      </div>
    );
  }

  return (
    <div className="embed">
      {!isNative && tool && <h1 className="embed-title">{tool.name}</h1>}

      {error && <div className="embed-error">{error}</div>}

      <div className="embed-compose">
        {kind === 'doc' ? (
          <>
            <button className="embed-btn embed-btn-primary" onClick={() => submitImage(true)} disabled={busy}>
              {busy ? '识别中…' : '拍摄'}
            </button>
            <button className="embed-btn" onClick={() => submitImage(false)} disabled={busy}>
              相册
            </button>
          </>
        ) : (
          <>
            <input
              className="embed-input"
              value={text}
              placeholder={kind === 'todo' ? '用一句话添加待办…' : '输入内容…'}
              onChange={(e) => setText(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && submitText()}
              disabled={busy}
            />
            <button className="embed-btn embed-btn-primary" onClick={submitText} disabled={busy || !text.trim()}>
              {busy ? '处理中…' : '添加'}
            </button>
          </>
        )}
      </div>

      {items.length === 0 ? (
        <div className="embed-state">还没有内容</div>
      ) : (
        <ul className="embed-list">
          {items.map((obj) => {
            const open = expanded === obj.id;
            if (kind === 'doc') {
              const thumb = docImages[obj.id];
              return (
                <li key={obj.id} className="embed-row">
                  <div className="embed-doc">
                    <button className="embed-doc-main" onClick={() => setExpanded(open ? null : obj.id)}>
                      {thumb ? (
                        <img className="embed-thumb" src={thumb} alt="" />
                      ) : (
                        <div className="embed-thumb embed-thumb-empty">证</div>
                      )}
                      <div className="embed-doc-text">
                        <div className="embed-doc-title">{attr(obj, 'full_name') ?? attr(obj, 'cert_type') ?? displayTitle(obj)}</div>
                        {attr(obj, 'cert_type') && <div className="embed-doc-sub">{attr(obj, 'cert_type')}</div>}
                        {attr(obj, 'cert_number') && <div className="embed-doc-sub">{attr(obj, 'cert_number')}</div>}
                        {attr(obj, 'expiry_date') && <div className="embed-doc-sub">有效期至 {attr(obj, 'expiry_date')}</div>}
                      </div>
                    </button>
                    <button className="embed-del" onClick={() => remove(obj)} aria-label="delete">🗑</button>
                  </div>
                  {open && (
                    <div className="embed-doc-detail">
                      {thumb && <img className="embed-doc-full" src={thumb} alt="" />}
                      <dl className="embed-attrs">
                        {Object.entries(obj.attributes ?? {})
                          .filter(([, v]) => v != null && String(v).length <= 200)
                          .map(([k, v]) => (
                            <div className="embed-attr" key={k}>
                              <dt>{k}</dt>
                              <dd>{String(v)}</dd>
                            </div>
                          ))}
                      </dl>
                    </div>
                  )}
                </li>
              );
            }

            const done = !!obj.attributes?.done;
            return (
              <li key={obj.id} className={`embed-row ${done ? 'done' : ''}`}>
                <div className="embed-row-main">
                  {kind === 'todo' && (
                    <button
                      className={`embed-check ${done ? 'on' : ''}`}
                      onClick={() => toggleDone(obj)}
                      aria-label="toggle"
                    >
                      {done ? '✓' : ''}
                    </button>
                  )}
                  <button className="embed-row-title" onClick={() => setExpanded(open ? null : obj.id)}>
                    {displayTitle(obj)}
                  </button>
                  <button className="embed-del" onClick={() => remove(obj)} aria-label="delete">🗑</button>
                </div>
                {open && (
                  <dl className="embed-attrs">
                    {Object.entries(obj.attributes ?? {})
                      .filter(([, v]) => v != null && String(v).length <= 200)
                      .map(([k, v]) => (
                        <div className="embed-attr" key={k}>
                          <dt>{k}</dt>
                          <dd>{String(v)}</dd>
                        </div>
                      ))}
                  </dl>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
