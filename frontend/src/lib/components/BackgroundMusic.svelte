<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { audioSettings } from '$lib/stores/audioSettings';
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  
  let audioElement: HTMLAudioElement | null = null;
  let showVolumeSlider = $state(false);
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
  let isPlaying = $state(false);
  
  onMount(() => {
    // Create audio element
    audioElement = new Audio('/menu-theme.mp3');
    audioElement.loop = true;
    audioElement.volume = $audioSettings.musicVolume;
    // Start muted - user must click to play
  });
  
  onDestroy(() => {
    if (audioElement) {
      audioElement.pause();
      audioElement = null;
    }
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
    }
  });
  
  function togglePlay() {
    if (!audioElement) return;
    
    if (isPlaying) {
      // Pause and reset
      audioElement.pause();
      audioElement.currentTime = 0;
      isPlaying = false;
    } else {
      // Play from beginning
      audioElement.currentTime = 0;
      audioElement.play().catch(e => console.log('Play failed:', e));
      isPlaying = true;
    }
  }
  
  function handleVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const volume = parseFloat(target.value);
    audioSettings.update(s => ({ ...s, musicVolume: volume }));
    if (audioElement) {
      audioElement.volume = volume;
    }
  }
  
  function handleMouseEnter() {
    if (hoverTimeout) clearTimeout(hoverTimeout);
    showVolumeSlider = true;
  }
  
  function handleMouseLeave() {
    hoverTimeout = setTimeout(() => {
      showVolumeSlider = false;
    }, 300);
  }
</script>

<div 
  class="fixed bottom-6 right-6 z-50 flex items-center gap-2"
  role="region"
  aria-label="Background music controls"
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  <!-- Volume Slider -->
  {#if showVolumeSlider}
    <div 
      class="flex items-center gap-2 px-3 py-2 rounded-lg backdrop-blur-md transition-all duration-300 animate-fade-in"
      style="background: {$currentTheme.background}cc; border: 2px solid {$currentTheme.primary}80;"
    >
      <span class="text-white/60 text-sm font-medium">Volume</span>
      <input
        type="range"
        min="0"
        max="1"
        step="0.01"
        value={$audioSettings.musicVolume}
        oninput={handleVolumeChange}
        onchange={handleVolumeChange}
        class="w-24 h-2 rounded-lg appearance-none cursor-pointer volume-slider"
        style="
          background: linear-gradient(to right, {$currentTheme.primary} 0%, {$currentTheme.primary} {$audioSettings.musicVolume * 100}%, rgba(255,255,255,0.2) {$audioSettings.musicVolume * 100}%, rgba(255,255,255,0.2) 100%);
          --thumb-color: {$currentTheme.primary};
        "
      />
      <span class="text-white/80 text-sm font-bold min-w-[2.5rem]" style="color: {$currentTheme.secondary};">
        {Math.round($audioSettings.musicVolume * 100)}%
      </span>
    </div>
  {/if}
  
  <!-- Play/Pause Button -->
  <div class="flex items-center justify-center">
    <Button variant="secondary" size="md" onclick={togglePlay}>
      {#if !isPlaying}
        <!-- Muted/Paused Icon -->
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-6 h-6" style="color: {$currentTheme.primary};">
          <path d="M11 5L6 9H2v6h4l5 4V5z"/>
          <line x1="23" y1="9" x2="17" y2="15"/>
          <line x1="17" y1="9" x2="23" y2="15"/>
        </svg>
      {:else if $audioSettings.musicVolume < 0.5}
        <!-- Low Volume Icon -->
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-6 h-6" style="color: {$currentTheme.primary};">
          <path d="M11 5L6 9H2v6h4l5 4V5z"/>
          <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
        </svg>
      {:else}
        <!-- High Volume Icon -->
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-6 h-6" style="color: {$currentTheme.primary};">
          <path d="M11 5L6 9H2v6h4l5 4V5z"/>
          <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/>
        </svg>
      {/if}
    </Button>
  </div>
</div>

<style>
  /* Custom range slider styling */
  .volume-slider::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--thumb-color, white);
    cursor: pointer;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4), 0 0 0 2px rgba(255, 255, 255, 0.3);
    transition: all 0.2s;
  }
  
  .volume-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5), 0 0 0 3px rgba(255, 255, 255, 0.4);
  }
  
  .volume-slider::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--thumb-color, white);
    cursor: pointer;
    border: 2px solid rgba(255, 255, 255, 0.3);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
    transition: all 0.2s;
  }
  
  .volume-slider::-moz-range-thumb:hover {
    transform: scale(1.2);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    border-color: rgba(255, 255, 255, 0.4);
  }
</style>
