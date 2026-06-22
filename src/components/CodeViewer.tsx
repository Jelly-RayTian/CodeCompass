import Editor, { type OnMount } from '@monaco-editor/react';
import { useEffect, useRef, useState } from 'react';

import { tauriClient } from '@/lib/tauriClient';
import { LoadingState } from './LoadingState';

interface CodeViewerProps {
  workspaceId: number;
  filePath: string;
  focusLine: number | undefined;
  focusColumn: number | undefined;
  onFileLoaded?: (info: { language: string; totalLines: number }) => void;
}

export function CodeViewer({
  workspaceId,
  filePath,
  focusLine,
  focusColumn,
  onFileLoaded,
}: CodeViewerProps): JSX.Element {
  const [content, setContent] = useState<string | null>(null);
  const [language, setLanguage] = useState('plaintext');
  const [truncated, setTruncated] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const editorRef = useRef<Parameters<OnMount>[0] | null>(null);

  useEffect(() => {
    let cancelled = false;
    setError(null);
    setContent(null);
    const load = async (): Promise<void> => {
      try {
        const file = await tauriClient.readSourceFile(workspaceId, filePath);
        if (cancelled) return;
        setContent(file.content);
        setLanguage(file.language);
        setTruncated(file.truncated);
        onFileLoaded?.({
          language: file.language,
          totalLines: file.totalLines,
        });
      } catch (err) {
        if (!cancelled)
          setError(err instanceof Error ? err.message : String(err));
      }
    };
    load();
    return () => {
      cancelled = true;
    };
  }, [workspaceId, filePath, onFileLoaded]);

  useEffect(() => {
    if (
      editorRef.current === null ||
      focusLine === undefined ||
      focusLine === null
    )
      return;

    const editor = editorRef.current;
    editor.revealLineInCenter(focusLine);
    if (focusColumn !== undefined && focusColumn !== null) {
      const pos = {
        lineNumber: focusLine,
        column: focusColumn,
      };
      editor.setPosition(pos);
    }
  }, [focusLine, focusColumn, content]);

  const handleMount: OnMount = (editor) => {
    editorRef.current = editor;
    if (focusLine !== undefined && focusLine !== null) {
      editor.revealLineInCenter(focusLine);
    }
  };

  if (error !== null) {
    return (
      <div className="code-viewer-error banner banner-warning">{error}</div>
    );
  }

  if (content === null) {
    return <LoadingState label="Loading source…" />;
  }

  return (
    <div className="code-viewer">
      {truncated && (
        <div className="banner banner-warning">
          File too large — showing first 1 MB only.
        </div>
      )}
      <Editor
        height="100%"
        language={language}
        value={content}
        theme="vs-dark"
        onMount={handleMount}
        options={{
          readOnly: true,
          minimap: { enabled: true },
          lineNumbers: 'on',
          folding: true,
          scrollBeyondLastLine: false,
          wordWrap: 'off',
          automaticLayout: true,
          fontSize: 13,
        }}
      />
    </div>
  );
}
