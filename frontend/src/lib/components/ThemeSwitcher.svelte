<script lang="ts">
  import { currentTheme, setTheme, availableThemes, synthwaveTheme, blackWhiteTheme, type ThemeColors } from '$lib/theme/themeStore';
  import { customThemes, type CustomTheme, themeTemplates } from '$lib/stores/customThemes';
  
  let showDropdown = $state(false);
  let themes = $state($customThemes);
  
  customThemes.subscribe(value => {
    themes = value;
  });
  
  // Check if a theme is currently active
  function isThemeActive(themeColors: any): boolean {
    return JSON.stringify($currentTheme) === JSON.stringify(themeColors);
  }
  
  function applyBuiltInTheme(themeName: keyof typeof availableThemes) {
    setTheme(themeName);
    showDropdown = false;
  }
  
  function applyTemplateTheme(themeColors: ThemeColors) {
    currentTheme.set(themeColors);
    showDropdown = false;
  }
  
  function applyCustomTheme(theme: CustomTheme) {
    // Set the complete theme, not partial
    currentTheme.set(theme.colors);
    showDropdown = false;
  }
  
  // Close dropdown when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (!target.closest('.theme-switcher-container')) {
      showDropdown = false;
    }
  }
</script>

<svelte:window onclick={handleClickOutside} />

<div class="theme-switcher-container">
  <button
    class="theme-switcher-button"
    onclick={() => showDropdown = !showDropdown}
    aria-label="Switch theme"
    aria-expanded={showDropdown}
  >
    <svg 
      class="w-5 h-5" 
      fill="none" 
      stroke="currentColor" 
      viewBox="0 0 24 24"
    >
      <path 
        stroke-linecap="round" 
        stroke-linejoin="round" 
        stroke-width="2" 
        d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01"
      />
    </svg>
    <span class="ml-2 hidden sm:inline">Themes</span>
  </button>
  
  {#if showDropdown}
    <div class="theme-dropdown">
      <div class="dropdown-header">
        <span class="dropdown-title">Built-in Themes</span>
      </div>
      
      <div class="theme-list">
        <button
          class="theme-option {isThemeActive(synthwaveTheme) ? 'active' : ''}"
          onclick={() => applyBuiltInTheme('synthwave')}
        >
          <div class="theme-preview-dots">
            <span class="dot" style="background: #FF00FF;"></span>
            <span class="dot" style="background: #00FFFF;"></span>
            <span class="dot" style="background: #FF33CC;"></span>
          </div>
          <span>Synthwave</span>
          {#if isThemeActive(synthwaveTheme)}
            <span class="active-badge">✓</span>
          {/if}
        </button>
        
        <button
          class="theme-option {isThemeActive(blackWhiteTheme) ? 'active' : ''}"
          onclick={() => applyBuiltInTheme('blackwhite')}
        >
          <div class="theme-preview-dots">
            <span class="dot" style="background: #FFFFFF;"></span>
            <span class="dot" style="background: #000000;"></span>
            <span class="dot" style="background: #666666;"></span>
          </div>
          <span>Black & White</span>
          {#if isThemeActive(blackWhiteTheme)}
            <span class="active-badge">✓</span>
          {/if}
        </button>
        
        <button
          class="theme-option {isThemeActive(themeTemplates.ocean.colors) ? 'active' : ''}"
          onclick={() => applyTemplateTheme(themeTemplates.ocean.colors)}
        >
          <div class="theme-preview-dots">
            <span class="dot" style="background: #0077BE;"></span>
            <span class="dot" style="background: #00CED1;"></span>
            <span class="dot" style="background: #48D1CC;"></span>
          </div>
          <span>Ocean</span>
          {#if isThemeActive(themeTemplates.ocean.colors)}
            <span class="active-badge">✓</span>
          {/if}
        </button>
        
        <button
          class="theme-option {isThemeActive(themeTemplates.sunset.colors) ? 'active' : ''}"
          onclick={() => applyTemplateTheme(themeTemplates.sunset.colors)}
        >
          <div class="theme-preview-dots">
            <span class="dot" style="background: #FF6B35;"></span>
            <span class="dot" style="background: #F7931E;"></span>
            <span class="dot" style="background: #FFD23F;"></span>
          </div>
          <span>Sunset</span>
          {#if isThemeActive(themeTemplates.sunset.colors)}
            <span class="active-badge">✓</span>
          {/if}
        </button>
        
        <button
          class="theme-option {isThemeActive(themeTemplates.forest.colors) ? 'active' : ''}"
          onclick={() => applyTemplateTheme(themeTemplates.forest.colors)}
        >
          <div class="theme-preview-dots">
            <span class="dot" style="background: #2D5016;"></span>
            <span class="dot" style="background: #4A7C2E;"></span>
            <span class="dot" style="background: #8BC34A;"></span>
          </div>
          <span>Forest</span>
          {#if isThemeActive(themeTemplates.forest.colors)}
            <span class="active-badge">✓</span>
          {/if}
        </button>
      </div>
      
      {#if themes.length > 0}
        <div class="dropdown-divider"></div>
        
        <div class="dropdown-header">
          <span class="dropdown-title">Custom Themes</span>
          <a href="/themes" class="edit-link">Edit</a>
        </div>
        
        <div class="theme-list">
          {#each themes as theme}
            <button
              class="theme-option {isThemeActive(theme.colors) ? 'active' : ''}"
              onclick={() => applyCustomTheme(theme)}
            >
              <div class="theme-preview-dots">
                <span class="dot" style="background: {theme.colors.primary};"></span>
                <span class="dot" style="background: {theme.colors.secondary};"></span>
                <span class="dot" style="background: {theme.colors.accent};"></span>
              </div>
              <span>{theme.name}</span>
              {#if isThemeActive(theme.colors)}
                <span class="active-badge">✓</span>
              {/if}
            </button>
          {/each}
        </div>
      {/if}
      
      <div class="dropdown-divider"></div>
      
      <a href="/themes" class="create-theme-link">
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        <span>Create New Theme</span>
      </a>
    </div>
  {/if}
</div>

<style>
  .theme-switcher-container {
    position: relative;
  }
  
  .theme-switcher-button {
    display: flex;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: white;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .theme-switcher-button:hover {
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.3);
  }
  
  .theme-dropdown {
    position: absolute;
    top: calc(100% + 0.5rem);
    right: 0;
    width: 16rem;
    max-height: 24rem;
    overflow-y: auto;
    background: rgba(17, 24, 39, 0.95);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(139, 92, 246, 0.3);
    border-radius: 0.75rem;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.5);
    z-index: 100;
  }
  
  .dropdown-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .dropdown-title {
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: rgba(255, 255, 255, 0.6);
  }
  
  .edit-link {
    font-size: 0.75rem;
    color: #8B5CF6;
    text-decoration: none;
    transition: color 0.2s ease;
  }
  
  .edit-link:hover {
    color: #A78BFA;
  }
  
  .theme-list {
    padding: 0.5rem;
  }
  
  .theme-option {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    width: 100%;
    padding: 0.625rem 0.75rem;
    background: transparent;
    border: none;
    border-radius: 0.5rem;
    color: white;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .theme-option:hover {
    background: rgba(139, 92, 246, 0.2);
  }
  
  .theme-option.active {
    background: rgba(139, 92, 246, 0.3);
    border: 1px solid rgba(139, 92, 246, 0.5);
  }
  
  .active-badge {
    margin-left: auto;
    color: #8B5CF6;
    font-weight: bold;
    font-size: 1rem;
  }
  
  .theme-preview-dots {
    display: flex;
    gap: 0.25rem;
  }
  
  .dot {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.2);
  }
  
  .dropdown-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.1);
    margin: 0.5rem 0;
  }
  
  .create-theme-link {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(139, 92, 246, 0.2);
    border: none;
    border-radius: 0 0 0.75rem 0.75rem;
    color: #A78BFA;
    font-size: 0.875rem;
    font-weight: 600;
    text-decoration: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .create-theme-link:hover {
    background: rgba(139, 92, 246, 0.3);
    color: #C4B5FD;
  }
  
  /* Scrollbar styling */
  .theme-dropdown::-webkit-scrollbar {
    width: 0.5rem;
  }
  
  .theme-dropdown::-webkit-scrollbar-track {
    background: rgba(0, 0, 0, 0.2);
    border-radius: 0.25rem;
  }
  
  .theme-dropdown::-webkit-scrollbar-thumb {
    background: rgba(139, 92, 246, 0.5);
    border-radius: 0.25rem;
  }
  
  .theme-dropdown::-webkit-scrollbar-thumb:hover {
    background: rgba(139, 92, 246, 0.7);
  }
  
  @media (max-width: 640px) {
    .theme-dropdown {
      right: -1rem;
      width: 14rem;
    }
  }
</style>
