import { initI18n } from '$lib/i18n/i18n';
import type { Handle } from '@sveltejs/kit';

// Initialize i18n for SSR
await initI18n();

export const handle: Handle = async ({ event, resolve }) => {
	const response = await resolve(event);
	
	// Add COOP/COEP headers for SharedArrayBuffer support (required for wasm-bindgen-rayon)
	response.headers.set('Cross-Origin-Opener-Policy', 'same-origin');
	response.headers.set('Cross-Origin-Embedder-Policy', 'require-corp');
	
	return response;
};
