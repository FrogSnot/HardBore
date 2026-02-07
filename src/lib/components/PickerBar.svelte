<script lang="ts">
  import { pickerConfig, pickerSelection, confirmPickerSelection, cancelPicker, saveName, isSaveMode, currentPath } from '$lib/store';

  $: selectionCount = $pickerSelection.size;
  $: hasSelection = selectionCount > 0;
  $: saveMode = $isSaveMode;
  $: modeText = $pickerConfig?.mode === 'Files' ? 'files' : $pickerConfig?.mode === 'Directories' ? 'directories' : 'items';
  $: multipleAllowed = $pickerConfig?.allow_multiple ?? false;
  $: hasSaveName = $saveName.trim().length > 0;

  function handleSaveNameKeydown(e: KeyboardEvent) {
    e.stopPropagation();
    if (e.key === 'Enter') {
      e.preventDefault();
      confirmPickerSelection();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      cancelPicker();
    }
  }

  function handleSaveNameInput(e: Event) {
    const target = e.target as HTMLInputElement;
    saveName.set(target.value);
  }
</script>

<div class="picker-bar">
  {#if saveMode}
    <div class="picker-info">
      <span class="picker-label">SAVE FILE</span>
    </div>
    <div class="save-input-wrapper">
      <span class="save-path mono text-muted">{$currentPath}/</span>
      <input
        class="save-input mono"
        type="text"
        value={$saveName}
        oninput={handleSaveNameInput}
        onkeydown={handleSaveNameKeydown}
        placeholder="filename.ext"
        spellcheck="false"
      />
    </div>
    <div class="picker-actions">
      <button
        class="picker-btn cancel"
        onclick={cancelPicker}
        type="button"
      >
        Cancel (Esc)
      </button>
      <button
        class="picker-btn confirm"
        class:disabled={!hasSaveName}
        disabled={!hasSaveName}
        onclick={confirmPickerSelection}
        type="button"
      >
        Save (Enter)
      </button>
    </div>
  {:else}
    <div class="picker-info">
      <span class="picker-label">SELECT MODE</span>
      <span class="picker-details text-muted">
        {#if selectionCount === 0}
          {#if multipleAllowed}
            Click to select {modeText} • Multiple selection enabled
          {:else}
            Click to select a {modeText.slice(0, -1)}
          {/if}
        {:else}
          {selectionCount} {modeText} selected
        {/if}
      </span>
    </div>
    <div class="picker-actions">
      <button 
        class="picker-btn cancel"
        onclick={cancelPicker}
        type="button"
      >
        Cancel (Esc)
      </button>
      <button 
        class="picker-btn confirm"
        class:disabled={!hasSelection}
        disabled={!hasSelection}
        onclick={confirmPickerSelection}
        type="button"
      >
        Select{selectionCount > 0 ? ` (${selectionCount})` : ''} (Enter)
      </button>
    </div>
  {/if}
</div>

<style>
  .picker-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-md);
    background: var(--zinc-matte);
    border-bottom: 2px solid var(--safety-orange);
    box-shadow: var(--shadow-inset-sm);
    min-height: 56px;
  }

  .picker-info {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .picker-label {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--safety-orange);
    text-transform: uppercase;
  }

  .picker-details {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .picker-actions {
    display: flex;
    gap: var(--spacing-sm);
  }

  .picker-btn {
    padding: var(--spacing-sm) var(--spacing-lg);
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border: 1px solid var(--zinc-border);
    transition: all 0.1s;
    box-shadow: var(--shadow-inset-sm);
  }

  .picker-btn.cancel {
    background: var(--basalt-deep);
    color: var(--text-secondary);
  }

  .picker-btn.cancel:hover {
    background: var(--zinc-surface);
    color: var(--text-primary);
  }

  .picker-btn.confirm {
    background: var(--safety-orange);
    color: #fff;
    border-color: var(--safety-orange);
    font-weight: 600;
  }

  .picker-btn.confirm:hover:not(:disabled) {
    background: #ff6b3d;
    border-color: #ff6b3d;
  }

  .picker-btn.confirm:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .save-input-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0;
    background: var(--basalt-deep);
    border: 1px solid var(--zinc-border);
    border-radius: var(--radius-sm);
    padding: 0 var(--spacing-sm);
    box-shadow: var(--shadow-inset-sm);
    overflow: hidden;
  }

  .save-path {
    font-size: 12px;
    white-space: nowrap;
    flex-shrink: 0;
    max-width: 40%;
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl;
    text-align: left;
  }

  .save-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-size: 13px;
    padding: var(--spacing-sm) var(--spacing-xs);
    outline: none;
    min-width: 100px;
  }

  .save-input::placeholder {
    color: var(--text-dim);
  }
</style>
