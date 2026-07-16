import { MemoryRouter, Route, Routes } from 'react-router-dom';

import { AppShell } from './AppShell';
import { LanguageProvider } from '@/i18n/LanguageContext';
import { GitEvolution } from '@/pages/GitEvolution';
import { Graph } from '@/pages/Graph';
import { Health } from '@/pages/Health';
import { Home } from '@/pages/Home';
import { Insights } from '@/pages/Insights';
import { Settings } from '@/pages/Settings';
import { Viewer } from '@/pages/Viewer';
import { Workspaces } from '@/pages/Workspaces';

export function App(): JSX.Element {
  return (
    <LanguageProvider>
      <MemoryRouter
        future={{
          v7_startTransition: true,
          v7_relativeSplatPath: true,
        }}
      >
        <Routes>
          <Route element={<AppShell />}>
            <Route index element={<Home />} />
            <Route path="workspaces" element={<Workspaces />} />
            <Route path="graph" element={<Graph />} />
            <Route path="insights" element={<Insights />} />
            <Route path="health" element={<Health />} />
            <Route path="evolution" element={<GitEvolution />} />
            <Route path="viewer" element={<Viewer />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </MemoryRouter>
    </LanguageProvider>
  );
}
