import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { tools as toolsApi } from '../api';
import type { Tool } from '../api/types';

export default function HomePage() {
  const [toolList, setToolList] = useState<Tool[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');

  useEffect(() => {
    toolsApi
      .listTools()
      .then(setToolList)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) return <div className="loading">Loading...</div>;
  if (error) return <div className="alert alert-error">{error}</div>;

  return (
    <div>
      <h1>Tools</h1>
      {toolList.length === 0 ? (
        <p className="empty-state">No tools available yet.</p>
      ) : (
        <div className="card-grid">
          {toolList.map((tool) => (
            <Link to={`/tools/${tool.id}`} key={tool.id} className="card card-link">
              <div className="card-icon">{tool.icon || tool.name.charAt(0).toUpperCase()}</div>
              <h3>{tool.name}</h3>
              <p>{tool.description}</p>
            </Link>
          ))}
        </div>
      )}
    </div>
  );
}
