export function formatDatabasePath(path: string): string {
  return path.replace(
    /^[A-Za-z]:\\Users\\[^\\]+\\AppData\\Roaming\\/i,
    '%APPDATA%\\',
  );
}
