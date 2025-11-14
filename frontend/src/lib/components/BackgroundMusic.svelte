<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { audioSettings } from '$lib/stores/audioSettings';
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  import { setBackgroundMusicElement } from '$lib/utils/soundEffects';
  import menuTheme from '$lib/assets/sound/menu-theme.mp3';
  
  let audioElement: HTMLAudioElement | null = null;
  let showVolumeSlider = $state(false);
  let hoverTimeout: ReturnType<typeof setTimeout> | null = null;
  
  onMount(() => {
    // Create audio element
    audioElement = new Audio(menuTheme);
    audioElement.loop = true;
    audioElement.volume = $audioSettings.musicVolume;
    
    // Share the audio element with sound effects for ducking
    setBackgroundMusicElement(audioElement);
    
    // Try to auto-play, but don't worry if it fails (browser autoplay policy)
    // Music will start on first user interaction (like clicking mute button)
    audioElement.play().catch(() => {
      // Silently fail - music will start on user interaction
    });
  });
  
  onDestroy(() => {
    if (audioElement) {
      audioElement.pause();
      audioElement = null;
    }
    if (hoverTimeout) {
      clearTimeout(hoverTimeout);
    }
    // Clear the reference
    setBackgroundMusicElement(null);
  });
  
  function toggleMute() {
    if (!audioElement) return;
    
    const currentMuteState = $audioSettings.isMuted;
    
    if (currentMuteState) {
      // Unmute: play music at stored volume
      audioSettings.update(s => ({ ...s, isMuted: false }));
      audioElement.volume = $audioSettings.musicVolume;
      audioElement.play().catch(() => {});
    } else {
      // Mute: pause music (volumes stay the same in store)
      audioSettings.update(s => ({ ...s, isMuted: true }));
      audioElement.volume = 0;
      audioElement.pause();
    }
  }
  
  function handleMusicVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const volume = parseFloat(target.value);
    audioSettings.update(s => ({ ...s, musicVolume: volume }));
    if (audioElement) {
      audioElement.volume = volume;
    }
    // If user is adjusting volume, they're unmuting
    if ($audioSettings.isMuted && volume > 0) {
      audioSettings.update(s => ({ ...s, isMuted: false }));
    }
  }
  
  function handleSfxVolumeChange(event: Event) {
    const target = event.target as HTMLInputElement;
    const volume = parseFloat(target.value);
    audioSettings.update(s => ({ ...s, sfxVolume: volume }));
    // If user is adjusting volume, they're unmuting
    if ($audioSettings.isMuted && volume > 0) {
      audioSettings.update(s => ({ ...s, isMuted: false }));
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
  
  // Reactive effect to sync audio element volume with store
  $effect(() => {
    if (audioElement) {
      audioElement.volume = $audioSettings.musicVolume;
    }
  });
</script>

<div 
  class="fixed bottom-6 right-6 z-50 flex items-center gap-2"
  role="region"
  aria-label="Audio controls"
  onmouseenter={handleMouseEnter}
  onmouseleave={handleMouseLeave}
>
  <!-- Volume Sliders Panel -->
  {#if showVolumeSlider}
    <div 
      class="flex flex-col gap-3 px-4 py-3 rounded-lg backdrop-blur-md transition-all duration-300 animate-fade-in"
      style="background: {$currentTheme.background}cc; border: 2px solid {$currentTheme.primary}80;"
    >
      <!-- Music Volume -->
      <div class="flex items-center gap-3">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 flex-shrink-0" style="color: {$currentTheme.primary};">
          <path d="M9 18V5l12-2v13"/>
          <circle cx="6" cy="18" r="3"/>
          <circle cx="18" cy="16" r="3"/>
        </svg>
        <span class="text-white/60 text-sm font-medium min-w-[3.5rem]">Music</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={$audioSettings.musicVolume}
          oninput={handleMusicVolumeChange}
          onchange={handleMusicVolumeChange}
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
      
      <!-- SFX Volume -->
      <div class="flex items-center gap-3">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-4 h-4 flex-shrink-0" style="color: {$currentTheme.accent};">
          <path d="M11 5L6 9H2v6h4l5 4V5z"/>
          <path d="M15.54 8.46a5 5 0 0 1 0 7.07"/>
        </svg>
        <span class="text-white/60 text-sm font-medium min-w-[3.5rem]">SFX</span>
        <input
          type="range"
          min="0"
          max="1"
          step="0.01"
          value={$audioSettings.sfxVolume}
          oninput={handleSfxVolumeChange}
          onchange={handleSfxVolumeChange}
          class="w-24 h-2 rounded-lg appearance-none cursor-pointer volume-slider"
          style="
            background: linear-gradient(to right, {$currentTheme.accent} 0%, {$currentTheme.accent} {$audioSettings.sfxVolume * 100}%, rgba(255,255,255,0.2) {$audioSettings.sfxVolume * 100}%, rgba(255,255,255,0.2) 100%);
            --thumb-color: {$currentTheme.accent};
          "
        />
        <span class="text-white/80 text-sm font-bold min-w-[2.5rem]" style="color: {$currentTheme.accent};">
          {Math.round($audioSettings.sfxVolume * 100)}%
        </span>
      </div>
    </div>
  {/if}
  
  <!-- Control Buttons -->
  <div class="flex items-center gap-2">
    <!-- Mute/Unmute Button -->
    <Button variant="secondary" size="md" onclick={toggleMute}>
      {#if $audioSettings.isMuted}
        <!-- Muted Icon -->
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-6 h-6" style="color: {$currentTheme.primary};">
          <path d="M11 5L6 9H2v6h4l5 4V5z"/>
          <line x1="23" y1="9" x2="17" y2="15"/>
          <line x1="17" y1="9" x2="23" y2="15"/>
        </svg>
      {:else if $audioSettings.musicVolume < 0.3 && $audioSettings.sfxVolume < 0.3}
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
