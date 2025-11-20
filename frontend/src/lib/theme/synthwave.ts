// Synthwave theme colors converted from RGB (0-1) to hex
export const synthwave = {
	// Core colors
	primary: '#FF00FF',      // Magenta
	secondary: '#00FFFF',    // Cyan
	accent: '#FF33CC',       // Hot pink
	
	// Background colors
	background: '#0D0026',   // Deep purple-black
	surface: '#1A0033',      // Dark purple
	
	// Text colors
	textPrimary: '#00FFFF',    // Cyan
	textSecondary: '#FF00FF',  // Magenta
	
	// Button states
	buttonNormal: '#330066',   // Purple
	buttonHovered: '#660099',  // Lighter purple
	buttonPressed: '#FF33CC',  // Hot pink
	
	// Game elements
	stonePlayer1: '#FF66B2',   // Pink
	stonePlayer2: '#3399FF',   // Blue
	
	// UI backgrounds
	titleBg: 'rgba(26, 0, 51, 0.8)',
	contentBg: 'rgba(51, 0, 102, 0.6)',
	footerBg: 'rgba(77, 0, 153, 0.7)',
	
	// Gradients
	gradientPrimary: 'linear-gradient(135deg, #FF00FF 0%, #FF33CC 50%, #FF66B2 100%)',
	gradientSecondary: 'linear-gradient(135deg, #00FFFF 0%, #3399FF 100%)',
	gradientAccent: 'linear-gradient(135deg, #FF00FF 0%, #00FFFF 100%)',
	
	// Glow effects
	glowPrimary: '0 0 20px rgba(255, 0, 255, 0.5)',
	glowSecondary: '0 0 20px rgba(0, 255, 255, 0.5)',
	glowAccent: '0 0 30px rgba(255, 51, 204, 0.7)',
} as const;

// Tailwind config extension for synthwave theme
export const synthwaveTailwind = {
	colors: {
		synthwave: {
			primary: synthwave.primary,
			secondary: synthwave.secondary,
			accent: synthwave.accent,
			bg: synthwave.background,
			surface: synthwave.surface,
			'text-primary': synthwave.textPrimary,
			'text-secondary': synthwave.textSecondary,
			'button-normal': synthwave.buttonNormal,
			'button-hovered': synthwave.buttonHovered,
			'button-pressed': synthwave.buttonPressed,
			'stone-p1': synthwave.stonePlayer1,
			'stone-p2': synthwave.stonePlayer2,
		}
	},
	boxShadow: {
		'glow-primary': synthwave.glowPrimary,
		'glow-secondary': synthwave.glowSecondary,
		'glow-accent': synthwave.glowAccent,
	}
};
