import { NavLink, Outlet, useNavigate } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  const handleLogout = async () => {
    await logout();
    navigate('/login');
  };

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="sidebar-brand">
          <h2>Lifly</h2>
        </div>
        <nav className="sidebar-nav">
          <NavLink to="/" end className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}>
            Tools
          </NavLink>
          <NavLink to="/reminders" className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}>
            Reminders
          </NavLink>
          <NavLink to="/search" className={({ isActive }) => isActive ? 'nav-link active' : 'nav-link'}>
            Search
          </NavLink>
        </nav>
      </aside>
      <div className="main-wrapper">
        <header className="header">
          <div className="header-left" />
          <div className="header-right">
            {user && <span className="user-name">{user.display_name || user.username}</span>}
            <button className="btn btn-text" onClick={handleLogout}>
              Logout
            </button>
          </div>
        </header>
        <main className="main-content">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
