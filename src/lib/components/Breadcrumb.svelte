<script lang="ts">
  import { currentPath, parentPath, navigateTo, navigateUp, history, historyIndex, navigateBack, navigateForward, createDirectory } from '$lib/store';
  import { getPathRoot, getPathSegments, buildPath } from '$lib/utils';

  let editMode = false;
  let inputValue = '';
  let inputEl: HTMLInputElement;
  
  let showCreateDialog = false;
  let newDirName = '';
  let createInputEl: HTMLInputElement;

  function startEdit() {
    inputValue = $currentPath;
    editMode = true;
    setTimeout(() => inputEl?.focus(), 0);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      navigateTo(inputValue);
      editMode = false;
    } else if (e.key === 'Escape') {
      editMode = false;
    }
  }

  function handleBlur() {
    editMode = false;
  }

  function openCreateDialog() {
    newDirName = '';
    showCreateDialog = true;
    setTimeout(() => createInputEl?.focus(), 0);
  }

  function closeCreateDialog() {
    showCreateDialog = false;
    newDirName = '';
  }

  async function handleCreateDirectory() {
    if (newDirName.trim()) {
      await createDirectory($currentPath, newDirName.trim());
      closeCreateDialog();
    }
  }

  function handleCreateKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleCreateDirectory();
    } else if (e.key === 'Escape') {
      closeCreateDialog();
    }
  }

  $: pathRoot = getPathRoot($currentPath);
  $: segments = getPathSegments($currentPath);
  $: canGoBack = $historyIndex > 0;
  $: canGoForward = $historyIndex < $history.length - 1;
</script>

<div class="breadcrumb-bar">
  <div class="nav-buttons">
    <button 
      class="nav-btn icon-arrow-left" 
      onclick={navigateBack}
      disabled={!canGoBack}
      title="Back (Alt+←)"
    >
    </button>
    <button 
      class="nav-btn icon-arrow-right" 
      onclick={navigateForward}
      disabled={!canGoForward}
      title="Forward (Alt+→)"
    >
    </button>
    <button 
      class="nav-btn icon-arrow-up" 
      onclick={navigateUp}
      disabled={!$parentPath}
      title="Up (Backspace)"
    >
    </button>
  </div>

  {#if editMode}
    <input
      bind:this={inputEl}
      class="path-input carved-input"
      type="text"
      bind:value={inputValue}
      onkeydown={handleKeydown}
      onblur={handleBlur}
      spellcheck="false"
    />
  {:else}
    <div class="path-display mono" onclick={startEdit} role="button" tabindex="0" onkeydown={(e) => e.key === 'Enter' && startEdit()}>
      <button class="segment root" data-drop-path={pathRoot} onclick={(e) => { e.stopPropagation(); navigateTo(pathRoot); }} type="button">{pathRoot}</button>
      {#each segments as segment, i}
        <button 
          class="segment"
          data-drop-path={buildPath(pathRoot, segments, i + 1)}
          onclick={(e) => { e.stopPropagation(); navigateTo(buildPath(pathRoot, segments, i + 1)); }}
          type="button"
        >
          {segment}
        </button>
        {#if i < segments.length - 1}
          <span class="separator">/</span>
        {/if}
      {/each}
    </div>
  {/if}
  
  <button 
    class="create-btn"
    onclick={openCreateDialog}
    title="Create New Directory"
  >
    + DIR
  </button>
</div>

{#if showCreateDialog}
  <div class="dialog-overlay" onclick={closeCreateDialog}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <h3>Create New Directory</h3>
      <input
        bind:this={createInputEl}
        class="dialog-input"
        type="text"
        bind:value={newDirName}
        onkeydown={handleCreateKeydown}
        placeholder="Directory name"
        spellcheck="false"
      />
      <div class="dialog-actions">
        <button class="dialog-btn cancel" onclick={closeCreateDialog}>Cancel</button>
        <button class="dialog-btn create" onclick={handleCreateDirectory}>Create</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .breadcrumb-bar {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--zinc-matte);
    border-bottom: 1px solid var(--zinc-border);
    box-shadow: var(--shadow-inset-sm);
    min-height: 40px;
  }

  .nav-buttons {
    display: flex;
    gap: 2px;
  }

  .nav-btn {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    font-size: 14px;
    box-shadow: var(--shadow-inset-sm);
    transition: all 0.1s;
  }

  .nav-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--zinc-surface);
  }

  .nav-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .icon-arrow-left::before {
    content: '←';
  }

  .icon-arrow-right::before {
    content: '→';
  }

  .icon-arrow-up::before {
    content: '↑';
  }

  .path-display {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 2px;
    padding: var(--spacing-xs) var(--spacing-sm);
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    box-shadow: var(--shadow-inset-sm);
    font-size: 13px;
    color: var(--text-secondary);
    cursor: text;
    overflow-x: auto;
    white-space: nowrap;
  }

  .path-display::-webkit-scrollbar {
    height: 4px;
  }

  .segment {
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    transition: all 0.1s;
    background: none;
    border: none;
    font-family: inherit;
    font-size: inherit;
  }

  .segment:hover {
    color: var(--text-primary);
    background: var(--zinc-surface);
  }

  .segment:last-child {
    color: var(--text-primary);
    font-weight: 500;
  }

  .segment:global(.drop-highlight) {
    background: rgba(66, 165, 245, 0.2);
    color: #42a5f5;
    outline: 1px dashed rgba(66, 165, 245, 0.5);
  }

  .separator {
    color: var(--text-dim);
  }

  .path-input {
    flex: 1;
    height: 28px;
  }

  .create-btn {
    padding: 0 12px;
    height: 28px;
    display: flex;
    align-items: center;
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    box-shadow: var(--shadow-inset-sm);
    transition: all 0.1s;
  }

  .create-btn:hover {
    color: var(--text-primary);
    background: var(--zinc-surface);
    border-color: rgba(255, 255, 255, 0.12);
  }

  .dialog-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(25px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog {
    background: var(--zinc-matte);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: inset 0 1px 3px rgba(0, 0, 0, 0.4), 0 8px 24px rgba(0, 0, 0, 0.6);
    padding: 24px;
    min-width: 400px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .dialog h3 {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: 0.02em;
  }

  .dialog-input {
    width: 100%;
    padding: 8px 12px;
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    color: var(--text-primary);
    font-size: 13px;
    font-family: 'JetBrains Mono', monospace;
    box-shadow: var(--shadow-inset-sm);
  }

  .dialog-input:focus {
    outline: none;
    border-color: rgba(255, 255, 255, 0.2);
  }

  .dialog-actions {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .dialog-btn {
    padding: 8px 16px;
    font-size: 13px;
    cursor: pointer;
    border: 1px solid var(--zinc-border);
    transition: all 0.1s;
  }

  .dialog-btn.cancel {
    background: var(--basalt-deep);
    color: var(--text-secondary);
  }

  .dialog-btn.cancel:hover {
    background: var(--zinc-surface);
    color: var(--text-primary);
  }

  .dialog-btn.create {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text-primary);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .dialog-btn.create:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
  }
</style>
