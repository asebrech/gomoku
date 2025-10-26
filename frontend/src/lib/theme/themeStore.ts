import { writable } from 'svelte/store';

// Theme interface
export interface ThemeColors {
	// Core colors
	primary: string;
	secondary: string;
	accent: string;
	
	// Background colors
	background: string;
	surface: string;
	
	// Text colors
	textPrimary: string;
	textSecondary: string;
	
	// Button states
	buttonNormal: string;
	buttonHovered: string;
	buttonPressed: string;
	
	// Game elements
	stonePlayer1: string;
	stonePlayer2: string;
	
	// UI backgrounds
	titleBg: string;
	contentBg: string;
	footerBg: string;
	
	// Gradients
	gradientPrimary: string;
	gradientSecondary: string;
	gradientAccent: string;
	
	// Glow effects
	glowPrimary: string;
	glowSecondary: string;
	glowAccent: string;
}

// Synthwave theme (default)
export const synthwaveTheme: ThemeColors = {
	primary: '#FF00FF',
	secondary: '#00FFFF',
	accent: '#FF33CC',
	background: '#0D0026',
	surface: '#1A0033',
	textPrimary: '#00FFFF',
	textSecondary: '#FF00FF',
	buttonNormal: '#330066',
	buttonHovered: '#660099',
	buttonPressed: '#FF33CC',
	stonePlayer1: '#FF66B2',
	stonePlayer2: '#3399FF',
	titleBg: 'rgba(26, 0, 51, 0.8)',
	contentBg: 'rgba(51, 0, 102, 0.6)',
	footerBg: 'rgba(77, 0, 153, 0.7)',
	gradientPrimary: 'linear-gradient(135deg, #FF00FF 0%, #FF33CC 50%, #FF66B2 100%)',
	gradientSecondary: 'linear-gradient(135deg, #00FFFF 0%, #3399FF 100%)',
	gradientAccent: 'linear-gradient(135deg, #FF00FF 0%, #00FFFF 100%)',
	glowPrimary: '0 0 20px rgba(255, 0, 255, 0.5)',
	glowSecondary: '0 0 20px rgba(0, 255, 255, 0.5)',
	glowAccent: '0 0 30px rgba(255, 51, 204, 0.7)',
};

// Black & White theme (from your themes folder)
export const blackWhiteTheme: ThemeColors = {
	primary: '#FFFFFF',
	secondary: '#000000',
	accent: '#666666',
	background: '#000000',
	surface: '#1A1A1A',
	textPrimary: '#FFFFFF',
	textSecondary: '#CCCCCC',
	buttonNormal: '#333333',
	buttonHovered: '#555555',
	buttonPressed: '#FFFFFF',
	stonePlayer1: '#FFFFFF',
	stonePlayer2: '#000000',
	titleBg: 'rgba(26, 26, 26, 0.8)',
	contentBg: 'rgba(51, 51, 51, 0.6)',
	footerBg: 'rgba(77, 77, 77, 0.7)',
	gradientPrimary: 'linear-gradient(135deg, #FFFFFF 0%, #CCCCCC 100%)',
	gradientSecondary: 'linear-gradient(135deg, #666666 0%, #333333 100%)',
	gradientAccent: 'linear-gradient(135deg, #FFFFFF 0%, #000000 100%)',
	glowPrimary: '0 0 20px rgba(255, 255, 255, 0.5)',
	glowSecondary: '0 0 20px rgba(200, 200, 200, 0.5)',
	glowAccent: '0 0 30px rgba(255, 255, 255, 0.7)',
};

// Active theme store - can be changed at runtime
export const currentTheme = writable<ThemeColors>(synthwaveTheme);

// Available themes registry
export const availableThemes = {
	synthwave: synthwaveTheme,
	blackwhite: blackWhiteTheme,
} as const;

// Helper function to switch themes
export function setTheme(themeName: keyof typeof availableThemes) {
	currentTheme.set(availableThemes[themeName]);
	
	// Also update CSS custom properties for use outside Svelte
	if (typeof document !== 'undefined') {
		updateCSSVariables(availableThemes[themeName]);
	}
}

// Helper function to set custom theme
export function setCustomTheme(theme: Partial<ThemeColors>) {
	currentTheme.update(current => ({
		...current,
		...theme
	}));
	
	if (typeof document !== 'undefined') {
		currentTheme.subscribe(t => updateCSSVariables(t));
	}
}

// Update CSS custom properties
function updateCSSVariables(theme: ThemeColors) {
	const root = document.documentElement;
	
	root.style.setProperty('--color-primary', theme.primary);
	root.style.setProperty('--color-secondary', theme.secondary);
	root.style.setProperty('--color-accent', theme.accent);
	root.style.setProperty('--color-background', theme.background);
	root.style.setProperty('--color-surface', theme.surface);
	root.style.setProperty('--color-text-primary', theme.textPrimary);
	root.style.setProperty('--color-text-secondary', theme.textSecondary);
	root.style.setProperty('--color-button-normal', theme.buttonNormal);
	root.style.setProperty('--color-button-hovered', theme.buttonHovered);
	root.style.setProperty('--color-button-pressed', theme.buttonPressed);
	root.style.setProperty('--color-stone-p1', theme.stonePlayer1);
	root.style.setProperty('--color-stone-p2', theme.stonePlayer2);
	root.style.setProperty('--gradient-primary', theme.gradientPrimary);
	root.style.setProperty('--gradient-secondary', theme.gradientSecondary);
	root.style.setProperty('--gradient-accent', theme.gradientAccent);
	root.style.setProperty('--glow-primary', theme.glowPrimary);
	root.style.setProperty('--glow-secondary', theme.glowSecondary);
	root.style.setProperty('--glow-accent', theme.glowAccent);
}

// Initialize CSS variables on load
if (typeof document !== 'undefined') {
	currentTheme.subscribe(updateCSSVariables);
}
