<script lang="ts">
  import { getToasts, removeToast, subscribe, type Toast, type ToastType } from '$lib/toast.svelte.js';

  let toasts = $state<readonly Toast[]>(getToasts());

  // Subscribe to toast changes
  $effect(() => {
    const unsub = subscribe(() => {
      toasts = getToasts();
    });
    return unsub;
  });

  function iconFor(type: ToastType): string {
    switch (type) {
      case 'error': return '✕';
      case 'success': return '✓';
      case 'info': return 'ℹ';
    }
  }

  function classFor(type: ToastType): string {
    switch (type) {
      case 'error': return 'toast-error';
      case 'success': return 'toast-success';
      case 'info': return 'toast-info';
    }
  }
</script>

{#if toasts.length > 0}
  <div class="toast-container" aria-live="polite">
    {#each toasts as toast (toast.id)}
      <div class="toast {classFor(toast.type)}" role="alert">
        <span class="toast-icon">{iconFor(toast.type)}</span>
        <span class="toast-message">{toast.message}</span>
        <button class="toast-close" onclick={() => removeToast(toast.id)} aria-label="Dismiss">✕</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-container {
    position: fixed;
    bottom: 24px;
    right: 24px;
    z-index: 2000;
    display: flex;
    flex-direction: column-reverse;
    gap: 8px;
    max-width: 400px;
  }

  .toast {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 16px;
    background: var(--bg-secondary);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 13px;
    color: var(--fg-primary);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    animation: slideIn 0.2s ease-out;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .toast-error { border-left: 3px solid var(--danger); }
  .toast-success { border-left: 3px solid var(--success); }
  .toast-info { border-left: 3px solid var(--accent); }

  .toast-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .toast-error .toast-icon { color: var(--danger); }
  .toast-success .toast-icon { color: var(--success); }
  .toast-info .toast-icon { color: var(--accent); }

  .toast-message {
    flex: 1;
    line-height: 1.4;
  }

  .toast-close {
    background: transparent;
    border: none;
    color: var(--fg-muted);
    cursor: pointer;
    font-size: 12px;
    padding: 2px 4px;
    flex-shrink: 0;
  }

  .toast-close:hover { color: var(--fg-primary); }
</style>
