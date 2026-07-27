import { lazy, Suspense } from 'react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';

import { AppShell } from './AppShell';
import { LoadingState } from '@/components/LoadingState';
import { LanguageProvider } from '@/i18n/LanguageContext';
import { GitEvolution } from '@/pages/GitEvolution';
import { Graph } from '@/pages/Graph';
import { Health } from '@/pages/Health';
import { Home } from '@/pages/Home';
import { Insights } from '@/pages/Insights';
import { Settings } from '@/pages/Settings';
import { Workspaces } from '@/pages/Workspaces';

const Viewer = lazy(async () => {
  const module = await import('@/pages/Viewer');
  return { default: module.Viewer };
});

interface AppProps {
  initialEntries?: string[];
}

export function App({ initialEntries }: AppProps = {}): JSX.Element {
  return (
    <LanguageProvider>
      <MemoryRouter
        {...(initialEntries !== undefined ? { initialEntries } : {})}
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
            <Route
              path="viewer"
              element={
                <Suspense fallback={<LoadingState label="Loading viewer…" />}>
                  <Viewer />
                </Suspense>
              }
            />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </MemoryRouter>
    </LanguageProvider>
  );
}
