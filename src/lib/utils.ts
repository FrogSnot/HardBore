export function formatSize(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  if (unitIndex === 0) {
    return `${bytes} ${units[unitIndex]}`;
  }

  return `${size.toFixed(1)} ${units[unitIndex]}`;
}

const PATH_SEP_RE = /[\\/]/;

export function splitPath(path: string): string[] {
  return path.split(PATH_SEP_RE).filter(Boolean);
}

export function basename(path: string): string {
  const parts = splitPath(path);
  return parts[parts.length - 1] || path;
}

export function getPathRoot(path: string): string {
  if (path.startsWith('/')) return '/';
  const match = path.match(/^[A-Za-z]:[\\/]/);
  return match ? match[0] : '/';
}

export function getPathSegments(path: string): string[] {
  const root = getPathRoot(path);
  const rest = path.substring(root.length);
  return rest.split(PATH_SEP_RE).filter(Boolean);
}

export function buildPath(root: string, segments: string[], count: number): string {
  const joined = segments.slice(0, count).join('/');
  if (root === '/') return '/' + joined;
  return root + joined;
}

export function parentDir(path: string): string {
  const lastSep = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  if (lastSep < 0) return path;
  if (lastSep === 0) return '/';
  const root = getPathRoot(path);
  if (lastSep < root.length) return root;
  return path.substring(0, lastSep);
}

export function formatDate(timestamp: number): string {
  const date = new Date(timestamp * 1000);
  const now = new Date();
  
  if (date.getFullYear() === now.getFullYear()) {
    return date.toLocaleDateString('en-US', {
      month: 'short',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
      hour12: false
    }).replace(',', '');
  }
  
  return date.toLocaleDateString('en-US', {
    month: 'short',
    day: '2-digit',
    year: 'numeric'
  });
}

export function getFileIcon(entry: { is_dir: boolean; is_symlink: boolean; extension: string | null; name: string }): string {
  if (entry.is_dir) {
    return entry.is_symlink ? '\u2197' : '\u25b8';
  }
  
  const ext = entry.extension?.toLowerCase();
  
  if (['rs', 'py', 'js', 'ts', 'jsx', 'tsx', 'svelte', 'vue', 'go', 'c', 'cpp', 'h', 'java', 'kt', 'swift', 'rb', 'php'].includes(ext ?? '')) {
    return '\u25c7';
  }
  
  if (['json', 'yaml', 'yml', 'toml', 'xml', 'ini', 'conf'].includes(ext ?? '')) {
    return '\u25c6';
  }
  
  if (['md', 'txt', 'doc', 'docx', 'pdf'].includes(ext ?? '')) {
    return '\u25ab';
  }
  
  if (['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp', 'ico', 'bmp'].includes(ext ?? '')) {
    return '\u25aa';
  }
  
  if (['exe', 'bin', 'so', 'dylib', 'dll', 'o', 'a'].includes(ext ?? '')) {
    return '\u25cf';
  }
  
  if (['zip', 'tar', 'gz', 'bz2', 'xz', '7z', 'rar'].includes(ext ?? '')) {
    return '\u25c8';
  }
  
  if (entry.is_symlink) {
    return '\u2197';
  }
  
  return '\u25cb';
}

export function getLanguage(extension: string | null): string {
  const extMap: Record<string, string> = {
    'rs': 'rust',
    'py': 'python',
    'js': 'javascript',
    'ts': 'typescript',
    'jsx': 'javascript',
    'tsx': 'typescript',
    'svelte': 'svelte',
    'vue': 'html',
    'html': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'json': 'json',
    'yaml': 'yaml',
    'yml': 'yaml',
    'toml': 'toml',
    'xml': 'xml',
    'md': 'markdown',
    'sh': 'bash',
    'bash': 'bash',
    'zsh': 'bash',
    'c': 'c',
    'cpp': 'cpp',
    'h': 'c',
    'hpp': 'cpp',
    'go': 'go',
    'java': 'java',
    'kt': 'kotlin',
    'swift': 'swift',
    'rb': 'ruby',
    'php': 'php',
    'sql': 'sql',
    'lua': 'lua',
    'dockerfile': 'dockerfile',
    'makefile': 'makefile'
  };
  
  return extMap[extension?.toLowerCase() ?? ''] ?? 'plaintext';
}

export function truncatePath(path: string, maxLength: number): string {
  if (path.length <= maxLength) return path;
  
  const parts = splitPath(path);
  let result = parts[parts.length - 1];
  
  for (let i = parts.length - 2; i >= 0; i--) {
    const newResult = parts[i] + '/' + result;
    if (newResult.length > maxLength - 3) {
      return '.../' + result;
    }
    result = newResult;
  }
  
  return result;
}


export function debounce<T extends (...args: Parameters<T>) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Check if a key event is a modifier key only
 */
export function isModifierOnly(e: KeyboardEvent): boolean {
  return ['Control', 'Alt', 'Shift', 'Meta'].includes(e.key);
}

/**
 * Get key binding string
 */
export function getKeyString(e: KeyboardEvent): string {
  const parts: string[] = [];
  
  if (e.metaKey) parts.push('\u2318');
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  
  if (!isModifierOnly(e)) {
    parts.push(e.key.length === 1 ? e.key.toUpperCase() : e.key);
  }
  
  return parts.join('+');
}
