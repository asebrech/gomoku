import { register, init, getLocaleFromNavigator, waitLocale } from 'svelte-i18n';
import { browser } from '$app/environment';

register('en', () => import('./locales/en.json'));
register('fr', () => import('./locales/fr.json'));
register('es', () => import('./locales/es.json'));

let isInitialized = false;

// Get saved locale from localStorage or fallback to browser locale
function getSavedLocale(): string {
  if (browser) {
    const saved = localStorage.getItem('selectedLocale');
    if (saved && ['en', 'fr', 'es'].includes(saved)) {
      return saved;
    }
    return getLocaleFromNavigator() || 'en';
  }
  return 'en';
}

export async function initI18n() {
  if (isInitialized) return;
  
  init({
    fallbackLocale: 'en',
    initialLocale: getSavedLocale(),
  });
  
  // Wait for the locale to be loaded
  await waitLocale();
  isInitialized = true;
}
