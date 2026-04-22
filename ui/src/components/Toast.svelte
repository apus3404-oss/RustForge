<script lang="ts">
  import { toasts } from '../lib/toast';
  import { fade, fly } from 'svelte/transition';

  function getIcon(type: string) {
    switch (type) {
      case 'success': return '✓';
      case 'error': return '✕';
      case 'warning': return '⚠';
      case 'info': return 'ℹ';
      default: return 'ℹ';
    }
  }
</script>

<div class="toast-container">
  {#each $toasts as toast (toast.id)}
    <div
      class="toast toast-{toast.type}"
      transition:fly={{ y: -20, duration: 300 }}
    >
      <span class="toast-icon">{getIcon(toast.type)}</span>
      <span class="toast-message">{toast.message}</span>
      <button
        class="toast-close"
        onclick={() => toasts.dismiss(toast.id)}
        aria-label="Dismiss"
      >
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    background: #1a1a1a;
    border: 1px solid #333;
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    min-width: 300px;
    max-width: 500px;
    pointer-events: auto;
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from {
      transform: translateX(100%);
      opacity: 0;
    }
    to {
      transform: translateX(0);
      opacity: 1;
    }
  }

  .toast-success {
    border-left: 3px solid #00ff88;
  }

  .toast-error {
    border-left: 3px solid #ff4444;
  }

  .toast-warning {
    border-left: 3px solid #fbbf24;
  }

  .toast-info {
    border-left: 3px solid #00d4ff;
  }

  .toast-icon {
    font-size: 1.25rem;
    font-weight: bold;
    flex-shrink: 0;
  }

  .toast-success .toast-icon {
    color: #00ff88;
  }

  .toast-error .toast-icon {
    color: #ff4444;
  }

  .toast-warning .toast-icon {
    color: #fbbf24;
  }

  .toast-info .toast-icon {
    color: #00d4ff;
  }

  .toast-message {
    flex: 1;
    color: #e5e7eb;
    font-size: 0.875rem;
    line-height: 1.4;
  }

  .toast-close {
    background: none;
    border: none;
    color: #999;
    font-size: 1.5rem;
    cursor: pointer;
    padding: 0;
    width: 1.5rem;
    height: 1.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: color 0.2s;
  }

  .toast-close:hover {
    color: #fff;
  }
</style>
