import { Outlet } from 'react-router-dom';

import { Nav } from './Nav';

export function AppShell(): JSX.Element {
  return (
    <div className="app-layout">
      <aside className="app-sidebar">
        <div className="app-brand">
          CodeCompass
          <div className="app-brand-sub">Understand any codebase</div>
        </div>
        <Nav />
      </aside>
      <main className="app-content" role="main">
        <Outlet />
      </main>
    </div>
  );
}
