import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface AudioSettings {
  musicVolume: number; // 0 to 1
}

const DEFAULT_SETTINGS: AudioSettings = {
  musicVolume: 0.5
};

// Load settings from localStorage if in browser
function loadSettings(): AudioSettings {
  if (browser) {
    const stored = localStorage.getItem('gomoku-audio-settings');
    if (stored) {
      try {
        return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
      } catch (e) {
        console.error('Failed to parse stored audio settings:', e);
      }
    }
  }
  return DEFAULT_SETTINGS;
}

// Create the store
function createAudioSettingsStore() {
  const { subscribe, set, update } = writable<AudioSettings>(loadSettings());

  return {
    subscribe,
    set: (value: AudioSettings) => {
      if (browser) {
        localStorage.setItem('gomoku-audio-settings', JSON.stringify(value));
      }
      set(value);
    },
    update: (fn: (value: AudioSettings) => AudioSettings) => {
      update((current) => {
        const updated = fn(current);
        if (browser) {
          localStorage.setItem('gomoku-audio-settings', JSON.stringify(updated));
        }
        return updated;
      });
    },
    reset: () => {
      if (browser) {
        localStorage.removeItem('gomoku-audio-settings');
      }
      set(DEFAULT_SETTINGS);
    }
  };
}

export const audioSettings = createAudioSettingsStore();
