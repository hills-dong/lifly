import { useState, useRef, type DragEvent } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { rawInputs } from '../api';
import { useWebSocket } from '../hooks/useWebSocket';

export default function DocUploadPage() {
  const { id: toolId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [dragging, setDragging] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [pipelineStatus, setPipelineStatus] = useState('');
  const [error, setError] = useState('');

  useWebSocket((msg) => {
    if (msg.type === 'pipeline.status') {
      const status = msg.payload.status as string;
      setPipelineStatus(status);
      if (status === 'completed') {
        setTimeout(() => navigate(`/tools/${toolId}`), 1000);
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
    if (dropped) setFile(dropped);
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
      // Convert file to base64 for raw input
      const reader = new FileReader();
      const base64 = await new Promise<string>((resolve, reject) => {
        reader.onload = () => resolve(reader.result as string);
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

  return (
    <div>
      <Link to={`/tools/${toolId}`} className="back-link">Back to Tool</Link>
      <h1>Upload Document</h1>

      {error && <div className="alert alert-error">{error}</div>}

      {pipelineStatus && (
        <div className={`alert ${pipelineStatus === 'completed' ? 'alert-success' : 'alert-info'}`}>
          Pipeline status: {pipelineStatus}
          {pipelineStatus === 'completed' && ' - Redirecting...'}
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
            <p>Drag and drop a file here, or click to browse</p>
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
