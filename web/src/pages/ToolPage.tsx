import { useEffect, useState } from 'react';
import { useParams, Link } from 'react-router-dom';
import { tools as toolsApi, dataObjects as doApi, files as filesApi } from '../api';
import type { Tool, DataObject, FileStorage } from '../api/types';
import { displayTitle } from '../utils/displayTitle';

export default function ToolPage() {
  const { id } = useParams<{ id: string }>();
  const [tool, setTool] = useState<Tool | null>(null);
  const [objects, setObjects] = useState<DataObject[]>([]);
  const [statusFilter, setStatusFilter] = useState('');
  const [thumbnails, setThumbnails] = useState<Record<string, FileStorage | null>>({});
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!id) return;
    Promise.all([
      toolsApi.getTool(id),
      doApi.listDataObjects({ tool_id: id, status: statusFilter || undefined }),
    ])
      .then(([t, res]) => {
        setTool(t);
        setObjects(Array.isArray(res) ? res : []);
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [id, statusFilter]);

  // Fetch thumbnail images for doc-type data objects
  useEffect(() => {
    if (!tool || objects.length === 0) return;
    const nameLower = (tool.name + ' ' + tool.description).toLowerCase();
    const isDocTool = nameLower.includes('证件') || nameLower.includes('document') || nameLower.includes('id-doc');
    if (!isDocTool) return;

    objects.forEach((obj) => {
      if (thumbnails[obj.id] !== undefined) return; // already fetched or in-flight
      setThumbnails((prev) => ({ ...prev, [obj.id]: null })); // mark as in-flight
      filesApi.listByDataObject(obj.id).then((fileList) => {
        const files = Array.isArray(fileList) ? fileList : [];
        // Prefer "original" role image, fall back to any image
        const original = files.find((f: FileStorage) => f.mime_type.startsWith('image/') && f.role === 'original');
        const anyImage = files.find((f: FileStorage) => f.mime_type.startsWith('image/'));
        const thumb = original || anyImage || null;
        setThumbnails((prev) => ({ ...prev, [obj.id]: thumb }));
      }).catch(() => {
        // ignore — thumbnail is best-effort
      });
    });
  }, [tool, objects, thumbnails]);

  const handleToggleTodo = async (obj: DataObject) => {
    const done = !obj.attributes?.done;
    await doApi.updateDataObject(obj.id, {
      attributes: { ...obj.attributes, done },
      status: done ? 'completed' : 'active',
    });
    setObjects((prev) =>
      prev.map((o) =>
        o.id === obj.id
          ? { ...o, attributes: { ...o.attributes, done }, status: done ? 'completed' : 'active' }
          : o
      )
    );
  };

  if (loading) return <div className="loading">Loading...</div>;
  if (error) return <div className="alert alert-error">{error}</div>;
  if (!tool) return <div className="alert alert-error">Tool not found</div>;

  const nameLower = (tool.name + ' ' + tool.description).toLowerCase();
  const isTodo = nameLower.includes('todo');
  const isDoc = nameLower.includes('证件') || nameLower.includes('document') || nameLower.includes('id-doc');

  return (
    <div>
      <div className="page-header">
        <div>
          <Link to="/" className="back-link">Back to Tools</Link>
          <h1>{tool.name}</h1>
          <p>{tool.description}</p>
        </div>
        <div className="page-actions">
          {isTodo && (
            <Link to={`/tools/${id}/new-todo`} className="btn btn-primary">
              New Todo
            </Link>
          )}
          {isDoc && (
            <Link to={`/tools/${id}/upload-doc`} className="btn btn-primary">
              Upload Document
            </Link>
          )}
        </div>
      </div>

      <div className="filter-bar">
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
          <option value="">All statuses</option>
          <option value="active">Active</option>
          <option value="completed">Completed</option>
          <option value="archived">Archived</option>
        </select>
      </div>

      {objects.length === 0 ? (
        <p className="empty-state">No items yet.</p>
      ) : isTodo ? (
        <ul className="todo-list">
          {objects.map((obj) => (
            <li key={obj.id} className={`todo-item ${obj.attributes?.done ? 'done' : ''}`}>
              <label className="todo-checkbox">
                <input
                  type="checkbox"
                  checked={!!obj.attributes?.done}
                  onChange={() => handleToggleTodo(obj)}
                />
                <span>{displayTitle(obj)}</span>
              </label>
              <Link to={`/data-objects/${obj.id}`} className="btn btn-text btn-sm">
                View
              </Link>
            </li>
          ))}
        </ul>
      ) : isDoc ? (
        <div className="card-grid">
          {objects.map((obj) => (
            <Link to={`/data-objects/${obj.id}`} key={obj.id} className="card card-link">
              {thumbnails[obj.id] && (
                <img
                  src={filesApi.getFileUrl(thumbnails[obj.id]!.id)}
                  alt={displayTitle(obj)}
                  style={{ width: '100%', maxHeight: '120px', objectFit: 'cover', borderRadius: '4px', marginBottom: '8px' }}
                />
              )}
              <h3>{displayTitle(obj)}</h3>
              <p className="card-meta">
                {obj.status} &middot; {new Date(obj.created_at).toLocaleDateString()}
              </p>
              <span className={`badge badge-${obj.status}`}>{obj.status}</span>
            </Link>
          ))}
        </div>
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Type</th>
              <th>Status</th>
              <th>Created</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {objects.map((obj) => (
              <tr key={obj.id}>
                <td>{displayTitle(obj)}</td>
                <td>{obj.status}</td>
                <td><span className={`badge badge-${obj.status}`}>{obj.status}</span></td>
                <td>{new Date(obj.created_at).toLocaleDateString()}</td>
                <td>
                  <Link to={`/data-objects/${obj.id}`} className="btn btn-text btn-sm">
                    View
                  </Link>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  );
}
