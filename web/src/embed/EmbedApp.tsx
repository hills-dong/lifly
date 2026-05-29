import { useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import EmbedToolView from './EmbedToolView';
import { getContext } from './lifly';

/**
 * Host for the persistent native WebView. The current tool comes from the bridge
 * context; the native shell switches tools by updating the context and calling
 * `window.__lifly_refresh()`. Remounting via `key` gives each tool a clean slate.
 */
export function EmbedApp() {
  const [toolId, setToolId] = useState<string | undefined>();

  useEffect(() => {
    const apply = async () => {
      const ctx = await getContext();
      setToolId(ctx.toolId);
    };
    (window as Window & { __lifly_refresh?: () => void }).__lifly_refresh = apply;
    apply();
    return () => {
      delete (window as Window & { __lifly_refresh?: () => void }).__lifly_refresh;
    };
  }, []);

  if (!toolId) return <div className="embed-state" />;
  return <EmbedToolView key={toolId} toolId={toolId} />;
}

/** Browser route wrapper: tool id from the URL path. */
export function EmbedRoute() {
  const { id } = useParams<{ id: string }>();
  if (!id) return <div className="embed-state">缺少工具 ID</div>;
  return <EmbedToolView toolId={id} />;
}
