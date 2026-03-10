import { useEffect, useState, type FormEvent } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { dataObjects as doApi, files as filesApi } from '../api';
import type { DataObject, FileStorage } from '../api/types';

function displayTitle(obj: DataObject): string {
  const a = obj.attributes;
  const candidates = [a?.content, a?.title, a?.full_name, a?.cert_type];
  for (const c of candidates) {
    if (c != null) {
      const s = String(c);
      if (s.length > 0 && s.length <= 200) return s;
    }
  }
  return 'Untitled';
}

export default function DataObjectPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [obj, setObj] = useState<DataObject | null>(null);
  const [editing, setEditing] = useState(false);
  const [editAttrs, setEditAttrs] = useState('');
  const [associatedFiles, setAssociatedFiles] = useState<FileStorage[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    if (!id) return;
    doApi
      .getDataObject(id)
      .then((data) => {
        setObj(data);
        setEditAttrs(JSON.stringify(data.attributes, null, 2));
        if ((data as DataObject & { files?: FileStorage[] }).files) {
          setAssociatedFiles((data as DataObject & { files?: FileStorage[] }).files!);
        }
      })
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, [id]);

  const handleSave = async (e: FormEvent) => {
    e.preventDefault();
    if (!id) return;
    try {
      const attrs = JSON.parse(editAttrs);
      const updated = await doApi.updateDataObject(id, {
        attributes: attrs,
      });
      setObj(updated);
      setEditing(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Update failed');
    }
  };

  const handleDelete = async () => {
    if (!id) return;
    if (!window.confirm('Are you sure you want to delete this item?')) return;
    try {
      await doApi.deleteDataObject(id);
      navigate(-1);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Delete failed');
    }
  };

  if (loading) return <div className="loading">Loading...</div>;
  if (error) return <div className="alert alert-error">{error}</div>;
  if (!obj) return <div className="alert alert-error">Not found</div>;

  return (
    <div>
      <Link to={`/tools/${obj.tool_id}`} className="back-link">Back to Tool</Link>

      {editing ? (
        <form onSubmit={handleSave} className="form">
          <h1>Edit Item</h1>
          <div className="form-group">
            <label htmlFor="attrs">Attributes (JSON)</label>
            <textarea
              id="attrs"
              value={editAttrs}
              onChange={(e) => setEditAttrs(e.target.value)}
              rows={10}
              className="monospace"
            />
          </div>
          <div className="form-actions">
            <button type="submit" className="btn btn-primary">Save</button>
            <button type="button" className="btn btn-secondary" onClick={() => setEditing(false)}>
              Cancel
            </button>
          </div>
        </form>
      ) : (
        <div>
          <div className="page-header">
            <h1>{displayTitle(obj)}</h1>
            <div className="page-actions">
              <button className="btn btn-secondary" onClick={() => setEditing(true)}>Edit</button>
              <button className="btn btn-danger" onClick={handleDelete}>Delete</button>
            </div>
          </div>

          <div className="detail-grid">
            <div className="detail-row">
              <span className="detail-label">Status</span>
              <span className={`badge badge-${obj.status}`}>{obj.status}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Created</span>
              <span>{new Date(obj.created_at).toLocaleString()}</span>
            </div>
            <div className="detail-row">
              <span className="detail-label">Updated</span>
              <span>{new Date(obj.updated_at).toLocaleString()}</span>
            </div>
          </div>

          <h2>Attributes</h2>
          <div className="attributes-view">
            {Object.entries(obj.attributes || {}).map(([key, value]) => {
              const str = typeof value === 'object' ? JSON.stringify(value) : String(value ?? '');
              // Detect base64 image data and truncate display
              const isBase64Image = typeof value === 'string' && value.length > 500 &&
                /^[A-Za-z0-9+/=\s]+$/.test(value.slice(0, 100));
              return (
                <div className="detail-row" key={key}>
                  <span className="detail-label">{key}</span>
                  <span>{isBase64Image ? `[Image data, ${(str.length / 1024).toFixed(0)} KB]` : str}</span>
                </div>
              );
            })}
            {Object.keys(obj.attributes || {}).length === 0 && (
              <p className="empty-state">No attributes.</p>
            )}
          </div>

          {associatedFiles.length > 0 && (
            <>
              <h2>Files</h2>
              <ul className="file-list">
                {associatedFiles.map((f) => (
                  <li key={f.id}>
                    <a href={filesApi.getFileUrl(f.id)} target="_blank" rel="noopener noreferrer">
                      {f.file_name}
                    </a>
                    <span className="file-meta">
                      {f.mime_type} &middot; {(f.file_size / 1024).toFixed(1)} KB
                    </span>
                  </li>
                ))}
              </ul>
            </>
          )}
        </div>
      )}
    </div>
  );
}
