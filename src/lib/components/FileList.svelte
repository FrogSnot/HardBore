<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import type { FileEntry } from '$lib/types';
  import { formatSize, formatDate, getFileIcon, basename } from '$lib/utils';
  import { 
    selectedIndex, 
    selectedIndices,
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
    moveFile,
    copyFile,
    saveName,
    selectSingle,
    selectToggle,
    selectRange,
    selectedEntries,
    commandPaletteOpen
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
  let lastClickIndex = 0;

  let contextMenuVisible = false;
  let contextMenuX = 0;
  let contextMenuY = 0;
  let contextMenuEntry: FileEntry | null = null;
  let contextMenuAutoRename = false;

  let draggedEntry: FileEntry | null = null;
  let dropTargetPath: string | null = null;
  let _dropTargetEl: HTMLElement | null = null;

  let _mouseDown: { entry: FileEntry; x: number; y: number } | null = null;
  let _dragGhost: HTMLElement | null = null;
  const DRAG_THRESHOLD = 25;

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

  function handleClick(index: number, entry: FileEntry, e: MouseEvent) {
    mouseUsedRecently = true;
    
    if ($isPickerMode) {
      selectedIndex.set(index);
      const config = get(pickerConfig);
      if (config?.mode === 'Save') {
        if (!entry.is_dir) {
          saveName.set(entry.name);
        }
        return;
      }
      if (config?.mode === 'Files' && entry.is_dir) return;
      if (config?.mode === 'Directories' && !entry.is_dir) return;
      
      togglePickerSelection(entry.path);
      return;
    }

    if (e.shiftKey) {
      selectRange(lastClickIndex, index);
    } else if (e.ctrlKey || e.metaKey) {
      selectToggle(index);
    } else {
      selectSingle(index);
      lastClickIndex = index;
      if (!entry.is_dir) {
        loadPreview(entry.path);
        viewConfig.update(c => ({ ...c, previewOpen: true }));
      }
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

    const currentSelection = get(selectedIndices);
    if (!currentSelection.has(index)) {
      selectSingle(index);
    } else {
      selectedIndex.set(index);
    }

    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuEntry = entry;
    contextMenuVisible = true;
  }

  function closeContextMenu() {
    contextMenuVisible = false;
    contextMenuEntry = null;
    contextMenuAutoRename = false;
  }

  async function refreshDirectory() {
    const path = get(currentPath);
    if (path) {
      await navigateTo(path, false);
    }
  }

  function handleRowMouseDown(e: MouseEvent, entry: FileEntry) {
    if (e.button !== 0) return;
    _mouseDown = { entry, x: e.clientX, y: e.clientY };
  }

  function handleWindowMouseMove(e: MouseEvent) {
    if (!_mouseDown) return;

    const dx = e.clientX - _mouseDown.x;
    const dy = e.clientY - _mouseDown.y;

    if (!draggedEntry && (dx * dx + dy * dy) > DRAG_THRESHOLD * DRAG_THRESHOLD) {
      draggedEntry = _mouseDown.entry;
      const sel = get(selectedEntries);
      const dragCount = sel.length > 1 && sel.some(e => e.path === draggedEntry!.path) ? sel.length : 1;
      _dragGhost = document.createElement('div');
      _dragGhost.textContent = dragCount > 1
        ? `${dragCount} items`
        : `${getFileIcon(draggedEntry)} ${draggedEntry.name}`;
      Object.assign(_dragGhost.style, {
        position: 'fixed', zIndex: '10000', pointerEvents: 'none',
        padding: '4px 12px', borderRadius: '4px',
        background: '#2a2a2a', color: '#e0e0e0',
        fontSize: '13px', fontFamily: "'JetBrains Mono', monospace",
        border: '1px solid #3a3a3a', whiteSpace: 'nowrap',
        boxShadow: '0 4px 12px rgba(0,0,0,0.6)',
      });
      document.body.appendChild(_dragGhost);
      document.addEventListener('click', _suppressNextClick, { once: true, capture: true });
    }

    if (!draggedEntry || !_dragGhost) return;

    const zoom = parseFloat(getComputedStyle(document.documentElement).zoom) || 1;
    _dragGhost.style.left = `${e.clientX / zoom + 14}px`;
    _dragGhost.style.top = `${e.clientY / zoom + 14}px`;

    const el = document.elementFromPoint(e.clientX, e.clientY);
    const target = el?.closest('[data-drop-path]') as HTMLElement | null;
    const path = target?.dataset.dropPath ?? null;
    dropTargetPath = (path && path !== draggedEntry.path) ? path : null;

    if (_dropTargetEl && _dropTargetEl !== target) {
      _dropTargetEl.classList.remove('drop-highlight');
    }
    if (dropTargetPath && target) {
      target.classList.add('drop-highlight');
      _dropTargetEl = target;
    } else if (_dropTargetEl) {
      _dropTargetEl.classList.remove('drop-highlight');
      _dropTargetEl = null;
    }
  }

  async function handleWindowMouseUp(e: MouseEvent) {
    if (draggedEntry && dropTargetPath) {
      try {
        const sel = get(selectedEntries);
        const isCopy = e.ctrlKey;
        if (sel.length > 1 && sel.some(s => s.path === draggedEntry!.path)) {
          for (const entry of sel) {
            await performFileOperation(entry.path, dropTargetPath!, isCopy);
          }
        } else {
          await performFileOperation(draggedEntry.path, dropTargetPath, isCopy);
        }
      } finally {
        await refreshDirectory();
      }
    }
    _cleanupDrag();
  }

  function _suppressNextClick(e: Event) {
    e.preventDefault();
    e.stopPropagation();
  }

  async function performFileOperation(sourcePath: string, targetDir: string, isCopy: boolean) {
    const fileName = basename(sourcePath);
    if (!fileName) return;
    const destination = `${targetDir}/${fileName}`;
    if (sourcePath === destination) return;
    if (destination.startsWith(sourcePath + '/') || destination.startsWith(sourcePath + '\\')) return;

    if (isCopy) {
      await copyFile(sourcePath, destination);
    } else {
      await moveFile(sourcePath, destination);
    }
  }

  function _cleanupDrag() {
    _mouseDown = null;
    draggedEntry = null;
    dropTargetPath = null;
    if (_dropTargetEl) {
      _dropTargetEl.classList.remove('drop-highlight');
      _dropTargetEl = null;
    }
    if (_dragGhost) {
      _dragGhost.remove();
      _dragGhost = null;
    }
  }

  function handleContainerRightClick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('.file-row')) return;
    e.preventDefault();
    contextMenuX = e.clientX;
    contextMenuY = e.clientY;
    contextMenuEntry = null;
    contextMenuAutoRename = false;
    contextMenuVisible = true;
  }

  function handleF2(e: KeyboardEvent) {
    if (e.key !== 'F2') return;
    if (contextMenuVisible || get(commandPaletteOpen)) return;
    const idx = get(selectedIndex);
    const all = get(entries);
    if (idx < 0 || idx >= all.length) return;
    e.preventDefault();
    const entry = all[idx];
    contextMenuEntry = entry;
    const row = container?.querySelector('.file-row.focused');
    if (row) {
      const rect = row.getBoundingClientRect();
      contextMenuX = rect.left + 50;
      contextMenuY = rect.top + rect.height;
    } else {
      contextMenuX = 200;
      contextMenuY = 200;
    }
    contextMenuAutoRename = true;
    contextMenuVisible = true;
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
    window.addEventListener('keydown', handleF2);
    
    return () => {
      resizeObserver.disconnect();
      window.removeEventListener('keydown', handleF2);
    };
  });
</script>

<svelte:window onmousemove={handleWindowMouseMove} onmouseup={handleWindowMouseUp} />

<div class="file-list-container" bind:this={container} onscroll={handleScroll} onmousemove={handleMouseMove} oncontextmenu={handleContainerRightClick} role="region" aria-label="File list">
  <div class="file-list-scroll" style="height: {totalHeight}px">
    {#each visibleItems as { entry, index, top } (entry.path)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="file-row"
        class:selected={$selectedIndices.has(index)}
        class:focused={index === $selectedIndex}
        class:picker-selected={$pickerSelection.has(entry.path)}
        class:directory={entry.is_dir}
        class:hidden-file={entry.hidden}
        class:dragging={draggedEntry != null && (draggedEntry.path === entry.path || ($selectedIndices.has(index) && $selectedEntries.some(e => e.path === draggedEntry?.path)))}
        style="transform: translateY({top}px)"
        data-drop-path={entry.is_dir ? entry.path : null}
        onmousedown={(e) => handleRowMouseDown(e, entry)}
        onclick={(e) => handleClick(index, entry, e)}
        ondblclick={() => handleDoubleClick(index, entry)}
        oncontextmenu={(e) => handleRightClick(e, index, entry)}
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
  selectedEntries={$selectedEntries}
  autoRename={contextMenuAutoRename}
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
    user-select: none;
    -webkit-user-select: none;
  }

  .file-row:hover {
    background: var(--zinc-matte);
  }

  .file-row.selected {
    background: var(--selection-bg);
  }

  .file-row.focused {
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

  .file-row:global(.drop-highlight) {
    background: rgba(66, 165, 245, 0.15);
    border: 1px dashed rgba(66, 165, 245, 0.5);
    border-radius: var(--radius-sm);
  }

  .file-row:global(.drop-highlight) .file-icon {
    color: #42a5f5;
  }

  .file-row.dragging {
    opacity: 0.3;
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
