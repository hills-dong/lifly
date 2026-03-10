import { useState, type FormEvent } from 'react';
import { Link } from 'react-router-dom';
import { dataObjects } from '../api';
import type { DataObject } from '../api/types';

export default function SearchPage() {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<DataObject[]>([]);
  const [searched, setSearched] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSearch = async (e: FormEvent) => {
    e.preventDefault();
    if (!query.trim()) return;
    setLoading(true);
    setError('');
    try {
      const res = await dataObjects.searchDataObjects({ query: query.trim() });
      setResults(res.items || []);
      setSearched(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Search failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h1>Search</h1>

      <form onSubmit={handleSearch} className="search-form">
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search data objects..."
          className="search-input"
          autoFocus
        />
        <button type="submit" className="btn btn-primary" disabled={loading}>
          {loading ? 'Searching...' : 'Search'}
        </button>
      </form>

      {error && <div className="alert alert-error">{error}</div>}

      {searched && results.length === 0 && (
        <p className="empty-state">No results found.</p>
      )}

      {results.length > 0 && (
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
            {results.map((obj) => (
              <tr key={obj.id}>
                <td>{obj.title}</td>
                <td>{obj.type}</td>
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
