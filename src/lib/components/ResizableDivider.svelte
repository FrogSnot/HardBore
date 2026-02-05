<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export let minSize = 200;
  export let maxSize = 800;
  export let defaultSize = 400;
  export let onResize: (size: number) => void;

  let isDragging = false;
  let currentSize = defaultSize;

  function handleMouseDown(e: MouseEvent) {
    e.preventDefault();
    isDragging = true;
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isDragging) return;
    
    const newSize = window.innerWidth - e.clientX;
    currentSize = Math.max(minSize, Math.min(maxSize, newSize));
    onResize(currentSize);
  }

  function handleMouseUp() {
    if (isDragging) {
      isDragging = false;
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }
  }

  onMount(() => {
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  });

  onDestroy(() => {
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  });
</script>

<div 
  class="divider" 
  class:dragging={isDragging}
  onmousedown={handleMouseDown}
  role="separator"
  aria-orientation="vertical"
  aria-label="Resize panel"
>
  <div class="divider-handle"></div>
</div>

<style>
  .divider {
    width: 6px;
    background: transparent;
    cursor: col-resize;
    position: relative;
    flex-shrink: 0;
    transition: background 150ms ease;
  }

  .divider:hover,
  .divider.dragging {
    background: var(--zinc-border);
  }

  .divider-handle {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 2px;
    height: 40px;
    background: var(--text-dim);
    border-radius: 2px;
    opacity: 0;
    transition: opacity 150ms ease;
  }

  .divider:hover .divider-handle,
  .divider.dragging .divider-handle {
    opacity: 0.5;
  }

  .divider.dragging .divider-handle {
    opacity: 1;
  }
</style>
