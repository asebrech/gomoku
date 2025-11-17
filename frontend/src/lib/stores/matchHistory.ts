import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface MatchData {
  id: string;
  date: Date;
  player1Name: string;
  player2Name: string;
  player1Type: 'human' | 'ai';
  player2Type: 'human' | 'ai';
  boardSize: number;
  winCondition: number;
  aiDepth?: number;
  moveHistory: Array<{ row: number; col: number }>;
  winner: 1 | 2 | null; // null means ongoing or draw
  status: 'ongoing' | 'finished';
  totalMoves: number;
  player1Captures: number;
  player2Captures: number;
  duration?: number; // in seconds
}

const STORAGE_KEY = 'gomoku_match_history';
const MAX_MATCHES = 100; // Limit to prevent storage overflow

function loadMatchHistory(): MatchData[] {
  if (!browser) {
    return [];
  }
  
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    
    if (!stored) {
      return [];
    }
    
    const parsed = JSON.parse(stored);
    
    // Convert date strings back to Date objects
    const matches = parsed.map((match: any) => ({
      ...match,
      date: new Date(match.date)
    }));
    
    return matches;
  } catch (error) {
    console.error('❌ Failed to load match history:', error);
    return [];
  }
}

function saveMatchHistory(matches: MatchData[]): void {
  if (!browser) return;
  
  try {
    // Keep only the most recent MAX_MATCHES
    const toSave = matches.slice(0, MAX_MATCHES);
    const jsonString = JSON.stringify(toSave);
    localStorage.setItem(STORAGE_KEY, jsonString);
  } catch (error) {
    console.error('❌ Failed to save match history:', error);
  }
}

function createMatchHistoryStore() {
  const { subscribe, set, update } = writable<MatchData[]>(loadMatchHistory());
  
  return {
    subscribe,
    addMatch: (match: MatchData) => {
      update(matches => {
        const newMatches = [match, ...matches];
        saveMatchHistory(newMatches);
        return newMatches;
      });
    },
    updateMatch: (id: string, updates: Partial<MatchData>) => {
      update(matches => {
        const newMatches = matches.map(match => 
          match.id === id ? { ...match, ...updates } : match
        );
        saveMatchHistory(newMatches);
        return newMatches;
      });
    },
    deleteMatch: (id: string) => {
      update(matches => {
        const newMatches = matches.filter(match => match.id !== id);
        saveMatchHistory(newMatches);
        return newMatches;
      });
    },
    clearAll: () => {
      set([]);
      saveMatchHistory([]);
    },
    refresh: () => {
      const loaded = loadMatchHistory();
      set(loaded);
    }
  };
}

export const matchHistory = createMatchHistoryStore();

// Helper function to generate a unique match ID
export function generateMatchId(): string {
  return `match_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}
