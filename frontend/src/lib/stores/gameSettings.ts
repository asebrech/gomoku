import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface GameSettings {
  boardSize: number;
  winCondition: number;
  aiDepth: number;
  aiMaxThinkingTime: number; // in milliseconds
  showDoubleThree: boolean;
  showAIHint: boolean;
}

const DEFAULT_SETTINGS: GameSettings = {
  boardSize: 19,
  winCondition: 5,
  aiDepth: 20,
  aiMaxThinkingTime: 500, // 500ms (0.5 second) default
  showDoubleThree: false,
  showAIHint: false
};

// Load settings from localStorage if in browser
function loadSettings(): GameSettings {
  if (browser) {
    const stored = localStorage.getItem('gomoku-settings');
    if (stored) {
      try {
        return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
      } catch (e) {
        console.error('Failed to parse stored settings:', e);
      }
    }
  }
  return DEFAULT_SETTINGS;
}

// Create the store
function createGameSettingsStore() {
  const { subscribe, set, update } = writable<GameSettings>(loadSettings());

  return {
    subscribe,
    set: (value: GameSettings) => {
      if (browser) {
        localStorage.setItem('gomoku-settings', JSON.stringify(value));
      }
      set(value);
    },
    update: (fn: (value: GameSettings) => GameSettings) => {
      update((current) => {
        const updated = fn(current);
        if (browser) {
          localStorage.setItem('gomoku-settings', JSON.stringify(updated));
        }
        return updated;
      });
    },
    reset: () => {
      if (browser) {
        localStorage.removeItem('gomoku-settings');
      }
      set(DEFAULT_SETTINGS);
    }
  };
}

export const gameSettings = createGameSettingsStore();
