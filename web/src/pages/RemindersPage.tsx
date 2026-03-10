import { useEffect, useState, type FormEvent } from 'react';
import { reminders as remindersApi } from '../api';
import type { Reminder } from '../api/types';

export default function RemindersPage() {
  const [reminderList, setReminderList] = useState<Reminder[]>([]);
  const [statusFilter, setStatusFilter] = useState('');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showForm, setShowForm] = useState(false);
  const [formTitle, setFormTitle] = useState('');
  const [formDescription, setFormDescription] = useState('');
  const [formDueAt, setFormDueAt] = useState('');

  const fetchReminders = () => {
    setLoading(true);
    remindersApi
      .listReminders({ status: statusFilter || undefined })
      .then(setReminderList)
      .catch((err) => setError(err.message))
      .finally(() => setLoading(false));
  };

  useEffect(() => {
    fetchReminders();
  }, [statusFilter]);

  const handleCreate = async (e: FormEvent) => {
    e.preventDefault();
    try {
      await remindersApi.createReminder({
        title: formTitle,
        description: formDescription,
        due_at: new Date(formDueAt).toISOString(),
      });
      setShowForm(false);
      setFormTitle('');
      setFormDescription('');
      setFormDueAt('');
      fetchReminders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create reminder');
    }
  };

  const handleDismiss = async (id: string) => {
    try {
      await remindersApi.dismissReminder(id);
      fetchReminders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to dismiss reminder');
    }
  };

  const handleDelete = async (id: string) => {
    if (!window.confirm('Delete this reminder?')) return;
    try {
      await remindersApi.deleteReminder(id);
      fetchReminders();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete reminder');
    }
  };

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

      {error && <div className="alert alert-error">{error}</div>}

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
      ) : reminderList.length === 0 ? (
        <p className="empty-state">No reminders.</p>
      ) : (
        <ul className="reminder-list">
          {reminderList.map((r) => (
            <li key={r.id} className={`reminder-item reminder-${r.status}`}>
              <div className="reminder-info">
                <strong>{r.title}</strong>
                {r.description && <p>{r.description}</p>}
                <span className="reminder-due">
                  Due: {new Date(r.due_at).toLocaleString()}
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
