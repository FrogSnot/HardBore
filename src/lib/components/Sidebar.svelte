<script lang="ts">
  import { navigateTo, favorites, mountPoints, addFavorite, removeFavorite, currentPath, startIndexing } from '$lib/store';
  import type { MountPoint } from '$lib/types';

  let showContextMenu = false;
  let contextMenuPath = '';
  let contextMenuX = 0;
  let contextMenuY = 0;

  function handleFavoriteClick(path: string) {
    navigateTo(path);
  }

  function handleMountClick(mount: MountPoint) {
    navigateTo(mount.path);
  }

  function handleAddFavorite() {
    const path = $currentPath;
    if (path) {
      addFavorite(path);
    }
  }

  function handleRightClick(event: MouseEvent, path: string) {
    event.preventDefault();
    contextMenuPath = path;
    contextMenuX = event.clientX;
    contextMenuY = event.clientY;
    showContextMenu = true;
  }

  function handleRemoveFavorite() {
    if (contextMenuPath) {
      removeFavorite(contextMenuPath);
    }
    showContextMenu = false;
  }

  async function handleIndexDirectory() {
    if (contextMenuPath) {
      await startIndexing(contextMenuPath, 10);
    }
    showContextMenu = false;
  }

  function closeContextMenu() {
    showContextMenu = false;
  }

  function getDisplayName(path: string): string {
    const parts = path.split('/').filter(Boolean);
    return parts[parts.length - 1] || path;
  }

  function isFavorite(path: string): boolean {
    return $favorites.includes(path);
  }
</script>

<svelte:window on:click={closeContextMenu} />

<aside class="sidebar">
  <div class="sidebar-section">
    <div class="section-header">
      <span class="section-title">FAVORITES</span>
      <button 
        class="add-favorite-btn" 
        on:click={handleAddFavorite}
        title="Add current directory to favorites"
      >+</button>
    </div>
    <div class="favorites-list">
      {#if $favorites.length === 0}
        <div class="empty-state">No favorites</div>
      {:else}
        {#each $favorites as fav}
          <button 
            class="sidebar-item"
            class:active={$currentPath === fav}
            on:click={() => handleFavoriteClick(fav)}
            on:contextmenu={(e) => handleRightClick(e, fav)}
          >
            <span class="item-icon">★</span>
            <span class="item-name">{getDisplayName(fav)}</span>
          </button>
        {/each}
      {/if}
    </div>
  </div>

  <div class="sidebar-section">
    <div class="section-header">
      <span class="section-title">DEVICES</span>
    </div>
    <div class="devices-list">
      {#if $mountPoints.length === 0}
        <div class="empty-state">No devices</div>
      {:else}
        {#each $mountPoints as mount}
          <button 
            class="sidebar-item"
            class:active={$currentPath === mount.path}
            on:click={() => handleMountClick(mount)}
            on:contextmenu={(e) => handleRightClick(e, mount.path)}
            title={mount.device}
          >
            <span class="item-icon">◊</span>
            <span class="item-name">{mount.name}</span>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</aside>

{#if showContextMenu}
  <div 
    class="context-menu" 
    style="left: {contextMenuX}px; top: {contextMenuY}px;"
  >
    <button class="context-item" on:click={handleIndexDirectory}>
      Index Directory
    </button>
    <button class="context-item" on:click={handleRemoveFavorite}>
      Remove from Favorites
    </button>
  </div>
{/if}

<style>
  .sidebar {
    display: flex;
    flex-direction: column;
    width: 220px;
    background: var(--zinc);
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    overflow-y: auto;
    flex-shrink: 0;
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    padding: 16px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  }

  .sidebar-section:last-child {
    border-bottom: none;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px 8px 12px;
  }

  .section-title {
    font-family: 'JetBrains Mono', monospace;
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.05em;
    color: var(--zinc-muted);
    text-transform: uppercase;
  }

  .add-favorite-btn {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.08);
    color: var(--zinc-primary);
    font-size: 14px;
    cursor: pointer;
    transition: all 0.05s;
  }

  .add-favorite-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.15);
  }

  .favorites-list,
  .devices-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sidebar-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: transparent;
    border: none;
    color: var(--zinc-primary);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.05s;
    border-left: 2px solid transparent;
  }

  .sidebar-item:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .sidebar-item.active {
    background: rgba(255, 255, 255, 0.06);
    border-left-color: var(--safety-orange);
  }

  .item-icon {
    font-size: 12px;
    color: var(--zinc-muted);
    flex-shrink: 0;
  }

  .item-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .empty-state {
    padding: 8px 12px;
    font-size: 12px;
    color: var(--zinc-muted);
    font-style: italic;
  }

  .context-menu {
    position: fixed;
    background: var(--zinc);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3), 0 4px 12px rgba(0, 0, 0, 0.5);
    z-index: 1000;
    min-width: 180px;
  }

  .context-item {
    display: block;
    width: 100%;
    padding: 8px 12px;
    background: transparent;
    border: none;
    color: var(--zinc-primary);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.05s;
  }

  .context-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }
</style>
