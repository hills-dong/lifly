import { useState, type FormEvent } from 'react';
import { reminders as remindersApi } from '../api';
import type { Reminder } from '../api/types';
import { useFetchData } from '../hooks/useFetchData';

export default function RemindersPage() {
  const [statusFilter, setStatusFilter] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [formTitle, setFormTitle] = useState('');
  const [formDescription, setFormDescription] = useState('');
  const [formDueAt, setFormDueAt] = useState('');
  const [actionError, setActionError] = useState('');

  const { data: reminderList, loading, error, refetch } = useFetchData<Reminder[]>(
    () => remindersApi.listReminders({ status: statusFilter || undefined }),
    [statusFilter],
  );

  const handleCreate = async (e: FormEvent) => {
    e.preventDefault();
    try {
      await remindersApi.createReminder({
        title: formTitle,
        description: formDescription,
        trigger_at: new Date(formDueAt).toISOString(),
      });
      setShowForm(false);
      setFormTitle('');
      setFormDescription('');
      setFormDueAt('');
      refetch();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to create reminder');
    }
  };

  const handleDismiss = async (id: string) => {
    try {
      await remindersApi.dismissReminder(id);
      refetch();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to dismiss reminder');
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this reminder?')) return;
    try {
      await remindersApi.deleteReminder(id);
      refetch();
    } catch (err) {
      setActionError(err instanceof Error ? err.message : 'Failed to delete reminder');
    }
  };

  const displayError = error || actionError;
  const items = reminderList ?? [];

  return (
    <div>
      <div className="page-header">
        <h1>Reminders</h1>
        <div className="page-actions">
          <button className="btn btn-primary" onClick={() => setShowForm(!showForm)}>
            {showForm ? 'Cancel' : 'New Reminder'}
          </button>
        </div>
      </div>

      {displayError && <div className="alert alert-error">{displayError}</div>}

      {showForm && (
        <form onSubmit={handleCreate} className="form card" style={{ marginBottom: '1.5rem' }}>
          <div className="form-group">
            <label htmlFor="r-title">Title</label>
            <input
              id="r-title"
              type="text"
              value={formTitle}
              onChange={(e) => setFormTitle(e.target.value)}
              required
              autoFocus
            />
          </div>
          <div className="form-group">
            <label htmlFor="r-desc">Description</label>
            <textarea
              id="r-desc"
              value={formDescription}
              onChange={(e) => setFormDescription(e.target.value)}
              rows={3}
            />
          </div>
          <div className="form-group">
            <label htmlFor="r-due">Due At</label>
            <input
              id="r-due"
              type="datetime-local"
              value={formDueAt}
              onChange={(e) => setFormDueAt(e.target.value)}
              required
            />
          </div>
          <button type="submit" className="btn btn-primary">Create</button>
        </form>
      )}

      <div className="filter-bar">
        <select value={statusFilter} onChange={(e) => setStatusFilter(e.target.value)}>
          <option value="">All</option>
          <option value="pending">Pending</option>
          <option value="triggered">Triggered</option>
          <option value="dismissed">Dismissed</option>
        </select>
      </div>

      {loading ? (
        <div className="loading">Loading...</div>
      ) : items.length === 0 ? (
        <p className="empty-state">No reminders.</p>
      ) : (
        <ul className="reminder-list">
          {items.map((r) => (
            <li key={r.id} className={`reminder-item reminder-${r.status}`}>
              <div className="reminder-info">
                <strong>{r.title}</strong>
                {r.description && <p>{r.description}</p>}
                <span className="reminder-due">
                  Due: {new Date(r.trigger_at).toLocaleString()}
                </span>
                <span className={`badge badge-${r.status}`}>{r.status}</span>
              </div>
              <div className="reminder-actions">
                {r.status !== 'dismissed' && (
                  <button className="btn btn-secondary btn-sm" onClick={() => handleDismiss(r.id)}>
                    Dismiss
                  </button>
                )}
                <button className="btn btn-danger btn-sm" onClick={() => handleDelete(r.id)}>
                  Delete
                </button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
