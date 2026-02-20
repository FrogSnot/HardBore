<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import '../app.css';
  import Breadcrumb from '$lib/components/Breadcrumb.svelte';
  import FileList from '$lib/components/FileList.svelte';
  import CommandPalette from '$lib/components/CommandPalette.svelte';
  import PreviewPane from '$lib/components/PreviewPane.svelte';
  import StatusBar from '$lib/components/StatusBar.svelte';
  import ResizableDivider from '$lib/components/ResizableDivider.svelte';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import PickerBar from '$lib/components/PickerBar.svelte';
  import {
    initializeApp,
    updateIndexerStatus,
    openCommandPalette,
    closeCommandPalette,
    commandPaletteOpen,
    selectNext,
    selectPrevious,
    selectFirst,
    selectLast,
    enterSelected,
    navigateUp,
    navigateBack,
    navigateForward,
    toggleHidden,
    togglePreview,
    toggleSidebar,
    viewConfig,
    selectedEntry,
    selectedEntries,
    loadPreview,
    isPickerMode,
    pickerConfig,
    confirmPickerSelection,
    cancelPicker,
    copyToClipboard,
    cutToClipboard,
    pasteFromClipboard,
    deleteFile,
    currentPath
  } from '$lib/store';

  let statusInterval: ReturnType<typeof setInterval>;
  let previewWidth = 400;

  $: if ($viewConfig.previewOpen && !previewWidth) {
    previewWidth = 400;
  }

  function handlePreviewResize(newWidth: number) {
    previewWidth = newWidth;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ($isPickerMode) {
      if (e.key === 'Enter') {
        e.preventDefault();
        if ($pickerConfig?.mode === 'Save') {
          if ($selectedEntry?.is_dir) {
            enterSelected();
          } else {
            confirmPickerSelection();
          }
        } else {
          confirmPickerSelection();
        }
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        cancelPicker();
        return;
      }
    }

    if (e.ctrlKey && !e.shiftKey && !e.altKey) {
      if (e.key === 'c') {
        e.preventDefault();
        if ($selectedEntries.length > 0) {
          copyToClipboard($selectedEntries.map(e => e.path));
        }
        return;
      }
      if (e.key === 'x') {
        e.preventDefault();
        if ($selectedEntries.length > 0) {
          cutToClipboard($selectedEntries.map(e => e.path));
        }
        return;
      }
      if (e.key === 'v') {
        e.preventDefault();
        pasteFromClipboard();
        return;
      }
    }

    if (e.key === 'Delete' && $selectedEntries.length > 0 && !$commandPaletteOpen) {
      e.preventDefault();
      const names = $selectedEntries.map(e => e.name).join(', ');
      if (confirm(`Delete ${names}?`)) {
        for (const entry of $selectedEntries) {
          deleteFile(entry.path, entry.is_dir);
        }
      }
      return;
    }

    if (e.ctrlKey && e.key === 'p') {
      e.preventDefault();
      if ($commandPaletteOpen) {
        closeCommandPalette();
      } else {
        openCommandPalette();
      }
      return;
    }

    if ($commandPaletteOpen) return;

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        e.preventDefault();
        selectNext();
        break;
      case 'k':
      case 'ArrowUp':
        e.preventDefault();
        selectPrevious();
        break;
      case 'g':
        if (e.shiftKey) {
          e.preventDefault();
          selectLast();
        }
        break;
      case 'G':
        e.preventDefault();
        selectLast();
        break;
      case 'Enter':
      case 'l':
      case 'ArrowRight':
        e.preventDefault();
        if ($isPickerMode) {
          if ($selectedEntry?.is_dir) enterSelected();
        } else {
          enterSelected();
        }
        break;
      case 'h':
      case 'ArrowLeft':
      case 'Backspace':
        e.preventDefault();
        navigateUp();
        break;
      case 'H':
        if (e.ctrlKey) {
          e.preventDefault();
          toggleHidden();
        }
        break;
      case 'b':
        e.preventDefault();
        toggleSidebar();
        break;
      case 'p':
        e.preventDefault();
        togglePreview();
        if ($viewConfig.previewOpen && $selectedEntry && !$selectedEntry.is_dir) {
          loadPreview($selectedEntry.path);
        }
        break;
    }

    if (e.altKey) {
      if (e.key === 'ArrowLeft') {
        e.preventDefault();
        navigateBack();
      } else if (e.key === 'ArrowRight') {
        e.preventDefault();
        navigateForward();
      }
    }
  }

  onMount(async () => {
    await initializeApp();
    await getCurrentWindow().show();
    
    statusInterval = setInterval(updateIndexerStatus, 2000);
  });

  onDestroy(() => {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app-container">
  {#if $isPickerMode}
    <PickerBar />
  {/if}
  <Breadcrumb />
  
  <div class="main-content">
    {#if $viewConfig.sidebarOpen}
      <Sidebar />
    {/if}
    <FileList />
    {#if $viewConfig.previewOpen}
      <ResizableDivider 
        minSize={250} 
        maxSize={800} 
        defaultSize={400}
        onResize={handlePreviewResize}
      />
      <div class="preview-wrapper" style="width: {previewWidth}px;">
        <PreviewPane />
      </div>
    {/if}
  </div>
  
  <StatusBar />
</div>

<CommandPalette />

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background: var(--basalt);
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  .preview-wrapper {
    display: flex;
    flex-shrink: 0;
    min-width: 250px;
    max-width: 800px;
    overflow: hidden;
  }
</style>
