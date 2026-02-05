<script lang="ts">
  import { previewFile, previewLoading, viewConfig, togglePreview } from '$lib/store';
  import { formatSize, getLanguage } from '$lib/utils';
  import { convertFileSrc } from '@tauri-apps/api/core';

  let imageError = false;
  let imageLoaded = false;

  $: preview = $previewFile;
  $: loading = $previewLoading;
  $: isOpen = $viewConfig.previewOpen;

  $: assetUrl = preview ? convertFileSrc(preview.path) : '';

  $: if (preview) {
    imageError = false;
    imageLoaded = false;
  }

  function handleImageError() {
    imageError = true;
  }

  function handleImageLoad() {
    imageLoaded = true;
  }
</script>

{#if isOpen}
  <aside class="preview-pane">
    <div class="preview-header">
      <span class="preview-title truncate mono">
        {preview ? preview.path.split('/').pop() : 'No file selected'}
      </span>
      <button class="close-btn icon-close" onclick={togglePreview} title="Close preview (p)" type="button" aria-label="Close preview">
      </button>
    </div>

    <div class="preview-content">
      {#if loading}
        <div class="preview-loading">
          <span class="text-dim mono">Reading...</span>
        </div>
      {:else if preview}
        <div class="preview-meta">
          <span class="meta-item">
            <span class="meta-label text-dim">Size:</span>
            <span class="meta-value mono">{formatSize(preview.size)}</span>
          </span>
          {#if preview.extension}
            <span class="meta-item">
              <span class="meta-label text-dim">Type:</span>
              <span class="meta-value mono">.{preview.extension}</span>
            </span>
          {/if}
          {#if preview.truncated}
            <span class="meta-item truncated-notice">
              <span class="text-muted">Preview truncated</span>
            </span>
          {/if}
        </div>

        {#if preview.preview_type === 'Code' && preview.text_content}
          <div class="preview-code">
            <pre class="code-block mono"><code>{preview.text_content}</code></pre>
          </div>
        {:else if preview.preview_type === 'Hex' && preview.hex_content}
          <div class="preview-hex">
            <pre class="hex-block mono">{preview.hex_content}</pre>
          </div>
        {:else if preview.preview_type === 'Image'}
          <div class="preview-media">
            {#if imageError}
              <div class="media-error">
                <span class="error-icon icon-warning"></span>
                <span class="text-dim">Unable to load image</span>
              </div>
            {:else}
              {#if !imageLoaded}
                <div class="media-loading">
                  <span class="text-dim mono">Loading...</span>
                </div>
              {/if}
              <img 
                src={assetUrl} 
                alt={preview.path.split('/').pop()} 
                class="preview-img"
                class:loaded={imageLoaded}
                onerror={handleImageError}
                onload={handleImageLoad}
              />
            {/if}
          </div>
        {:else}
          <div class="preview-empty">
            <span class="text-dim">Unable to preview this file</span>
          </div>
        {/if}
      {:else}
        <div class="preview-empty">
          <span class="text-dim">Select a file to preview</span>
          <span class="text-muted" style="font-size: 11px; margin-top: 8px;">
            Press <span class="kbd">Enter</span> on a file
          </span>
        </div>
      {/if}
    </div>
  </aside>
{/if}

<style>
  .preview-pane {
    width: 100%;
    display: flex;
    flex-direction: column;
    background: var(--zinc-matte);
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--basalt-deep);
    border-bottom: 1px solid var(--zinc-border);
    min-height: 40px;
  }

  .preview-title {
    font-size: 12px;
    color: var(--text-secondary);
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
    content: '×';
  }

  .icon-warning::before {
    content: '!';
    font-weight: bold;
    font-style: normal;
  }

  .preview-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .preview-meta {
    display: flex;
    flex-wrap: wrap;
    gap: var(--spacing-md);
    padding: var(--spacing-sm) var(--spacing-md);
    background: var(--basalt);
    border-bottom: 1px solid var(--zinc-border);
    font-size: 11px;
  }

  .meta-item {
    display: flex;
    align-items: center;
    gap: var(--spacing-xs);
  }

  .truncated-notice {
    margin-left: auto;
    color: var(--warning-amber);
  }

  .preview-code,
  .preview-hex {
    flex: 1;
    overflow: auto;
    padding: var(--spacing-md);
  }

  .code-block,
  .hex-block {
    margin: 0;
    font-size: 12px;
    line-height: 1.6;
    color: var(--text-secondary);
    white-space: pre;
    tab-size: 4;
  }

  .hex-block {
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-muted);
  }

  .preview-loading,
  .preview-empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-xs);
    font-size: 13px;
  }

  .preview-media {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--spacing-md);
    overflow: hidden;
    position: relative;
    background: var(--basalt-deep);
    isolation: isolate;
    z-index: 2;
  }

  .preview-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
    border-radius: var(--radius-sm);
    opacity: 0;
    transition: opacity 150ms ease;
  }

  .preview-img.loaded {
    opacity: 1;
  }

  .media-loading {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .media-error {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--spacing-sm);
    padding: var(--spacing-xl);
    color: var(--text-dim);
  }

  .error-icon {
    font-size: 24px;
    opacity: 0.5;
  }
</style>
