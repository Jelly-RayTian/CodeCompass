import { useContext } from 'react';

import { LangContext } from './LangContext';

export function useT() {
  return useContext(LangContext);
}
