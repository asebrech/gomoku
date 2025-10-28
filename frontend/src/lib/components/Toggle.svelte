<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    checked?: boolean;
    label?: string;
    id?: string;
    disabled?: boolean;
    onchange?: (checked: boolean) => void;
  }
  
  let {
    checked = $bindable(false),
    label = '',
    id = '',
    disabled = false,
    onchange
  }: Props = $props();
  
  function handleToggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<div class="flex items-center gap-3">
  {#if label}
    <label 
      for={id} 
      class="text-base font-medium text-white cursor-pointer select-none"
      class:opacity-50={disabled}
    >
      {label}
    </label>
  {/if}
  
  <button
    {id}
    role="switch"
    aria-checked={checked}
    aria-label={label || 'Toggle switch'}
    {disabled}
    onclick={handleToggle}
    class="toggle-switch relative inline-flex h-7 w-14 items-center rounded-full transition-all duration-300 ease-in-out outline-none focus:outline-none disabled:opacity-50 disabled:cursor-not-allowed"
    style="
      background: {checked ? $currentTheme.gradientPrimary : '#333'};
      box-shadow: {checked ? $currentTheme.glowPrimary : 'inset 0 2px 4px rgba(0, 0, 0, 0.3)'};
      border: 2px solid {checked ? $currentTheme.primary : '#555'};
    "
  >
    <span
      class="toggle-thumb inline-block h-5 w-5 transform rounded-full bg-white transition-transform duration-300 ease-in-out"
      style="
        transform: translateX({checked ? '1.85rem' : '0.25rem'});
        box-shadow: 0 2px 6px rgba(0, 0, 0, 0.4);
      "
    ></span>
  </button>
</div>

<style>
  .toggle-switch:hover:not(:disabled) {
    transform: scale(1.05);
  }
  
  .toggle-switch:active:not(:disabled) {
    transform: scale(0.98);
  }
  
  .toggle-thumb {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }
</style>
