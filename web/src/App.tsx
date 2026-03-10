import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './contexts/AuthContext';
import Layout from './components/Layout';
import LoginPage from './pages/LoginPage';
import HomePage from './pages/HomePage';
import ToolPage from './pages/ToolPage';
import TodoInputPage from './pages/TodoInputPage';
import DocUploadPage from './pages/DocUploadPage';
import DataObjectPage from './pages/DataObjectPage';
import RemindersPage from './pages/RemindersPage';
import SearchPage from './pages/SearchPage';
import './App.css';

function RequireAuth({ children }: { children: React.ReactNode }) {
  const { isAuthenticated } = useAuth();
  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }
  return <>{children}</>;
}

function AppRoutes() {
  return (
    <Routes>
      <Route path="/login" element={<LoginPage />} />
      <Route
        element={
          <RequireAuth>
            <Layout />
          </RequireAuth>
        }
      >
        <Route path="/" element={<HomePage />} />
        <Route path="/tools/:id" element={<ToolPage />} />
        <Route path="/tools/:id/new-todo" element={<TodoInputPage />} />
        <Route path="/tools/:id/upload-doc" element={<DocUploadPage />} />
        <Route path="/data-objects/:id" element={<DataObjectPage />} />
        <Route path="/reminders" element={<RemindersPage />} />
        <Route path="/search" element={<SearchPage />} />
      </Route>
    </Routes>
  );
}

export default function App() {
  return (
    <BrowserRouter>
      <AuthProvider>
        <AppRoutes />
      </AuthProvider>
    </BrowserRouter>
  );
}
