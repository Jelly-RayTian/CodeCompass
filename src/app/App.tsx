import { MemoryRouter, Route, Routes } from 'react-router-dom';

import { AppShell } from './AppShell';
import { Graph } from '@/pages/Graph';
import { Home } from '@/pages/Home';
import { Settings } from '@/pages/Settings';
import { Workspaces } from '@/pages/Workspaces';

export function App(): JSX.Element {
  return (
    <MemoryRouter>
      <Routes>
        <Route element={<AppShell />}>
          <Route index element={<Home />} />
          <Route path="workspaces" element={<Workspaces />} />
          <Route path="graph" element={<Graph />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      </Routes>
    </MemoryRouter>
  );
}
