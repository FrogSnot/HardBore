<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { 
    commandPaletteOpen, 
    searchQuery, 
    searchResults, 
    searchSelectedIndex,
    search,
    closeCommandPalette,
    jumpToSearchResult
  } from '$lib/store';
  import { debounce, truncatePath } from '$lib/utils';

  let inputEl: HTMLInputElement;

  const debouncedSearch = debounce((query: string) => search(query), 100);

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    searchQuery.set(target.value);
    debouncedSearch(target.value);
  }

  function handleKeydown(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Escape') {
      e.preventDefault();
      closeCommandPalette();
    } else if (e.key === 'ArrowDown' || (e.key === 'j' && e.ctrlKey)) {
      e.preventDefault();
      searchSelectedIndex.update(i => Math.min(i + 1, $searchResults.length - 1));
    } else if (e.key === 'ArrowUp' || (e.key === 'k' && e.ctrlKey)) {
      e.preventDefault();
      searchSelectedIndex.update(i => Math.max(i - 1, 0));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      jumpToSearchResult();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    e.stopPropagation();
    if (e.target === e.currentTarget) {
      closeCommandPalette();
    }
  }

  $: if ($commandPaletteOpen && inputEl) {
    inputEl.focus();
    inputEl.select();
  }
</script>

{#if $commandPaletteOpen}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="palette-overlay" onclick={handleBackdropClick}>
    <div class="palette-container">
      <div class="palette-input-wrapper">
        <span class="palette-icon icon-search"></span>
        <input
          bind:this={inputEl}
          class="palette-input mono"
          type="text"
          placeholder="Search files..."
          value={$searchQuery}
          oninput={handleInput}
          onkeydown={handleKeydown}
          spellcheck="false"
        />
        <span class="palette-hint">
          <span class="kbd">↑↓</span> navigate
          <span class="kbd">↵</span> open
          <span class="kbd">esc</span> close
        </span>
      </div>

      {#if $searchResults.length > 0}
        <div class="palette-results">
          {#each $searchResults as result, i}
            <button
              class="result-item"
              class:selected={i === $searchSelectedIndex}
              onclick={() => { searchSelectedIndex.set(i); jumpToSearchResult(); }}
              type="button"
            >
              <div class="result-content">
                <div class="result-header">
                  <span class="result-icon" class:is-dir={result.is_dir}></span>
                  <span class="result-name">{result.name}</span>
                  {#if result.hidden}
                    <span class="result-badge hidden-badge">hidden</span>
                  {/if}
                  <span class="result-type text-dim">{result.is_dir ? 'directory' : 'file'}</span>
                </div>
                <div class="result-path mono text-dim">{result.path}</div>
              </div>
            </button>
          {/each}
        </div>
      {:else if $searchQuery.length >= 2}
        <div class="palette-empty">
          <span class="text-dim">No results found</span>
        </div>
      {:else if $searchQuery.length > 0}
        <div class="palette-empty">
          <span class="text-dim">Type at least 2 characters to search</span>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .palette-overlay {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 15vh;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(25px);
    -webkit-backdrop-filter: blur(25px);
  }

  .palette-container {
    width: 100%;
    max-width: 600px;
    background: var(--zinc-matte);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-md);
    box-shadow: 
      0 24px 48px rgba(0, 0, 0, 0.4),
      var(--shadow-inset-sm);
    overflow: hidden;
  }

  .palette-input-wrapper {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-md) var(--spacing-lg);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--zinc-border);
  }

  .palette-icon {
    color: var(--text-dim);
    font-size: 14px;
    width: 16px;
    height: 16px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .icon-search::before {
    content: '⌕';
  }

  .palette-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 15px;
    outline: none;
  }

  .palette-input::placeholder {
    color: var(--text-dim);
  }

  .palette-hint {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
    font-size: 11px;
    color: var(--text-dim);
  }

  .palette-results {
    max-height: 400px;
    overflow-y: auto;
    padding: var(--spacing-xs) 0;
  }

  .result-item {
    width: 100%;
    padding: var(--spacing-md) var(--spacing-lg);
    background: none;
    border: none;
    border-left: 2px solid transparent;
    cursor: pointer;
    text-align: left;
    transition: background 80ms ease;
  }

  .result-item:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .result-item.selected {
    background: var(--selection-bg);
    border-left-color: var(--text-muted);
  }

  .result-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .result-header {
    display: flex;
    align-items: center;
    gap: var(--spacing-sm);
  }

  .result-icon {
    font-size: 11px;
    color: var(--text-muted);
    width: 12px;
    height: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .result-icon::before {
    content: '○';
  }

  .result-icon.is-dir::before {
    content: '▸';
  }

  .result-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
  }

  .result-badge {
    padding: 2px 6px;
    border-radius: 3px;
    font-size: 9px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .hidden-badge {
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-dim);
    border: 1px solid var(--zinc-border);
  }

  .result-type {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-left: auto;
  }

  .result-path {
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-dim);
    padding-left: 19px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .palette-empty {
    padding: var(--spacing-xl);
    text-align: center;
    font-size: 13px;
  }
</style>
