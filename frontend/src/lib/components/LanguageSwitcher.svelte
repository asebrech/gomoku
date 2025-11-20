<script lang="ts">
  import { locale } from 'svelte-i18n';
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  import { browser } from '$app/environment';
  
  const languageNames: Record<string, string> = {
    'en': '🇬🇧 EN',
    'fr': '🇫🇷 FR',
    'es': '🇪🇸 ES'
  };
  
  const availableLocales = ['en', 'fr', 'es'];
  
  let isOpen = $state(false);
  
  // Get current locale with fallback
  const currentLocale = $derived($locale || 'en');
  const displayText = $derived(languageNames[currentLocale] || '🇬🇧 EN');
  
  function setLanguage(lang: string) {
    locale.set(lang);
    // Save to localStorage for persistence
    if (browser) {
      localStorage.setItem('selectedLocale', lang);
    }
    isOpen = false;
  }
  
  const dropdownStyles = $derived({
    background: `${$currentTheme.surface}E6`,
    border: `1px solid ${$currentTheme.primary}33`,
    backdropFilter: 'blur(12px)',
    boxShadow: $currentTheme.glowPrimary
  });
  
  const dropdownStyleString = $derived(
    Object.entries(dropdownStyles)
      .map(([key, value]) => `${key.replace(/([A-Z])/g, '-$1').toLowerCase()}: ${value}`)
      .join('; ')
  );
</script>

<div class="relative">
  <Button variant="secondary" size="md" onclick={() => isOpen = !isOpen}>
    {displayText}
  </Button>
  
  {#if isOpen}
    <div
      class="absolute right-0 mt-2 rounded-lg overflow-hidden min-w-[120px] z-50"
      style={dropdownStyleString}
    >
      {#each availableLocales as lang}
        {@const isActive = currentLocale === lang}
        <button
          class="w-full px-4 py-2 text-left text-white hover:bg-white/10 transition-colors duration-150 {isActive ? 'bg-white/20' : ''}"
          onclick={() => setLanguage(lang)}
        >
          {languageNames[lang]}
        </button>
      {/each}
    </div>
  {/if}
</div>

<svelte:window onclick={(e: MouseEvent) => {
  const target = e.target as HTMLElement;
  if (isOpen && !target.closest('.relative')) {
    isOpen = false;
  }
}} />
