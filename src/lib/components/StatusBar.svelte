<script lang="ts">
  import { currentDir, viewConfig, toggleHidden, setSort, indexerStatus, errorMessage } from '$lib/store';
  import { formatSize } from '$lib/utils';
  import type { SortField } from '$lib/types';

  $: totalItems = $currentDir?.total_items ?? 0;
  $: totalSize = $currentDir?.total_size ?? 0;
  $: showHidden = $viewConfig.showHidden;
  $: sortField = $viewConfig.sort.field;
  $: sortDir = $viewConfig.sort.direction;
  $: indexing = $indexerStatus?.is_running ?? false;
  $: indexedCount = $indexerStatus?.indexed_count ?? 0;
  $: error = $errorMessage;

  const sortOptions: { field: SortField; label: string }[] = [
    { field: 'name', label: 'Name' },
    { field: 'size', label: 'Size' },
    { field: 'modified', label: 'Date' },
    { field: 'extension', label: 'Type' }
  ];
</script>

<footer class="status-bar">
  <div class="status-left">
    <span class="status-item mono">
      {totalItems} items
    </span>
    <span class="status-item mono text-muted">
      {formatSize(totalSize)}
    </span>
    {#if indexing}
      <span class="status-item indexing">
        <span class="indexing-dot"></span>
        Indexing... {indexedCount.toLocaleString()}
      </span>
    {/if}
    {#if error}
      <span class="status-item error">{error}</span>
    {/if}
  </div>

  <div class="status-right">
    <div class="sort-controls">
      {#each sortOptions as opt}
        <button 
          class="sort-btn"
          class:active={sortField === opt.field}
          onclick={() => setSort(opt.field)}
          type="button"
        >
          {opt.label}
          {#if sortField === opt.field}
            <span class="sort-indicator" class:asc={sortDir === 'asc'}></span>
          {/if}
        </button>
      {/each}
    </div>

    <button 
      class="hidden-toggle" 
      class:active={showHidden}
      onclick={toggleHidden}
      title="Toggle hidden files (Ctrl+H)"
      type="button"
    >
      <span class="hidden-icon" class:active={showHidden}></span> Hidden
    </button>
  </div>
</footer>

<style>
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-xs) var(--spacing-md);
    background: var(--zinc-matte);
    border-top: 1px solid var(--zinc-border);
    font-size: 11px;
    min-height: 28px;
  }

  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: var(--spacing-md);
  }

  .status-item {
    color: var(--text-muted);
  }

  .status-item.indexing {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    color: var(--text-secondary);
  }

  .indexing-dot {
    width: 6px;
    height: 6px;
    background: var(--success-green);
    border-radius: 50%;
    animation: pulse 1s ease-in-out infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.4; }
  }

  .status-item.error {
    color: var(--safety-orange);
  }

  .sort-controls {
    display: flex;
    gap: 2px;
  }

  .sort-btn {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 2px 6px;
    background: none;
    border: none;
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    font-family: inherit;
  }

  .sort-btn:hover {
    background: var(--zinc-surface);
    color: var(--text-secondary);
  }

  .sort-btn.active {
    color: var(--text-primary);
    background: var(--basalt-deep);
  }

  .sort-indicator {
    font-size: 10px;
    display: inline-block;
    width: 8px;
    height: 8px;
    position: relative;
  }

  .sort-indicator::before {
    content: '↓';
  }

  .sort-indicator.asc::before {
    content: '↑';
  }

  .hidden-icon {
    display: inline-block;
    width: 8px;
    height: 8px;
    border: 1px solid currentColor;
    border-radius: 50%;
  }

  .hidden-icon.active {
    background: currentColor;
  }

  .hidden-toggle {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
    padding: 2px 8px;
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    font-size: 11px;
    cursor: pointer;
    font-family: inherit;
  }

  .hidden-toggle:hover {
    color: var(--text-secondary);
  }

  .hidden-toggle.active {
    color: var(--text-primary);
    border-color: var(--text-muted);
  }
</style>
