import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
} from 'react';
import type { ReactNode } from 'react';

import { en, zh } from './translations';

type Lang = 'en' | 'zh';

const STORAGE_KEY = 'codecompass-lang';

function loadLang(): Lang {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === 'zh' || stored === 'en') return stored;
  } catch {
    // localStorage not available
  }
  // Default to browser language
  const nav = navigator.language?.toLowerCase() ?? '';
  if (nav.startsWith('zh')) return 'zh';
  return 'en';
}

const translations = { en, zh } as const;

interface LangCtx {
  lang: Lang;
  t: typeof en;
  setLang: (l: Lang) => void;
}

const LangContext = createContext<LangCtx>({
  lang: 'en',
  t: en,
  setLang: () => {},
});

export function LanguageProvider({
  children,
}: {
  children: ReactNode;
}): JSX.Element {
  const [lang, setLangState] = useState<Lang>(loadLang);

  const setLang = useCallback((l: Lang) => {
    setLangState(l);
    try {
      localStorage.setItem(STORAGE_KEY, l);
    } catch {
      // ignore
    }
  }, []);

  const value = useMemo<LangCtx>(
    () => ({ lang, t: translations[lang], setLang }),
    [lang, setLang],
  );

  return <LangContext.Provider value={value}>{children}</LangContext.Provider>;
}

export function useT(): LangCtx {
  return useContext(LangContext);
}
