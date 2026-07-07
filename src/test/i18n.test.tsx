import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, it, expect, beforeEach } from 'vitest';

import { LanguageProvider } from '@/i18n/LanguageContext';
import { useT } from '@/i18n/useT';
import { mockTauriCommand } from '@/test/setup';

function TestComponent() {
  const { t, lang, setLang } = useT();
  return (
    <div>
      <span data-testid="home-title">{t.home.title}</span>
      <span data-testid="lang">{lang}</span>
      <button onClick={() => setLang('zh')} type="button">
        Switch
      </button>
    </div>
  );
}

describe('LanguageProvider', () => {
  beforeEach(() => {
    mockTauriCommand('get_application_info', async () => ({
      name: 'CodeCompass',
      version: '0.1.1',
      buildTimestamp: '2026-01-01T00:00:00Z',
    }));
  });

  it('provides default English translations', () => {
    render(
      <LanguageProvider>
        <TestComponent />
      </LanguageProvider>,
    );
    expect(screen.getByTestId('home-title')).toHaveTextContent('CodeCompass');
    expect(screen.getByTestId('lang')).toHaveTextContent('en');
  });

  it('switches to Chinese when setLang is called', async () => {
    const user = userEvent.setup();
    render(
      <LanguageProvider>
        <TestComponent />
      </LanguageProvider>,
    );
    await user.click(screen.getByRole('button', { name: /switch/i }));
    expect(screen.getByTestId('lang')).toHaveTextContent('zh');
  });
});
