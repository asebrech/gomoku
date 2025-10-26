<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    value?: number;
    onchange?: (value: number) => void;
    label?: string;
    muted?: boolean;
    ontoggle?: () => void;
  }
  
  let { 
    value = $bindable(50),
    onchange,
    label = 'Volume',
    muted = $bindable(false),
    ontoggle
  }: Props = $props();
  
  function handleChange(e: Event) {
    const target = e.target as HTMLInputElement;
    value = Number(target.value);
    onchange?.(value);
  }
  
  function toggleMute() {
    muted = !muted;
    ontoggle?.();
  }
  
  const volumeIcon = $derived(
    muted || value === 0 ? 'M' : 
    value < 50 ? 'V' : 'V+'
  );
</script>

<div class="flex items-center gap-4 p-4 rounded-lg backdrop-blur-md border transition-colors duration-300"
     style="background-color: {$currentTheme.surface}80; border-color: {$currentTheme.primary}4D;">
  <button 
    onclick={toggleMute}
    class="w-10 h-10 flex items-center justify-center rounded-lg transition-all duration-200 border"
    style="background-color: {$currentTheme.buttonNormal}; 
           border-color: {$currentTheme.primary}80;
           box-shadow: {$currentTheme.glowPrimary};"
    aria-label={muted ? 'Unmute' : 'Mute'}
  >
    <span class="font-bold text-lg" style="color: {$currentTheme.secondary};">
      {volumeIcon}
    </span>
  </button>
  
  <div class="flex-1">
    <div class="relative">
      {#if label}
        <div class="block text-sm font-medium mb-2" style="color: {$currentTheme.textPrimary};">
          {label}
        </div>
      {/if}
      
      <input 
        type="range" 
        min="0" 
        max="100" 
        bind:value 
        onchange={handleChange}
        disabled={muted}
        aria-label={label}
        class="volume-slider w-full h-2 rounded-full appearance-none cursor-pointer border
               disabled:opacity-50 disabled:cursor-not-allowed transition-opacity duration-200"
        style="background: linear-gradient(to right, {$currentTheme.background}, {$currentTheme.surface});
               border-color: {$currentTheme.primary}4D;"
      />
      
      <div 
        class="absolute top-0 left-0 h-2 rounded-full pointer-events-none transition-all duration-150"
        style="width: {muted ? 0 : value}%; 
               background: {$currentTheme.gradientAccent}; 
               box-shadow: {$currentTheme.glowAccent};"
      ></div>
    </div>
  </div>
  
  <div class="w-12 text-right">
    <span class="font-mono font-bold text-lg" style="color: {$currentTheme.textSecondary};">
      {muted ? '0' : value}
    </span>
  </div>
</div>

<style>
  /* Custom range input styling */
  .volume-slider::-webkit-slider-thumb {
    appearance: none;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: linear-gradient(135deg, #FF00FF 0%, #00FFFF 100%);
    border: 2px solid #FF33CC;
    cursor: pointer;
    box-shadow: 0 0 15px rgba(255, 51, 204, 0.8);
    transition: all 0.2s;
    position: relative;
    z-index: 10;
  }
  
  .volume-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 0 25px rgba(255, 51, 204, 1);
  }
  
  .volume-slider::-moz-range-thumb {
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: linear-gradient(135deg, #FF00FF 0%, #00FFFF 100%);
    border: 2px solid #FF33CC;
    cursor: pointer;
    box-shadow: 0 0 15px rgba(255, 51, 204, 0.8);
    transition: all 0.2s;
    position: relative;
    z-index: 10;
  }
  
  .volume-slider::-moz-range-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 0 25px rgba(255, 51, 204, 1);
  }
  
  .volume-slider:disabled::-webkit-slider-thumb,
  .volume-slider:disabled::-moz-range-thumb {
    background: #666;
    border-color: #444;
    box-shadow: none;
  }
</style>
