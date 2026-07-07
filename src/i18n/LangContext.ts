import { createContext } from 'react';

import { en } from './translations';
import type { Lang } from './types';

export interface LangCtx {
  lang: Lang;
  t: typeof en;
  setLang: (l: Lang) => void;
}

export const LangContext = createContext<LangCtx>({
  lang: 'en',
  t: en,
  setLang: () => {},
});
