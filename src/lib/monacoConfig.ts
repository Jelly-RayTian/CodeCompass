// Configures @monaco-editor/loader to use the locally bundled monaco-editor
// package instead of its default CDN (cdn.jsdelivr.net).
//
// Without this, @monaco-editor/react fetches the Monaco runtime from the
// network on first use, which would break CodeCompass's local-first /
// no-network privacy guarantee. Importing this module once before the
// <Editor> mounts keeps the editor fully offline.

import { loader } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker';
import jsonWorker from 'monaco-editor/esm/vs/language/json/json.worker?worker';
import cssWorker from 'monaco-editor/esm/vs/language/css/css.worker?worker';
import htmlWorker from 'monaco-editor/esm/vs/language/html/html.worker?worker';
import tsWorker from 'monaco-editor/esm/vs/language/typescript/ts.worker?worker';

// Register Vite-bundled web workers for each Monaco language service.
self.MonacoEnvironment = {
  getWorker(_workerId: string, label: string): Worker {
    switch (label) {
      case 'json':
        return new jsonWorker();
      case 'css':
      case 'scss':
      case 'less':
        return new cssWorker();
      case 'html':
      case 'handlebars':
      case 'razor':
        return new htmlWorker();
      case 'typescript':
      case 'javascript':
        return new tsWorker();
      default:
        return new editorWorker();
    }
  },
};

// Hand the already-bundled monaco instance to the React wrapper so it
// never reaches for the CDN.
loader.config({ monaco });

export { monaco };
