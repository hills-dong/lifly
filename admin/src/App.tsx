import { Authenticated, Refine } from "@refinedev/core";
import routerProvider from "@refinedev/react-router";
import { Navigate, Route, Routes } from "react-router-dom";

import { Layout } from "./components/Layout";
import { MetaProvider } from "./meta";
import { Home } from "./pages/Home";
import { LoginPage } from "./pages/Login";
import { ResourceCreate } from "./pages/ResourceCreate";
import { ResourceEdit } from "./pages/ResourceEdit";
import { ResourceList } from "./pages/ResourceList";
import { authProvider } from "./providers/authProvider";
import { dataProvider } from "./providers/dataProvider";

export default function App() {
  return (
    <Refine
      dataProvider={dataProvider}
      authProvider={authProvider}
      routerProvider={routerProvider}
      options={{ disableTelemetry: true, warnWhenUnsavedChanges: false }}
    >
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route
          element={
            <Authenticated key="protected" fallback={<Navigate to="/login" />}>
              <MetaProvider>
                <Layout />
              </MetaProvider>
            </Authenticated>
          }
        >
          <Route index element={<Home />} />
          <Route path="/r/:resource" element={<ResourceList />} />
          <Route path="/r/:resource/create" element={<ResourceCreate />} />
          <Route path="/r/:resource/:id" element={<ResourceEdit />} />
        </Route>
        <Route path="*" element={<Navigate to="/" />} />
      </Routes>
    </Refine>
  );
}
