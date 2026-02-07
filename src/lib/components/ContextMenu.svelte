<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import type { FileEntry } from '$lib/types';
  import { 
    enterSelected, 
    navigateTo, 
    currentPath, 
    startIndexing,
    copyToClipboard,
    cutToClipboard,
    pasteFromClipboard,
    deleteFile,
    renameFile,
    clipboard
  } from '$lib/store';
  import { get } from 'svelte/store';
  
  export let x = 0;
  export let y = 0;
  export let visible = false;
  export let entry: FileEntry | null = null;
  export let onClose: () => void;
  export let onRefresh: () => void = () => {};

  let menuElement: HTMLDivElement;
  let menuX = 0;
  let menuY = 0;
  let renameMode = false;
  let renameValue = '';
  let renameInput: HTMLInputElement | undefined;
  let propertiesModal = false;
  let properties: FileProperties | null = null;
  let deleteModal = false;

  interface FileProperties {
    name: string;
    path: string;
    size: number;
    is_dir: boolean;
    is_symlink: boolean;
    readonly: boolean;
    created: number | null;
    modified: number | null;
    accessed: number | null;
  }

  interface MenuItem {
    label: string;
    icon: string;
    action: () => void;
    separator?: boolean;
    disabled?: boolean;
    danger?: boolean;
  }

  function handleClickOutside(e: MouseEvent) {
    if (menuElement && !menuElement.contains(e.target as Node)) {
      onClose();
    }
  }

  function handleEscape(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      if (renameMode) {
        renameMode = false;
      } else if (propertiesModal) {
        propertiesModal = false;
      } else if (deleteModal) {
        deleteModal = false;
      } else {
        onClose();
      }
    }
  }

  function copyPath() {
    if (entry) {
      navigator.clipboard.writeText(entry.path);
      onClose();
    }
  }

  function copyName() {
    if (entry) {
      navigator.clipboard.writeText(entry.name);
      onClose();
    }
  }

  function copyFilesToClipboard() {
    if (entry) {
      copyToClipboard([entry.path]);
      onClose();
    }
  }

  function cutFilesToClipboard() {
    if (entry) {
      cutToClipboard([entry.path]);
      onClose();
    }
  }

  async function pasteFiles() {
    try {
      const destDir = entry?.is_dir ? entry.path : get(currentPath);
      if (destDir) {
        await pasteFromClipboard(destDir);
        onRefresh();
      }
    } catch (e) {
      console.error('Failed to paste:', e);
    }
    onClose();
  }

  async function openItem() {
    if (entry) {
      if (entry.is_dir) {
        await navigateTo(entry.path);
      } else {
        await invoke('open_path', { path: entry.path });
      }
      onClose();
    }
  }

  async function showInFolder() {
    if (entry) {
      try {
        await invoke('show_in_folder', { path: entry.path });
      } catch (e) {
        console.error('Failed to show in folder:', e);
      }
      onClose();
    }
  }

  async function openTerminal() {
    if (entry) {
      try {
        await invoke('open_terminal', { path: entry.path });
      } catch (e) {
        console.error('Failed to open terminal:', e);
      }
      onClose();
    }
  }

  function startRename() {
    if (entry) {
      renameValue = entry.name;
      renameMode = true;
      setTimeout(() => {
        if (renameInput) {
          renameInput.focus();
          const dotIndex = renameValue.lastIndexOf('.');
          if (dotIndex > 0 && !entry?.is_dir) {
            renameInput.setSelectionRange(0, dotIndex);
          } else {
            renameInput.select();
          }
        }
      }, 10);
    }
  }

  async function confirmRename() {
    if (entry && renameValue && renameValue !== entry.name) {
      try {
        await renameFile(entry.path, renameValue);
        onRefresh();
      } catch (e) {
        alert(`Failed to rename: ${e}`);
      }
    }
    renameMode = false;
    onClose();
  }

  function handleRenameKeydown(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmRename();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      renameMode = false;
    }
  }

  async function deleteItem() {
    if (entry) {
      deleteModal = true;
    }
  }

  async function confirmDelete() {
    if (entry) {
      try {
        await deleteFile(entry.path, entry.is_dir);
        deleteModal = false;
        onRefresh();
        onClose();
      } catch (e) {
        alert(`Failed to delete: ${e}`);
      }
    }
  }

  function cancelDelete() {
    deleteModal = false;
    onClose();
  }

  async function showProperties() {
    if (entry) {
      try {
        properties = await invoke<FileProperties>('get_properties', { path: entry.path });
        propertiesModal = true;
      } catch (e) {
        console.error('Failed to get properties:', e);
      }
    }
  }

  function closeProperties() {
    propertiesModal = false;
    properties = null;
    onClose();
  }

  function formatSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function formatDate(timestamp: number | null): string {
    if (!timestamp) return '—';
    return new Date(timestamp * 1000).toLocaleString();
  }

  async function indexDirectory() {
    if (entry && entry.is_dir) {
      await startIndexing(entry.path, 10);
      onClose();
    }
  }

  $: menuItems = entry ? [
    { label: 'Open', icon: 'icon-enter', action: openItem, disabled: false },
    { label: '', icon: '', action: () => {}, separator: true },
    { label: 'Cut', icon: 'icon-scissors', action: cutFilesToClipboard, disabled: false },
    { label: 'Copy', icon: 'icon-copy', action: copyFilesToClipboard, disabled: false },
    { label: 'Paste', icon: 'icon-paste', action: pasteFiles, disabled: !$clipboard },
    { label: '', icon: '', action: () => {}, separator: true },
    ...(entry.is_dir ? [
      { label: 'Index Directory', icon: 'icon-lightning', action: indexDirectory, disabled: false },
      { label: '', icon: '', action: () => {}, separator: true },
    ] : []),
    { label: 'Copy Path', icon: 'icon-copy', action: copyPath, disabled: false },
    { label: 'Copy Name', icon: 'icon-copy', action: copyName, disabled: false },
    { label: '', icon: '', action: () => {}, separator: true },
    { label: 'Rename', icon: 'icon-edit', action: startRename, disabled: false },
    { label: 'Delete', icon: 'icon-trash', action: deleteItem, disabled: false, danger: true },
    { label: '', icon: '', action: () => {}, separator: true },
    { label: 'Show in Folder', icon: 'icon-folder', action: showInFolder, disabled: false },
    { label: 'Open Terminal Here', icon: 'icon-terminal', action: openTerminal, disabled: false },
    { label: '', icon: '', action: () => {}, separator: true },
    { label: 'Properties', icon: 'icon-info', action: showProperties, disabled: false },
  ] as MenuItem[] : [];

  $: if (visible) {
    // CSS zoom on <html> causes the right click menu to be misplaced, so we need to adjust the coordinates based on the zoom level
    const zoom = parseFloat(getComputedStyle(document.documentElement).zoom) || 1;
    menuX = x / zoom;
    menuY = y / zoom;

    if (menuElement) {
      const rect = menuElement.getBoundingClientRect();
      const viewportWidth = window.innerWidth / zoom;
      const viewportHeight = window.innerHeight / zoom;

      if (menuX + rect.width > viewportWidth) {
        menuX = viewportWidth - rect.width - 4;
      }
      if (menuY + rect.height > viewportHeight) {
        menuY = viewportHeight - rect.height - 4;
      }
    }
  }

  onMount(() => {
    document.addEventListener('click', handleClickOutside);
    document.addEventListener('keydown', handleEscape);
  });

  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
    document.removeEventListener('keydown', handleEscape);
  });
</script>

{#if visible && entry}
  {#if deleteModal}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="delete-overlay" onclick={cancelDelete}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_keys -->
      <div class="delete-dialog" onclick={(e) => e.stopPropagation()}>
        <div class="delete-header">
          <span class="delete-icon icon-trash"></span>
          <span class="delete-title">Delete {entry.is_dir ? 'Directory' : 'File'}</span>
        </div>
        <div class="delete-content">
          <div class="delete-item-name mono">{entry.name}</div>
          <div class="delete-warning">This action cannot be undone.</div>
        </div>
        <div class="delete-actions">
          <button class="delete-btn cancel" onclick={cancelDelete} type="button">Cancel</button>
          <button class="delete-btn confirm" onclick={confirmDelete} type="button">Delete</button>
        </div>
      </div>
    </div>
  {:else if renameMode}
    <div class="rename-overlay">
      <div class="rename-dialog">
        <div class="rename-header">Rename</div>
        <input
          bind:this={renameInput}
          class="rename-input mono"
          type="text"
          bind:value={renameValue}
          onkeydown={handleRenameKeydown}
          spellcheck="false"
        />
        <div class="rename-actions">
          <button class="rename-btn cancel" onclick={() => { renameMode = false; onClose(); }} type="button">Cancel</button>
          <button class="rename-btn confirm" onclick={confirmRename} type="button">Rename</button>
        </div>
      </div>
    </div>
  {:else if propertiesModal && properties}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="properties-overlay" onclick={closeProperties}>
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div class="properties-dialog" onclick={(e) => e.stopPropagation()}>
        <div class="properties-header">
          <span class="properties-icon" class:is-dir={properties.is_dir}></span>
          <span class="properties-title truncate">{properties.name}</span>
          <button class="close-btn icon-close" onclick={closeProperties} type="button" aria-label="Close"></button>
        </div>
        <div class="properties-content">
          <div class="prop-row">
            <span class="prop-label text-dim">Type</span>
            <span class="prop-value">{properties.is_dir ? 'Directory' : 'File'}{properties.is_symlink ? ' (Symlink)' : ''}</span>
          </div>
          <div class="prop-row">
            <span class="prop-label text-dim">Location</span>
            <span class="prop-value mono truncate">{properties.path}</span>
          </div>
          <div class="prop-row">
            <span class="prop-label text-dim">Size</span>
            <span class="prop-value mono">{formatSize(properties.size)}</span>
          </div>
          <div class="prop-separator"></div>
          <div class="prop-row">
            <span class="prop-label text-dim">Created</span>
            <span class="prop-value mono">{formatDate(properties.created)}</span>
          </div>
          <div class="prop-row">
            <span class="prop-label text-dim">Modified</span>
            <span class="prop-value mono">{formatDate(properties.modified)}</span>
          </div>
          <div class="prop-row">
            <span class="prop-label text-dim">Accessed</span>
            <span class="prop-value mono">{formatDate(properties.accessed)}</span>
          </div>
          <div class="prop-separator"></div>
          <div class="prop-row">
            <span class="prop-label text-dim">Read-only</span>
            <span class="prop-value">{properties.readonly ? 'Yes' : 'No'}</span>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div 
      bind:this={menuElement}
      class="context-menu"
      style="left: {menuX}px; top: {menuY}px;"
      role="menu"
      aria-label="Context menu"
    >
      <div class="context-header">
        <span class="context-icon" class:is-dir={entry.is_dir}></span>
        <span class="context-title truncate">{entry.name}</span>
      </div>
      
      {#each menuItems as item}
        {#if item.separator}
          <div class="menu-separator"></div>
        {:else}
          <button
            class="menu-item"
            class:disabled={item.disabled}
            class:danger={item.danger}
            onclick={item.action}
            disabled={item.disabled}
            type="button"
            role="menuitem"
          >
            <span class="menu-icon {item.icon}"></span>
            <span class="menu-label">{item.label}</span>
          </button>
        {/if}
      {/each}
    </div>
  {/if}
{/if}

<style>
  .context-menu {
    position: fixed;
    z-index: 2000;
    min-width: 220px;
    background: var(--zinc-matte);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    box-shadow: 
      0 12px 32px rgba(0, 0, 0, 0.5),
      var(--shadow-inset-sm);
    padding: var(--spacing-xs) 0;
    animation: contextFadeIn 120ms ease;
  }

  @keyframes contextFadeIn {
    from {
      opacity: 0;
      transform: scale(0.96);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .context-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: var(--spacing-sm) var(--spacing-md);
    margin-bottom: var(--spacing-xs);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--zinc-border);
  }

  .context-icon {
    font-size: 10px;
    color: var(--text-muted);
    width: 12px;
    height: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .context-icon::before {
    content: '○';
  }

  .context-icon.is-dir::before {
    content: '▸';
  }

  .context-title {
    font-size: 12px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .menu-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xs) var(--spacing-md);
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    font-size: 13px;
    color: var(--text-secondary);
    transition: background 80ms ease;
  }

  .menu-item:hover:not(.disabled) {
    background: var(--selection-bg);
    color: var(--text-primary);
  }

  .menu-item.danger {
    color: var(--safety-orange);
  }

  .menu-item.danger:hover:not(.disabled) {
    background: rgba(255, 87, 34, 0.15);
    color: var(--safety-orange);
  }

  .menu-item.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .menu-icon {
    width: 16px;
    height: 16px;
    display: inline-block;
    color: var(--text-muted);
    text-align: center;
    position: relative;
  }

  .icon-enter::before { content: '↵'; }
  .icon-scissors::before { content: '✂'; }
  .icon-copy::before { content: '□'; }
  .icon-paste::before { content: '▣'; }
  .icon-lightning::before { content: '⚡'; }
  .icon-edit::before { content: '✎'; }
  .icon-trash::before { content: '×'; color: var(--safety-orange); font-weight: bold; }
  .icon-folder::before { content: '▤'; }
  .icon-terminal::before { content: '›'; }
  .icon-info::before { content: 'i'; font-style: italic; font-weight: bold; }

  .menu-label {
    flex: 1;
  }

  .menu-separator {
    height: 1px;
    margin: var(--spacing-xs) var(--spacing-sm);
    background: var(--zinc-border);
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  /* Rename Dialog */
  .rename-overlay {
    position: fixed;
    inset: 0;
    z-index: 2100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
  }

  .rename-dialog {
    width: 400px;
    background: var(--zinc-matte);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-md);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .rename-header {
    padding: var(--spacing-md);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--zinc-border);
  }

  .rename-input {
    width: 100%;
    padding: var(--spacing-md);
    background: var(--basalt);
    border: none;
    border-bottom: 1px solid var(--zinc-border);
    color: var(--text-primary);
    font-size: 14px;
    outline: none;
  }

  .rename-input:focus {
    background: var(--basalt-deep);
  }

  .rename-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
  }

  .rename-btn {
    padding: var(--spacing-xs) var(--spacing-md);
    font-size: 13px;
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background 80ms ease;
  }

  .rename-btn.cancel {
    background: var(--zinc-matte);
    color: var(--text-secondary);
  }

  .rename-btn.cancel:hover {
    background: var(--zinc-surface);
  }

  .rename-btn.confirm {
    background: var(--text-muted);
    color: var(--basalt);
    border-color: var(--text-muted);
  }

  .rename-btn.confirm:hover {
    background: var(--text-primary);
  }

  /* Properties Dialog */
  .properties-overlay {
    position: fixed;
    inset: 0;
    z-index: 2100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(8px);
  }

  .properties-dialog {
    width: 450px;
    max-height: 80vh;
    background: var(--zinc-matte);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-md);
    box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .properties-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--zinc-border);
  }

  .properties-icon {
    font-size: 12px;
    color: var(--text-muted);
    width: 14px;
    height: 14px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .properties-icon::before {
    content: '○';
  }

  .properties-icon.is-dir::before {
    content: '▸';
  }

  .properties-title {
    flex: 1;
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .close-btn {
    width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-size: 12px;
  }

  .close-btn:hover {
    background: var(--zinc-surface);
    color: var(--text-primary);
  }

  .icon-close::before {
    content: '✕';
  }

  .properties-content {
    padding: var(--spacing-md);
    overflow-y: auto;
  }

  .prop-row {
    display: flex;
    align-items: flex-start;
    gap: var(--spacing-md);
    padding: var(--spacing-xs) 0;
  }

  .prop-label {
    width: 80px;
    flex-shrink: 0;
    font-size: 12px;
  }

  .prop-value {
    flex: 1;
    font-size: 12px;
    color: var(--text-primary);
    word-break: break-all;
  }

  .prop-separator {
    height: 1px;
    margin: var(--spacing-sm) 0;
    background: var(--zinc-border);
  }

  /* Delete Dialog */
  .delete-overlay {
    position: fixed;
    inset: 0;
    z-index: 2100;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(12px);
  }

  .delete-dialog {
    width: 380px;
    background: var(--zinc-matte);
    border: 1px solid var(--safety-orange);
    box-shadow: inset 0 0 0 1px rgba(255, 87, 34, 0.2),
                0 32px 64px rgba(0, 0, 0, 0.6);
  }

  .delete-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--safety-orange);
  }

  .delete-icon {
    font-size: 18px;
    color: var(--safety-orange);
    font-weight: bold;
  }

  .delete-icon::before {
    content: '⌫';
  }

  .delete-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-primary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .delete-content {
    padding: var(--spacing-lg) var(--spacing-md);
    display: flex;
    flex-direction: column;
    gap: var(--spacing-md);
  }

  .delete-item-name {
    font-size: 14px;
    color: var(--text-primary);
    padding: var(--spacing-sm);
    background: var(--basalt-deep);
    border-left: 2px solid var(--safety-orange);
  }

  .delete-warning {
    font-size: 12px;
    color: var(--text-muted);
  }

  .delete-actions {
    display: flex;
    gap: var(--spacing-sm);
    padding: var(--spacing-md);
    background: var(--basalt-deep);
    border-top: 1px solid var(--zinc-border);
  }

  .delete-btn {
    flex: 1;
    padding: var(--spacing-sm) var(--spacing-md);
    font-family: var(--font-ui);
    font-size: 13px;
    border: 1px solid var(--zinc-border);
    cursor: pointer;
    transition: all 80ms ease;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    font-weight: 500;
  }

  .delete-btn.cancel {
    background: var(--zinc-matte);
    color: var(--text-secondary);
  }

  .delete-btn.cancel:hover {
    background: var(--zinc-surface);
    color: var(--text-primary);
  }

  .delete-btn.confirm {
    background: var(--safety-orange);
    color: var(--basalt);
    border-color: var(--safety-orange);
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.1);
  }

  .delete-btn.confirm:hover {
    background: #ff6e40;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15);
  }
</style>
