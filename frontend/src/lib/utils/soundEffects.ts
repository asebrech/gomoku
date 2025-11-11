import { get } from 'svelte/store';
import { audioSettings } from '$lib/stores/audioSettings';

// Sound effect paths
const SOUND_PATHS = {
  click: '/src/lib/assets/sound/click.mp3',
  gameWin: '/src/lib/assets/sound/game_win.wav',
  gameLose: '/src/lib/assets/sound/game_lose.wav',
  stoneOne: '/src/lib/assets/sound/stone_one.wav',
  stoneTwo: '/src/lib/assets/sound/stone_two.wav'
};

// Preloaded audio elements for better performance
const audioCache: Map<string, HTMLAudioElement> = new Map();

// Reference to background music element for ducking
let backgroundMusicElement: HTMLAudioElement | null = null;

/**
 * Set the background music element reference for ducking
 */
export function setBackgroundMusicElement(element: HTMLAudioElement | null): void {
  backgroundMusicElement = element;
}

/**
 * Preload a sound effect
 */
function preloadSound(path: string): HTMLAudioElement {
  if (!audioCache.has(path)) {
    const audio = new Audio(path);
    audio.preload = 'auto';
    audioCache.set(path, audio);
  }
  return audioCache.get(path)!;
}

/**
 * Play a sound effect with the current SFX volume
 */
function playSound(path: string, important: boolean = false): void {
  try {
    const settings = get(audioSettings);
    
    // Don't play if muted or SFX volume is 0
    if (settings.isMuted || settings.sfxVolume === 0) {
      return;
    }
    
    const audio = preloadSound(path);
    
    // Clone the audio to allow overlapping sounds
    const soundClone = audio.cloneNode() as HTMLAudioElement;
    
    // Duck background music for important sounds (win/lose)
    if (important && backgroundMusicElement) {
      // Boost win/lose sounds and drastically reduce music
      soundClone.volume = Math.min(settings.sfxVolume * 1.8, 1.0); // Boost by 80%
      const targetMusicVolume = settings.musicVolume;
      backgroundMusicElement.volume = 0.02; // Almost completely silent
      
      // Restore volumes after sound finishes
      soundClone.addEventListener('ended', () => {
        if (backgroundMusicElement) {
          backgroundMusicElement.volume = targetMusicVolume;
        }
      });
      
      // Also restore after a timeout as fallback
      setTimeout(() => {
        if (backgroundMusicElement) {
          backgroundMusicElement.volume = targetMusicVolume;
        }
      }, 5000);
    } else {
      // Normal sounds at regular SFX volume
      soundClone.volume = settings.sfxVolume;
    }
    
    soundClone.play().catch(() => {});
  } catch (error) {
    console.error('Error playing sound:', error);
  }
}

/**
 * Play the click sound effect
 */
export function playClickSound(): void {
  playSound(SOUND_PATHS.click);
}

/**
 * Play a random stone placement sound
 */
export function playStoneSound(): void {
  const stoneSound = Math.random() < 0.5 ? SOUND_PATHS.stoneOne : SOUND_PATHS.stoneTwo;
  playSound(stoneSound);
}

/**
 * Play the game win sound
 */
export function playWinSound(): void {
  playSound(SOUND_PATHS.gameWin, true); // Mark as important for music ducking
}

/**
 * Play the game lose sound
 */
export function playLoseSound(): void {
  playSound(SOUND_PATHS.gameLose, true); // Mark as important for music ducking
}

/**
 * Preload all sound effects on app initialization
 */
export function preloadAllSounds(): void {
  Object.values(SOUND_PATHS).forEach(path => {
    preloadSound(path);
  });
}
