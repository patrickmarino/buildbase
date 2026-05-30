import React, { useState } from "react";
import { AppProvider, ToastHost, useStore } from "./store/AppContext";
import { Sidebar, type PageId } from "./components/Shell";
import { LoginPage } from "./pages/LoginPage";
import { RolesPage } from "./pages/RolesPage";
import { UsersPage } from "./pages/UsersPage";
import { OrgPage } from "./pages/OrgPage";
import { AuditPage } from "./pages/AuditPage";
import { KeysPage } from "./pages/KeysPage";

const PAGES: Record<PageId, React.ComponentType> = {
  users: UsersPage,
  roles: RolesPage,
  org: OrgPage,
  audit: AuditPage,
  keys: KeysPage,
};

function AppShell() {
  const { me, ready, accent } = useStore();
  const [page, setPage] = useState<PageId>("roles");

  if (!ready) return null;
  if (!me) return <LoginPage />;

  const Page = PAGES[page];
  const style = { ["--accent"]: accent, ["--r"]: "4px" } as React.CSSProperties;

  return (
    <div className="app" data-theme="light" data-density="comfortable" data-titleface="serif" style={style}>
      <Sidebar page={page} onNavigate={setPage} me={me} />
      <Page key={page} />
      <ToastHost />
    </div>
  );
}

export function App() {
  return (
    <AppProvider>
      <AppShell />
    </AppProvider>
  );
}
