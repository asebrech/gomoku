import { writable, get } from 'svelte/store';
import type { ThemeColors } from '$lib/theme/themeStore';
import { synthwaveTheme, blackWhiteTheme } from '$lib/theme/themeStore';

export interface CustomTheme {
	id: string;
	name: string;
	colors: ThemeColors;
	createdAt: string;
	updatedAt: string;
}

// Local storage key
const STORAGE_KEY = 'gomoku_custom_themes';

// Load custom themes from localStorage
function loadCustomThemes(): CustomTheme[] {
	if (typeof window === 'undefined') return [];
	
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		return stored ? JSON.parse(stored) : [];
	} catch (error) {
		console.error('Failed to load custom themes:', error);
		return [];
	}
}

// Save custom themes to localStorage
function saveCustomThemes(themes: CustomTheme[]) {
	if (typeof window === 'undefined') return;
	
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(themes));
	} catch (error) {
		console.error('Failed to save custom themes:', error);
	}
}

// Create the store
function createCustomThemesStore() {
	const { subscribe, set, update } = writable<CustomTheme[]>(loadCustomThemes());
	
	return {
		subscribe,
		
		// Add a new custom theme
		add: (theme: Omit<CustomTheme, 'id' | 'createdAt' | 'updatedAt'>) => {
			const newTheme: CustomTheme = {
				...theme,
				id: crypto.randomUUID(),
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};
			
			update(themes => {
				const updated = [...themes, newTheme];
				saveCustomThemes(updated);
				return updated;
			});
			
			return newTheme;
		},
		
		// Update an existing theme
		update: (id: string, updates: Partial<Omit<CustomTheme, 'id' | 'createdAt'>>) => {
			update(themes => {
				const updated = themes.map(theme =>
					theme.id === id
						? { ...theme, ...updates, updatedAt: new Date().toISOString() }
						: theme
				);
				saveCustomThemes(updated);
				return updated;
			});
		},
		
		// Delete a theme
		delete: (id: string) => {
			update(themes => {
				const updated = themes.filter(theme => theme.id !== id);
				saveCustomThemes(updated);
				return updated;
			});
		},
		
		// Get a theme by id
		getById: (id: string): CustomTheme | undefined => {
			return get({ subscribe }).find(theme => theme.id === id);
		},
		
		// Export theme as JSON
		exportTheme: (id: string): string => {
			const theme = get({ subscribe }).find(t => t.id === id);
			if (!theme) throw new Error('Theme not found');
			
			return JSON.stringify(theme, null, 2);
		},
		
		// Import theme from JSON
		importTheme: (jsonString: string): CustomTheme => {
			try {
				const theme = JSON.parse(jsonString) as CustomTheme;
				
				// Validate theme structure
				if (!theme.name || !theme.colors) {
					throw new Error('Invalid theme structure');
				}
				
				// Create new theme with new ID and timestamps
				const newTheme: CustomTheme = {
					...theme,
					id: crypto.randomUUID(),
					createdAt: new Date().toISOString(),
					updatedAt: new Date().toISOString(),
				};
				
				update(themes => {
					const updated = [...themes, newTheme];
					saveCustomThemes(updated);
					return updated;
				});
				
				return newTheme;
			} catch (error) {
				throw new Error('Failed to import theme: ' + (error as Error).message);
			}
		},
		
		// Duplicate an existing theme
		duplicate: (id: string, newName?: string): CustomTheme => {
			const theme = get({ subscribe }).find(t => t.id === id);
			if (!theme) throw new Error('Theme not found');
			
			const duplicated: CustomTheme = {
				...theme,
				id: crypto.randomUUID(),
				name: newName || `${theme.name} (Copy)`,
				createdAt: new Date().toISOString(),
				updatedAt: new Date().toISOString(),
			};
			
			update(themes => {
				const updated = [...themes, duplicated];
				saveCustomThemes(updated);
				return updated;
			});
			
			return duplicated;
		},
		
		// Reset to empty
		reset: () => {
			set([]);
			saveCustomThemes([]);
		}
	};
}

export const customThemes = createCustomThemesStore();

// Default theme templates users can start from
export const themeTemplates: Record<string, Omit<CustomTheme, 'id' | 'createdAt' | 'updatedAt'>> = {
	synthwave: {
		name: 'Synthwave',
		colors: synthwaveTheme
	},
	blackwhite: {
		name: 'Black & White',
		colors: blackWhiteTheme
	},
	ocean: {
		name: 'Ocean',
		colors: {
			primary: '#0077BE',
			secondary: '#00CED1',
			accent: '#48D1CC',
			background: '#001F3F',
			surface: '#003366',
			textPrimary: '#E0F7FA',
			textSecondary: '#B2EBF2',
			buttonNormal: '#004D73',
			buttonHovered: '#0077BE',
			buttonPressed: '#00CED1',
			stonePlayer1: '#00CED1',
			stonePlayer2: '#FFD700',
			titleBg: 'rgba(0, 51, 102, 0.8)',
			contentBg: 'rgba(0, 77, 115, 0.6)',
			footerBg: 'rgba(0, 119, 190, 0.7)',
			gradientPrimary: 'linear-gradient(135deg, #0077BE 0%, #00CED1 100%)',
			gradientSecondary: 'linear-gradient(135deg, #00CED1 0%, #48D1CC 100%)',
			gradientAccent: 'linear-gradient(135deg, #0077BE 0%, #48D1CC 100%)',
			glowPrimary: '0 0 20px rgba(0, 119, 190, 0.5)',
			glowSecondary: '0 0 20px rgba(0, 206, 209, 0.5)',
			glowAccent: '0 0 30px rgba(72, 209, 204, 0.7)',
		}
	},
	sunset: {
		name: 'Sunset',
		colors: {
			primary: '#FF6B35',
			secondary: '#F7931E',
			accent: '#FFD23F',
			background: '#1A0F0A',
			surface: '#2D1810',
			textPrimary: '#FFE5CC',
			textSecondary: '#FFD4A3',
			buttonNormal: '#4D2610',
			buttonHovered: '#803D16',
			buttonPressed: '#FF6B35',
			stonePlayer1: '#FF6B35',
			stonePlayer2: '#F7931E',
			titleBg: 'rgba(45, 24, 16, 0.8)',
			contentBg: 'rgba(77, 38, 16, 0.6)',
			footerBg: 'rgba(128, 61, 22, 0.7)',
			gradientPrimary: 'linear-gradient(135deg, #FF6B35 0%, #F7931E 50%, #FFD23F 100%)',
			gradientSecondary: 'linear-gradient(135deg, #F7931E 0%, #FFD23F 100%)',
			gradientAccent: 'linear-gradient(135deg, #FF6B35 0%, #FFD23F 100%)',
			glowPrimary: '0 0 20px rgba(255, 107, 53, 0.5)',
			glowSecondary: '0 0 20px rgba(247, 147, 30, 0.5)',
			glowAccent: '0 0 30px rgba(255, 210, 63, 0.7)',
		}
	},
	forest: {
		name: 'Forest',
		colors: {
			primary: '#2D5016',
			secondary: '#4A7C2E',
			accent: '#8BC34A',
			background: '#0D1F0A',
			surface: '#1A331A',
			textPrimary: '#C8E6C9',
			textSecondary: '#A5D6A7',
			buttonNormal: '#1B5E20',
			buttonHovered: '#2E7D32',
			buttonPressed: '#4CAF50',
			stonePlayer1: '#8BC34A',
			stonePlayer2: '#FDD835',
			titleBg: 'rgba(26, 51, 26, 0.8)',
			contentBg: 'rgba(45, 80, 22, 0.6)',
			footerBg: 'rgba(74, 124, 46, 0.7)',
			gradientPrimary: 'linear-gradient(135deg, #2D5016 0%, #4A7C2E 50%, #8BC34A 100%)',
			gradientSecondary: 'linear-gradient(135deg, #4A7C2E 0%, #8BC34A 100%)',
			gradientAccent: 'linear-gradient(135deg, #2D5016 0%, #8BC34A 100%)',
			glowPrimary: '0 0 20px rgba(45, 80, 22, 0.5)',
			glowSecondary: '0 0 20px rgba(74, 124, 46, 0.5)',
			glowAccent: '0 0 30px rgba(139, 195, 74, 0.7)',
		}
	}
};
