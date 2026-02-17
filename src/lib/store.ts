import { writable, derived, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { 
  FileEntry, 
  DirectoryContents, 
  SearchResult, 
  IndexerStatus, 
  FilePreview,
  ViewConfig,
  SortField,
  SortDirection,
  MountPoint,
  PickerConfig
} from './types';
import { splitPath, parentDir } from './utils';

export const currentDir = writable<DirectoryContents | null>(null);
export const history = writable<string[]>([]);
export const historyIndex = writable<number>(-1);
export const selectedIndex = writable<number>(0);
export const selectedIndices = writable<Set<number>>(new Set([0]));
export const previewFile = writable<FilePreview | null>(null);
export const previewLoading = writable<boolean>(false);
export const commandPaletteOpen = writable<boolean>(false);
export const searchQuery = writable<string>('');
export const searchResults = writable<SearchResult[]>([]);
export const searchSelectedIndex = writable<number>(0);
export const indexerStatus = writable<IndexerStatus | null>(null);
export const favorites = writable<string[]>([]);
export const mountPoints = writable<MountPoint[]>([]);
export const pickerConfig = writable<PickerConfig | null>(null);
export const pickerSelection = writable<Set<string>>(new Set());
export const isPickerMode = derived(pickerConfig, $config => $config?.mode !== 'Disabled');
export const saveName = writable<string>('');
export const isSaveMode = derived(pickerConfig, $config => $config?.mode === 'Save');

export interface ClipboardItem {
  paths: string[];
  operation: 'copy' | 'cut';
}
export const clipboard = writable<ClipboardItem | null>(null);

function loadViewConfig(): ViewConfig {
  if (typeof window !== 'undefined') {
    try {
      const saved = localStorage.getItem('hardbore_view_config');
      if (saved) {
        const parsed = JSON.parse(saved);
        return {
          showHidden: parsed.showHidden ?? false,
          sort: parsed.sort ?? { field: 'name', direction: 'asc' },
          previewOpen: false,
          sidebarOpen: parsed.sidebarOpen ?? true
        };
      }
    } catch (e) {
      console.error('Failed to load view config:', e);
    }
  }
  return {
    showHidden: false,
    sort: { field: 'name', direction: 'asc' },
    previewOpen: false,
    sidebarOpen: true
  };
}

export const viewConfig = writable<ViewConfig>(loadViewConfig());
if (typeof window !== 'undefined') {
  viewConfig.subscribe($config => {
    try {
      localStorage.setItem('hardbore_view_config', JSON.stringify({
        showHidden: $config.showHidden,
        sort: $config.sort,
        sidebarOpen: $config.sidebarOpen
      }));
    } catch (e) {
      console.error('Failed to save view config:', e);
    }
  });
}

export const errorMessage = writable<string | null>(null);
export const currentPath = derived(currentDir, $dir => $dir?.path ?? '');
export const parentPath = derived(currentDir, $dir => $dir?.parent ?? null);

export const entries = derived(
  [currentDir, viewConfig],
  ([$dir, $config]) => {
    if (!$dir) return [];
    
    let items = [...$dir.entries];
    
    const { field, direction } = $config.sort;
    const multiplier = direction === 'asc' ? 1 : -1;
    
    items.sort((a, b) => {
      if (a.is_dir !== b.is_dir) {
        return a.is_dir ? -1 : 1;
      }
      
      let comparison = 0;
      switch (field) {
        case 'name':
          comparison = a.name.toLowerCase().localeCompare(b.name.toLowerCase());
          break;
        case 'size':
          comparison = a.size - b.size;
          break;
        case 'modified':
          comparison = a.modified - b.modified;
          break;
        case 'extension':
          const extA = a.extension ?? '';
          const extB = b.extension ?? '';
          comparison = extA.localeCompare(extB);
          break;
      }
      
      return comparison * multiplier;
    });
    
    return items;
  }
);

export const selectedEntry = derived(
  [entries, selectedIndex],
  ([$entries, $index]) => $entries[$index] ?? null
);

export const selectedEntries = derived(
  [entries, selectedIndices],
  ([$entries, $indices]) => {
    const result: FileEntry[] = [];
    for (const i of $indices) {
      if ($entries[i]) result.push($entries[i]);
    }
    return result;
  }
);

export function selectSingle(index: number): void {
  selectedIndex.set(index);
  selectedIndices.set(new Set([index]));
}

export function selectToggle(index: number): void {
  selectedIndex.set(index);
  selectedIndices.update(s => {
    const next = new Set(s);
    if (next.has(index)) {
      next.delete(index);
      if (next.size === 0) next.add(index);
    } else {
      next.add(index);
    }
    return next;
  });
}

export function selectRange(from: number, to: number): void {
  const lo = Math.min(from, to);
  const hi = Math.max(from, to);
  const next = new Set<number>();
  for (let i = lo; i <= hi; i++) next.add(i);
  selectedIndex.set(to);
  selectedIndices.set(next);
}

export async function navigateTo(path: string, addToHistory = true): Promise<void> {
  try {
    const config = get(viewConfig);
    const contents = await invoke<DirectoryContents>('read_dir', {
      path,
      showHidden: config.showHidden
    });
    
    currentDir.set(contents);
    selectedIndex.set(0);
    selectedIndices.set(new Set([0]));
    
    if (addToHistory) {
      const hist = get(history);
      const idx = get(historyIndex);
      
      const newHist = [...hist.slice(0, idx + 1), path];
      history.set(newHist);
      historyIndex.set(newHist.length - 1);
    }
    
    previewFile.set(null);
    errorMessage.set(null);
  } catch (e) {
    errorMessage.set(String(e));
  }
}

export async function navigateUp(): Promise<void> {
  const parent = get(parentPath);
  if (parent) {
    await navigateTo(parent);
  }
}

export async function navigateBack(): Promise<void> {
  const idx = get(historyIndex);
  const hist = get(history);
  
  if (idx > 0) {
    historyIndex.set(idx - 1);
    await navigateTo(hist[idx - 1], false);
  }
}

export async function navigateForward(): Promise<void> {
  const idx = get(historyIndex);
  const hist = get(history);
  
  if (idx < hist.length - 1) {
    historyIndex.set(idx + 1);
    await navigateTo(hist[idx + 1], false);
  }
}

export async function enterSelected(): Promise<void> {
  const entry = get(selectedEntry);
  if (entry?.is_dir) {
    await navigateTo(entry.path);
  } else if (entry) {
    try {
      await invoke('open_path', { path: entry.path });
    } catch (e) {
      errorMessage.set(`Failed to open file: ${e}`);
    }
  }
}

export function selectNext(): void {
  const items = get(entries);
  const newIndex = Math.min(get(selectedIndex) + 1, items.length - 1);
  selectedIndex.set(newIndex);
  selectedIndices.set(new Set([newIndex]));
}

export function selectPrevious(): void {
  const newIndex = Math.max(get(selectedIndex) - 1, 0);
  selectedIndex.set(newIndex);
  selectedIndices.set(new Set([newIndex]));
}

export function selectFirst(): void {
  selectedIndex.set(0);
  selectedIndices.set(new Set([0]));
}

export function selectLast(): void {
  const items = get(entries);
  const last = Math.max(items.length - 1, 0);
  selectedIndex.set(last);
  selectedIndices.set(new Set([last]));
}

export async function loadPreview(path: string): Promise<void> {
  previewLoading.set(true);
  try {
    const preview = await invoke<FilePreview>('preview_file', { path, maxBytes: 65536 });
    previewFile.set(preview);
  } catch (e) {
    previewFile.set(null);
    errorMessage.set(String(e));
  } finally {
    previewLoading.set(false);
  }
}

function calculateProximityScore(resultPath: string, currentPath: string): number {
  const currentParts = splitPath(currentPath);
  const resultParts = splitPath(resultPath);
  
  const resultDir = resultParts.slice(0, -1).join('/');
  const currentDirPath = currentParts.join('/');
  if (resultDir === currentDirPath) return 1000;
  
  let commonDepth = 0;
  for (let i = 0; i < Math.min(currentParts.length, resultParts.length - 1); i++) {
    if (currentParts[i] === resultParts[i]) {
      commonDepth++;
    } else {
      break;
    }
  }
  
  const currentDepth = currentParts.length;
  const resultDepth = resultParts.length - 1;
  const distance = Math.abs(currentDepth - commonDepth) + Math.abs(resultDepth - commonDepth);
  
  return Math.max(0, 100 - (distance * 10));
}

function calculateNameMatchScore(fileName: string, query: string): number {
  const lowerName = fileName.toLowerCase();
  const lowerQuery = query.toLowerCase();
  
  if (lowerName === lowerQuery) return 100;
  
  if (lowerName.startsWith(lowerQuery)) return 75;
  
  if (lowerName.includes(lowerQuery)) return 50;
  
  return 25;
}

export async function search(query: string): Promise<void> {
  searchQuery.set(query);
  
  if (query.length < 2) {
    searchResults.set([]);
    return;
  }
  
  try {
    const results = await invoke<SearchResult[]>('search_files', { query, limit: 100 });
    const currentDirPath = get(currentPath);
    
    const scoredResults = results.map(result => ({
      ...result,
      score: calculateProximityScore(result.path, currentDirPath) + 
             calculateNameMatchScore(result.name, query)
    }));
    
    scoredResults.sort((a, b) => b.score - a.score);
    
    searchResults.set(scoredResults.slice(0, 50));
    searchSelectedIndex.set(0);
  } catch (e) {
    searchResults.set([]);
  }
}

export function openCommandPalette(): void {
  commandPaletteOpen.set(true);
  searchQuery.set('');
  searchResults.set([]);
  searchSelectedIndex.set(0);
}

export function closeCommandPalette(): void {
  commandPaletteOpen.set(false);
  searchQuery.set('');
  searchResults.set([]);
}

export async function jumpToSearchResult(): Promise<void> {
  const results = get(searchResults);
  const idx = get(searchSelectedIndex);
  const result = results[idx];
  
  if (result) {
    closeCommandPalette();
    if (result.is_dir) {
      await navigateTo(result.path);
    } else {
      const parentPath = parentDir(result.path);
      await navigateTo(parentPath);
      
      const items = get(entries);
      const fileIndex = items.findIndex(e => e.path === result.path);
      if (fileIndex >= 0) {
        selectedIndex.set(fileIndex);
        await loadPreview(result.path);
        viewConfig.update(c => ({ ...c, previewOpen: true }));
      }
    }
  }
}

export function toggleHidden(): void {
  viewConfig.update(c => ({ ...c, showHidden: !c.showHidden }));
  const path = get(currentPath);
  if (path) {
    navigateTo(path, false);
  }
}

export function togglePreview(): void {
  viewConfig.update(c => ({ ...c, previewOpen: !c.previewOpen }));
}

export function setSort(field: SortField): void {
  viewConfig.update(c => ({
    ...c,
    sort: {
      field,
      direction: c.sort.field === field && c.sort.direction === 'asc' ? 'desc' : 'asc'
    }
  }));
}

export async function initializeApp(): Promise<void> {
  try {
    await invoke('init_indexer');
    await loadPickerConfig();
    await loadFavorites();
    await loadMountPoints();
    
    const config = get(pickerConfig);
    if (config?.current_name) {
      saveName.set(config.current_name);
    }
    
    const cwd = config?.start_dir || await invoke<string | null>('get_current_dir');
    const startPath = cwd || await invoke<string | null>('get_home');
    
    if (startPath) {
      await navigateTo(startPath);
      
      const indexedCount = await invoke<number>('get_indexed_count');
      if (indexedCount === 0) {
        invoke('start_indexing', { path: '/', maxDepth: null });
        
        const mounts = get(mountPoints);
        
        for (const mount of mounts) {
          invoke('start_indexing', { path: mount.path, maxDepth: null });
        }
      }
    }
  } catch (e) {
    errorMessage.set(String(e));
  }
}

export async function updateIndexerStatus(): Promise<void> {
  try {
    const status = await invoke<IndexerStatus | null>('get_indexer_status');
    indexerStatus.set(status);
  } catch {}
}

export function toggleSidebar(): void {
  viewConfig.update(c => ({ ...c, sidebarOpen: !c.sidebarOpen }));
}

export async function loadFavorites(): Promise<void> {
  try {
    const favs = await invoke<string[]>('get_favorites');
    favorites.set(favs);
  } catch (e) {
    console.error('Failed to load favorites:', e);
  }
}

export async function loadMountPoints(): Promise<void> {
  try {
    const mounts = await invoke<MountPoint[]>('get_mount_points');
    mountPoints.set(mounts);
  } catch (e) {
    console.error('Failed to load mount points:', e);
  }
}

export async function addFavorite(path: string): Promise<void> {
  try {
    await invoke('add_favorite', { path });
    await loadFavorites();
  } catch (e) {
    errorMessage.set(`Failed to add favorite: ${e}`);
  }
}

export async function removeFavorite(path: string): Promise<void> {
  try {
    await invoke('remove_favorite', { path });
    await loadFavorites();
  } catch (e) {
    errorMessage.set(`Failed to remove favorite: ${e}`);
  }
}

export async function createDirectory(parentPath: string, name: string): Promise<void> {
  try {
    const newPath = `${parentPath}/${name}`;
    await invoke('create_directory', { path: newPath });
    await navigateTo(parentPath, false);
  } catch (e) {
    errorMessage.set(`Failed to create directory: ${e}`);
  }
}

export async function loadPickerConfig(): Promise<void> {
  try {
    const config = await invoke<PickerConfig>('get_picker_config');
    pickerConfig.set(config);
  } catch (e) {
    console.error('Failed to load picker config:', e);
  }
}

export function togglePickerSelection(path: string): void {
  pickerSelection.update($selection => {
    const newSelection = new Set($selection);
    if (newSelection.has(path)) {
      newSelection.delete(path);
    } else {
      const config = get(pickerConfig);
      if (!config?.allow_multiple) {
        newSelection.clear();
      }
      newSelection.add(path);
    }
    return newSelection;
  });
}

export function clearPickerSelection(): void {
  pickerSelection.set(new Set());
}

export async function confirmPickerSelection(): Promise<void> {
  try {
    const config = get(pickerConfig);
    if (config?.mode === 'Save') {
      const name = get(saveName).trim();
      const dir = get(currentPath);
      if (name && dir) {
        const fullPath = `${dir}/${name}`;
        const exists = await invoke<boolean>('path_exists', { path: fullPath });
        if (exists && !confirm(`"${name}" already exists. Overwrite?`)) {
          return;
        }
        await invoke('select_files', { paths: [fullPath] });
      }
      return;
    }
    let selection = Array.from(get(pickerSelection));
    
    if (selection.length === 0) {
      const entry = get(selectedEntry);
      if (entry) {
        const mode = config?.mode;
        if (
          mode === 'Both' ||
          (mode === 'Files' && !entry.is_dir) ||
          (mode === 'Directories' && entry.is_dir)
        ) {
          selection = [entry.path];
        }
      }
    }
    
    if (selection.length > 0) {
      await invoke('select_files', { paths: selection });
    }
  } catch (e) {
    errorMessage.set(`Failed to select files: ${e}`);
  }
}

export async function cancelPicker(): Promise<void> {
  try {
    await invoke('cancel_picker');
  } catch (e) {
    console.error('Failed to cancel picker:', e);
  }
}

export async function startIndexing(path: string, maxDepth: number = 10): Promise<void> {
  try {
    await invoke('start_indexing', { path, maxDepth });
  } catch (e) {
    errorMessage.set(`Failed to start indexing: ${e}`);
  }
}

export async function deleteFile(path: string, isDir: boolean): Promise<void> {
  try {
    await invoke('delete_path', { path, isDir });
    const currentDirPath = get(currentPath);
    if (currentDirPath) {
      await navigateTo(currentDirPath, false);
    }
  } catch (e) {
    errorMessage.set(`Failed to delete: ${e}`);
    throw e;
  }
}

export async function renameFile(oldPath: string, newName: string): Promise<void> {
  try {
    await invoke('rename_path', { oldPath, newName });
    const currentDirPath = get(currentPath);
    if (currentDirPath) {
      await navigateTo(currentDirPath, false);
    }
  } catch (e) {
    errorMessage.set(`Failed to rename: ${e}`);
    throw e;
  }
}

export function copyToClipboard(paths: string[]): void {
  clipboard.set({ paths, operation: 'copy' });
}

export function cutToClipboard(paths: string[]): void {
  clipboard.set({ paths, operation: 'cut' });
}

export async function pasteFromClipboard(destinationDir?: string): Promise<void> {
  const clip = get(clipboard);
  if (!clip) return;
  
  const destDir = destinationDir || get(currentPath);
  if (!destDir) return;
  
  try {
    if (clip.operation === 'copy') {
      await invoke('batch_copy_paths', { sources: clip.paths, destinationDir: destDir });
    } else {
      await invoke('batch_move_paths', { sources: clip.paths, destinationDir: destDir });
      clipboard.set(null);
    }
    
    await navigateTo(destDir, false);
  } catch (e) {
    errorMessage.set(`Failed to paste: ${e}`);
    throw e;
  }
}

export async function moveFile(source: string, destination: string): Promise<void> {
  try {
    await invoke('move_path', { source, destination });
  } catch (e) {
    errorMessage.set(`Failed to move: ${e}`);
    throw e;
  }
}

export async function copyFile(source: string, destination: string): Promise<void> {
  try {
    await invoke('copy_path', { source, destination });
  } catch (e) {
    errorMessage.set(`Failed to copy: ${e}`);
    throw e;
  }
}

