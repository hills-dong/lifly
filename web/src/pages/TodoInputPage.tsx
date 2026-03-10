import { useState, type FormEvent } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { rawInputs } from '../api';
import { useWebSocket } from '../hooks/useWebSocket';

export default function TodoInputPage() {
  const { id: toolId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [content, setContent] = useState('');
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

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!toolId || !content.trim()) return;
    setSubmitting(true);
    setError('');
    try {
      await rawInputs.createRawInput({
        tool_id: toolId,
        type: 'todo',
        content: content.trim(),
      });
      setPipelineStatus('submitted');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create todo');
      setSubmitting(false);
    }
  };

  return (
    <div>
      <Link to={`/tools/${toolId}`} className="back-link">Back to Tool</Link>
      <h1>New Todo</h1>

      {error && <div className="alert alert-error">{error}</div>}

      {pipelineStatus && (
        <div className={`alert ${pipelineStatus === 'completed' ? 'alert-success' : 'alert-info'}`}>
          Pipeline status: {pipelineStatus}
          {pipelineStatus === 'completed' && ' - Redirecting...'}
        </div>
      )}

      <form onSubmit={handleSubmit} className="form">
        <div className="form-group">
          <label htmlFor="content">What needs to be done?</label>
          <input
            id="content"
            type="text"
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Enter your todo item..."
            required
            autoFocus
            disabled={submitting}
          />
        </div>
        <div className="form-actions">
          <button type="submit" className="btn btn-primary" disabled={submitting}>
            {submitting ? 'Submitting...' : 'Create Todo'}
          </button>
          <Link to={`/tools/${toolId}`} className="btn btn-secondary">
            Cancel
          </Link>
        </div>
      </form>
    </div>
  );
}
