export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  is_symlink: boolean;
  size: number;
  modified: number;
  permissions: string;
  owner: number;
  group: number;
  extension: string | null;
  hidden: boolean;
}

export interface DirectoryContents {
  path: string;
  parent: string | null;
  entries: FileEntry[];
  total_items: number;
  total_size: number;
}

export interface SearchResult {
  name: string;
  path: string;
  is_dir: boolean;
  hidden: boolean;
  score: number;
}

export interface IndexerStatus {
  is_running: boolean;
  indexed_count: number;
  current_path: string | null;
  elapsed_ms: number;
}

export type PreviewType = 'Code' | 'Image' | 'Hex' | 'Auto';

export interface FilePreview {
  path: string;
  preview_type: PreviewType;
  size: number;
  text_content: string | null;
  hex_content: string | null;
  truncated: boolean;
  extension: string | null;
}

export interface MountPoint {
  name: string;
  path: string;
  device: string;
  fs_type: string;
}

export type SortField = 'name' | 'size' | 'modified' | 'extension';
export type SortDirection = 'asc' | 'desc';

export interface SortConfig {
  field: SortField;
  direction: SortDirection;
}

export interface ViewConfig {
  showHidden: boolean;
  sort: SortConfig;
  previewOpen: boolean;
  sidebarOpen: boolean;
}

export type PickerMode = 'Disabled' | 'Files' | 'Directories' | 'Both' | 'Save';

export interface PickerConfig {
  mode: PickerMode;
  allow_multiple: boolean;
  file_types: string[] | null;
  start_dir: string | null;
  current_name: string | null;
}
