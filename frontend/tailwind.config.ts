import type { Config } from 'tailwindcss';
import { synthwaveTailwind } from './src/lib/theme/synthwave';

export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	theme: {
		extend: {
			...synthwaveTailwind,
			animation: {
				'fade-in': 'fadeIn 1s ease-in',
				'float': 'float 3s ease-in-out infinite'
			},
			keyframes: {
				fadeIn: {
					'0%': { opacity: '0', transform: 'translateY(20px)' },
					'100%': { opacity: '1', transform: 'translateY(0)' }
				},
				float: {
					'0%, 100%': { transform: 'translateY(0px)' },
					'50%': { transform: 'translateY(-10px)' }
				}
			}
		}
	},
	plugins: []
} satisfies Config;
