import { register, init, getLocaleFromNavigator, waitLocale } from 'svelte-i18n';
import { browser } from '$app/environment';

register('en', () => import('./locales/en.json'));
register('fr', () => import('./locales/fr.json'));
register('es', () => import('./locales/es.json'));

let isInitialized = false;

export async function initI18n() {
  if (isInitialized) return;
  
  init({
    fallbackLocale: 'en',
    initialLocale: browser ? getLocaleFromNavigator() : 'en',
  });
  
  // Wait for the locale to be loaded
  await waitLocale();
  isInitialized = true;
}
