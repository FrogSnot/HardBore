<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { FileEntry } from '$lib/types';
  import { formatSize, formatDate, getFileIcon } from '$lib/utils';
  import { 
    selectedIndex, 
    entries, 
    enterSelected, 
    selectNext, 
    selectPrevious, 
    loadPreview, 
    viewConfig, 
    currentPath, 
    navigateTo,
    isPickerMode,
    pickerConfig,
    pickerSelection,
    togglePickerSelection,
    moveFile
  } from '$lib/store';
  import { get } from 'svelte/store';
  import ContextMenu from './ContextMenu.svelte';

  const ITEM_HEIGHT = 28;
  const OVERSCAN = 5;

  let container: HTMLDivElement;
  let scrollTop = 0;
  let containerHeight = 0;
  let mouseUsedRecently = false;
  let mouseTimeout: ReturnType<typeof setTimeout>;
  let lastKeyboardIndex = -1;

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuEntry: FileEntry | null = null;

  let draggedEntry: FileEntry | null = null;
  let dragOverEntry: FileEntry | null = null;

  $: totalHeight = $entries.length * ITEM_HEIGHT;
  $: visibleStart = Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - OVERSCAN);
  $: visibleEnd = Math.min($entries.length, Math.ceil((scrollTop + containerHeight) / ITEM_HEIGHT) + OVERSCAN);
  $: visibleItems = $entries.slice(visibleStart, visibleEnd).map((entry, i) => ({
    entry,
    index: visibleStart + i,
    top: (visibleStart + i) * ITEM_HEIGHT
  }));

  function handleScroll(e: Event) {
    const target = e.target as HTMLDivElement;
    scrollTop = target.scrollTop;
  }

  function handleMouseMove() {
    mouseUsedRecently = true;
    clearTimeout(mouseTimeout);
    mouseTimeout = setTimeout(() => {
      mouseUsedRecently = false;
    }, 500);
  }

  function handleClick(index: number, entry: FileEntry) {
    mouseUsedRecently = true;
    selectedIndex.set(index);
    
    if ($isPickerMode) {
      const config = get(pickerConfig);
      if (config?.mode === 'Files' && entry.is_dir) return;
      if (config?.mode === 'Directories' && !entry.is_dir) return;
      
      togglePickerSelection(entry.path);
    } else if (!entry.is_dir) {
      loadPreview(entry.path);
      viewConfig.update(c => ({ ...c, previewOpen: true }));
    }
  }

  function handleDoubleClick(index: number, entry: FileEntry) {
    mouseUsedRecently = true;
    selectedIndex.set(index);
    enterSelected();
  }

  function handleRightClick(e: MouseEvent, index: number, entry: FileEntry) {
    e.preventDefault();
    e.stopPropagation();
    mouseUsedRecently = true;
    selectedIndex.set(index);
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuEntry = entry;
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    contextMenuEntry = null;
  }

  async function refreshDirectory() {
    const path = get(currentPath);
    if (path) {
      await navigateTo(path, false);
    }
  }

  function handleDragStart(e: DragEvent, entry: FileEntry) {
    if (!e.dataTransfer) return;
    draggedEntry = entry;
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', entry.path);
    
    const target = e.target as HTMLElement;
    target.style.opacity = '0.5';
  }

  function handleDragEnd(e: DragEvent) {
    const target = e.target as HTMLElement;
    target.style.opacity = '1';
    draggedEntry = null;
    dragOverEntry = null;
  }

  function handleDragOver(e: DragEvent, entry: FileEntry) {
    if (!draggedEntry || !entry.is_dir) return;
    if (draggedEntry.path === entry.path) return;
    
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    dragOverEntry = entry;
  }

  function handleDragLeave(e: DragEvent, entry: FileEntry) {
    if (dragOverEntry?.path === entry.path) {
      dragOverEntry = null;
    }
  }

  async function handleDrop(e: DragEvent, targetEntry: FileEntry) {
    e.preventDefault();
    e.stopPropagation();
    
    if (!targetEntry.is_dir || !draggedEntry) {
      dragOverEntry = null;
      return;
    }
    
    if (draggedEntry.path === targetEntry.path) {
      dragOverEntry = null;
      return;
    }
    
    try {
      const fileName = draggedEntry.path.split('/').pop();
      if (!fileName) return;
      
      const destination = `${targetEntry.path}/${fileName}`;
      await moveFile(draggedEntry.path, destination);
      await refreshDirectory();
    } catch (e) {
      console.error('Failed to move file:', e);
    } finally {
      dragOverEntry = null;
      draggedEntry = null;
    }
  }

  $: if (container && $entries.length > 0 && containerHeight > 0 && !mouseUsedRecently && $selectedIndex !== lastKeyboardIndex) {
    lastKeyboardIndex = $selectedIndex;
    const selectedTop = $selectedIndex * ITEM_HEIGHT;
    const selectedBottom = selectedTop + ITEM_HEIGHT;
    const viewportTop = scrollTop;
    const viewportBottom = scrollTop + containerHeight;
    
    if (selectedTop < viewportTop) {
      container.scrollTop = selectedTop;
    } else if (selectedBottom > viewportBottom) {
      container.scrollTop = selectedBottom - containerHeight;
    }
  }

  onMount(() => {
    const resizeObserver = new ResizeObserver((entries) => {
      containerHeight = entries[0].contentRect.height;
    });
    resizeObserver.observe(container);
    
    return () => resizeObserver.disconnect();
  });
</script>

<div class="file-list-container" bind:this={container} onscroll={handleScroll} onmousemove={handleMouseMove} role="region" aria-label="File list">
  <div class="file-list-scroll" style="height: {totalHeight}px">
    {#each visibleItems as { entry, index, top } (entry.path)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="file-row"
        class:selected={index === $selectedIndex}
        class:picker-selected={$pickerSelection.has(entry.path)}
        class:directory={entry.is_dir}
        class:hidden-file={entry.hidden}
        class:drag-over={dragOverEntry?.path === entry.path}
        style="transform: translateY({top}px)"
        onclick={() => handleClick(index, entry)}
        ondblclick={() => handleDoubleClick(index, entry)}
        oncontextmenu={(e) => handleRightClick(e, index, entry)}
        draggable="true"
        ondragstart={(e) => handleDragStart(e, entry)}
        ondragend={handleDragEnd}
        ondragover={(e) => handleDragOver(e, entry)}
        ondragleave={(e) => handleDragLeave(e, entry)}
        ondrop={(e) => handleDrop(e, entry)}
        role="row"
        tabindex="-1"
      >
        <span class="file-icon mono">{getFileIcon(entry)}</span>
        <span class="file-name truncate" class:symlink={entry.is_symlink}>
          {entry.name}
        </span>
        <span class="file-permissions mono text-dim">{entry.permissions}</span>
        <span class="file-size mono text-muted">
          {entry.is_dir ? '—' : formatSize(entry.size)}
        </span>
        <span class="file-date mono text-muted">{formatDate(entry.modified)}</span>
      </div>
    {/each}
  </div>
  
  {#if $entries.length === 0}
    <div class="empty-state">
      <span class="text-dim">Empty directory</span>
    </div>
  {/if}
</div>

<ContextMenu 
  bind:visible={contextMenuVisible}
  x={contextMenuX}
  y={contextMenuY}
  entry={contextMenuEntry}
  onClose={closeContextMenu}
  onRefresh={refreshDirectory}
/>

<style>
  .file-list-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    background: var(--basalt);
    position: relative;
    min-height: 0;
  }

  .file-list-scroll {
    position: relative;
    width: 100%;
  }

  .file-row {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 28px;
    display: grid;
    grid-template-columns: 24px 1fr 100px 80px 120px;
    align-items: center;
    padding: 0 var(--spacing-md);
    gap: var(--spacing-sm);
    cursor: default;
    border-bottom: 1px solid transparent;
  }

  .file-row:hover {
    background: var(--zinc-matte);
  }

  .file-row.selected {
    background: var(--selection-bg);
    border-bottom-color: var(--zinc-border);
    border-top: 1px solid var(--zinc-border);
  }

  .file-row.picker-selected {
    background: rgba(255, 87, 34, 0.15);
    border-left: 2px solid var(--safety-orange);
  }

  .file-row.picker-selected.selected {
    background: rgba(255, 87, 34, 0.2);
  }

  .file-row.drag-over {
    background: rgba(66, 165, 245, 0.2);
    border: 1px dashed var(--text-secondary);
  }

  .file-icon {
    font-size: 11px;
    color: var(--text-muted);
    text-align: center;
  }

  .file-row.directory .file-icon {
    color: var(--text-secondary);
  }

  .file-name {
    font-size: 13px;
    color: var(--text-primary);
  }

  .file-row.directory .file-name {
    font-weight: 500;
  }

  .file-name.symlink {
    font-style: italic;
    color: var(--text-secondary);
  }

  .file-row.hidden-file .file-name {
    color: var(--text-muted);
  }

  .file-permissions,
  .file-size,
  .file-date {
    font-size: 11px;
    text-align: right;
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    font-size: 13px;
  }
</style>
