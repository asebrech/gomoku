<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    unit?: string;
    showInput?: boolean;
    id?: string;
    onchange?: (value: number) => void;
  }
  
  let {
    value = $bindable(0),
    min = 0,
    max = 100,
    step = 1,
    label = '',
    unit = '',
    showInput = true,
    id = '',
    onchange
  }: Props = $props();
  
  function handleChange(newValue: number) {
    value = newValue;
    onchange?.(value);
  }
</script>

<div class="flex flex-col gap-3">
  {#if label}
    <label for={id} class="flex justify-between items-center text-lg font-semibold text-white">
      {label}
      {#if unit}
        <span class="font-bold" style="color: {$currentTheme.secondary};">
          {value}{unit}
        </span>
      {/if}
    </label>
  {/if}
  
  <div class="flex items-center gap-4">
    <input
      {id}
      type="range"
      {min}
      {max}
      {step}
      bind:value
      onchange={() => handleChange(value)}
      class="flex-1 h-2 bg-white/20 rounded-full appearance-none cursor-pointer outline-none transition-all duration-200 hover:bg-white/25 slider-range"
      style="--slider-primary: {$currentTheme.primary}; --slider-secondary: {$currentTheme.secondary}; --slider-glow: {$currentTheme.glowPrimary};"
    />
    
    {#if showInput}
      <input
        type="number"
        {min}
        {max}
        {step}
        bind:value
        onchange={() => handleChange(value)}
        class="w-24 px-3 py-2 border rounded text-center font-medium outline-none transition-all duration-200 slider-number"
        style="background: {$currentTheme.background}33; border-color: {$currentTheme.primary}33; color: {$currentTheme.textPrimary};"
      />
    {/if}
  </div>
</div>

<style>
  /* Webkit (Chrome, Safari, Edge) */
  .slider-range::-webkit-slider-thumb {
    appearance: none;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--slider-primary), var(--slider-secondary));
    cursor: pointer;
    box-shadow: var(--slider-glow);
    transition: all 0.2s ease;
  }
  
  .slider-range::-webkit-slider-thumb:hover {
    transform: scale(1.15);
    box-shadow: 0 0 15px var(--slider-primary);
  }
  
  .slider-range::-webkit-slider-thumb:active {
    transform: scale(0.95);
  }
  
  /* Firefox */
  .slider-range::-moz-range-thumb {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--slider-primary), var(--slider-secondary));
    cursor: pointer;
    box-shadow: var(--slider-glow);
    border: none;
    transition: all 0.2s ease;
  }
  
  .slider-range::-moz-range-thumb:hover {
    transform: scale(1.15);
    box-shadow: 0 0 15px var(--slider-primary);
  }
  
  .slider-range::-moz-range-thumb:active {
    transform: scale(0.95);
  }
  
  /* Number input */
  .slider-number:focus {
    border-color: var(--slider-primary);
    box-shadow: 0 0 10px var(--slider-primary);
  }
  
  .slider-number::-webkit-inner-spin-button,
  .slider-number::-webkit-outer-spin-button {
    opacity: 1;
  }
</style>
