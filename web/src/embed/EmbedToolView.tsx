import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { tools as toolsApi, dataObjects as doApi, rawInputs, pipelines } from '../api';
import type { DataObject, Tool } from '../api/types';
import { displayTitle } from '../utils/displayTitle';
import { captureImages, isNative, setTitle } from './lifly';
import './embed.css';

type Kind = 'todo' | 'doc' | 'generic';

function kindOf(tool: Tool | null): Kind {
  if (!tool) return 'generic';
  const s = (tool.name + ' ' + (tool.description ?? '')).toLowerCase();
  if (s.includes('todo') || s.includes('待办')) return 'todo';
  if (s.includes('证件') || s.includes('document') || s.includes('id-doc')) return 'doc';
  return 'generic';
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

export default function EmbedToolView() {
  const { id: toolId } = useParams<{ id: string }>();
  const [tool, setTool] = useState<Tool | null>(null);
  const [items, setItems] = useState<DataObject[]>([]);
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const [text, setText] = useState('');
  const [expanded, setExpanded] = useState<string | null>(null);

  const kind = kindOf(tool);

  const loadItems = useCallback(async () => {
    if (!toolId) return;
    const list = await doApi.listDataObjects({ tool_id: toolId, status: 'active' });
    setItems(Array.isArray(list) ? list : []);
  }, [toolId]);

  useEffect(() => {
    if (!toolId) return;
    setLoading(true);
    Promise.all([toolsApi.getTool(toolId), loadItems()])
      .then(([t]) => {
        setTool(t);
        setTitle(t.name);
        setError('');
      })
      .catch((e) => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false));
  }, [toolId, loadItems]);

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

  if (loading) return <div className="embed-state">加载中…</div>;

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
            const done = !!obj.attributes?.done;
            const open = expanded === obj.id;
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
                  <button className="embed-del" onClick={() => remove(obj)} aria-label="delete">
                    🗑
                  </button>
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
