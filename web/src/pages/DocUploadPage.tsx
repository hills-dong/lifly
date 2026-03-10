import { useState, useRef, useCallback, type DragEvent } from 'react';
import { useParams, Link } from 'react-router-dom';
import { rawInputs, dataObjects as doApi } from '../api';
import type { DataObject } from '../api/types';
import { useWebSocket } from '../hooks/useWebSocket';

const CERT_FIELDS: { key: string; label: string }[] = [
  { key: 'cert_type', label: '证件类型' },
  { key: 'cert_number', label: '证件号码' },
  { key: 'full_name', label: '姓名' },
  { key: 'expiry_date', label: '有效期' },
  { key: 'issuing_country', label: '签发国家' },
];

export default function DocUploadPage() {
  const { id: toolId } = useParams<{ id: string }>();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [dragging, setDragging] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [pipelineStatus, setPipelineStatus] = useState('');
  const [error, setError] = useState('');
  const [resultObject, setResultObject] = useState<DataObject | null>(null);

  const loadResult = useCallback(async () => {
    if (!toolId) return;
    try {
      const page = await doApi.listDataObjects({ tool_id: toolId, limit: 1 });
      if (page.items.length > 0) {
        setResultObject(page.items[0]);
      }
    } catch {
      // silently ignore — the user can still navigate to the tool page
    }
  }, [toolId]);

  useWebSocket((msg) => {
    if (msg.type === 'pipeline.status') {
      const status = msg.payload.status as string;
      setPipelineStatus(status);
      if (status === 'completed') {
        loadResult();
      }
    }
  });

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    setDragging(true);
  };

  const handleDragLeave = () => setDragging(false);

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    setDragging(false);
    const dropped = e.dataTransfer.files[0];
    if (dropped && dropped.type.startsWith('image/')) setFile(dropped);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0];
    if (selected) setFile(selected);
  };

  const handleSubmit = async () => {
    if (!toolId || !file) return;
    setSubmitting(true);
    setError('');
    try {
      const reader = new FileReader();
      const base64 = await new Promise<string>((resolve, reject) => {
        reader.onload = () => {
          const dataUrl = reader.result as string;
          // Strip data URL prefix, send only the base64 payload
          const idx = dataUrl.indexOf(',');
          resolve(idx >= 0 ? dataUrl.substring(idx + 1) : dataUrl);
        };
        reader.onerror = reject;
        reader.readAsDataURL(file);
      });

      await rawInputs.createRawInput({
        tool_id: toolId,
        type: 'document',
        content: base64,
        metadata: {
          filename: file.name,
          content_type: file.type,
          size: file.size,
        },
      });
      setPipelineStatus('submitted');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Upload failed');
      setSubmitting(false);
    }
  };

  const handleReset = () => {
    setFile(null);
    setSubmitting(false);
    setPipelineStatus('');
    setError('');
    setResultObject(null);
  };

  // ---------- Extraction results view ----------
  if (resultObject) {
    const attrs = resultObject.attributes ?? {};
    return (
      <div>
        <Link to={`/tools/${toolId}`} className="back-link">Back to Tool</Link>
        <h1>Extraction Results</h1>

        <div className="alert alert-success">Pipeline completed — fields extracted</div>

        <div className="detail-grid">
          {CERT_FIELDS.map(({ key, label }) => (
            <div className="detail-row" key={key}>
              <span className="detail-label">{label}</span>
              <span>{attrs[key] != null ? String(attrs[key]) : '—'}</span>
            </div>
          ))}
        </div>

        <div className="form-actions">
          <button className="btn btn-primary" onClick={handleReset}>
            Upload Another
          </button>
          <Link to={`/tools/${toolId}`} className="btn btn-secondary">
            Back to Tool
          </Link>
        </div>
      </div>
    );
  }

  // ---------- Upload view ----------
  return (
    <div>
      <Link to={`/tools/${toolId}`} className="back-link">Back to Tool</Link>
      <h1>Upload Document</h1>

      {error && <div className="alert alert-error">{error}</div>}

      {pipelineStatus && (
        <div className={`alert ${pipelineStatus === 'completed' ? 'alert-success' : 'alert-info'}`}>
          Pipeline status: {pipelineStatus}
        </div>
      )}

      <div
        className={`drop-zone ${dragging ? 'drop-zone-active' : ''} ${file ? 'drop-zone-has-file' : ''}`}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        onClick={() => fileInputRef.current?.click()}
      >
        <input
          ref={fileInputRef}
          type="file"
          accept="image/*"
          onChange={handleFileChange}
          style={{ display: 'none' }}
        />
        {file ? (
          <div className="drop-zone-file">
            <strong>{file.name}</strong>
            <span>{(file.size / 1024).toFixed(1)} KB</span>
          </div>
        ) : (
          <div className="drop-zone-prompt">
            <p>Drag and drop an image here, or click to browse</p>
          </div>
        )}
      </div>

      <div className="form-actions">
        <button
          className="btn btn-primary"
          onClick={handleSubmit}
          disabled={!file || submitting}
        >
          {submitting ? 'Uploading...' : 'Upload'}
        </button>
        <Link to={`/tools/${toolId}`} className="btn btn-secondary">
          Cancel
        </Link>
      </div>
    </div>
  );
}
