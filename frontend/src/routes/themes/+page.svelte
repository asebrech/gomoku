<script lang="ts">
  import { goto } from '$app/navigation';
  import { _, locale } from 'svelte-i18n';
  import { get } from 'svelte/store';
  import Button from '$lib/components/Button.svelte';
  import ColorPicker from '$lib/components/ColorPicker.svelte';
  import ThemePreview from '$lib/components/ThemePreview.svelte';
  import { customThemes, themeTemplates, type CustomTheme } from '$lib/stores/customThemes';
  import { currentTheme, setCustomTheme, synthwaveTheme, type ThemeColors } from '$lib/theme/themeStore';
  
  // Current editing state
  let editingTheme = $state<Partial<ThemeColors>>({ ...synthwaveTheme });
  let themeName = $state('My Custom Theme');
  let selectedTemplate = $state<string>('synthwave');
  let editingThemeId = $state<string | null>(null);
  let showImportDialog = $state(false);
  let importJson = $state('');
  let errorMessage = $state('');
  let successMessage = $state('');
  
  // Load custom themes
  let themes = $state($customThemes);
  
  customThemes.subscribe(value => {
    themes = value;
  });
  
  // Color categories for organization
  const colorCategories = [
    {
      titleKey: 'themeEditor.colorCategories.core',
      colors: [
        { key: 'primary', labelKey: 'themeEditor.colorLabels.primary', descKey: 'themeEditor.colorDescriptions.primary' },
        { key: 'secondary', labelKey: 'themeEditor.colorLabels.secondary', descKey: 'themeEditor.colorDescriptions.secondary' },
        { key: 'accent', labelKey: 'themeEditor.colorLabels.accent', descKey: 'themeEditor.colorDescriptions.accent' },
      ]
    },
    {
      titleKey: 'themeEditor.colorCategories.backgrounds',
      colors: [
        { key: 'background', labelKey: 'themeEditor.colorLabels.background', descKey: 'themeEditor.colorDescriptions.background' },
        { key: 'surface', labelKey: 'themeEditor.colorLabels.surface', descKey: 'themeEditor.colorDescriptions.surface' },
      ]
    },
    {
      titleKey: 'themeEditor.colorCategories.text',
      colors: [
        { key: 'textPrimary', labelKey: 'themeEditor.colorLabels.textPrimary', descKey: 'themeEditor.colorDescriptions.textPrimary' },
        { key: 'textSecondary', labelKey: 'themeEditor.colorLabels.textSecondary', descKey: 'themeEditor.colorDescriptions.textSecondary' },
      ]
    },
    {
      titleKey: 'themeEditor.colorCategories.buttons',
      colors: [
        { key: 'buttonNormal', labelKey: 'themeEditor.colorLabels.buttonNormal', descKey: 'themeEditor.colorDescriptions.buttonNormal' },
        { key: 'buttonHovered', labelKey: 'themeEditor.colorLabels.buttonHovered', descKey: 'themeEditor.colorDescriptions.buttonHovered' },
        { key: 'buttonPressed', labelKey: 'themeEditor.colorLabels.buttonPressed', descKey: 'themeEditor.colorDescriptions.buttonPressed' },
      ]
    },
    {
      titleKey: 'themeEditor.colorCategories.gameElements',
      colors: [
        { key: 'stonePlayer1', labelKey: 'themeEditor.colorLabels.stonePlayer1', descKey: 'themeEditor.colorDescriptions.stonePlayer1' },
        { key: 'stonePlayer2', labelKey: 'themeEditor.colorLabels.stonePlayer2', descKey: 'themeEditor.colorDescriptions.stonePlayer2' },
      ]
    }
  ];
  
  function hexToRgba(hex: string, alpha: number = 1): string {
    const r = parseInt(hex.slice(1, 3), 16);
    const g = parseInt(hex.slice(3, 5), 16);
    const b = parseInt(hex.slice(5, 7), 16);
    return `rgba(${r}, ${g}, ${b}, ${alpha})`;
  }
  
  function generateGradient(color1: string, color2: string, color3?: string): string {
    if (color3) {
      return `linear-gradient(135deg, ${color1} 0%, ${color2} 50%, ${color3} 100%)`;
    }
    return `linear-gradient(135deg, ${color1} 0%, ${color2} 100%)`;
  }
  
  function generateGlow(color: string, strength: number = 0.5): string {
    return `0 0 20px ${hexToRgba(color, strength)}`;
  }
  
  // Auto-generate derived colors
  $effect(() => {
    if (editingTheme.primary && editingTheme.secondary && editingTheme.accent) {
      editingTheme.gradientPrimary = generateGradient(
        editingTheme.primary,
        editingTheme.accent,
        editingTheme.primary
      );
      editingTheme.gradientSecondary = generateGradient(
        editingTheme.secondary,
        editingTheme.accent
      );
      editingTheme.gradientAccent = generateGradient(
        editingTheme.primary,
        editingTheme.secondary
      );
    }
    
    if (editingTheme.primary) {
      editingTheme.glowPrimary = generateGlow(editingTheme.primary);
    }
    if (editingTheme.secondary) {
      editingTheme.glowSecondary = generateGlow(editingTheme.secondary);
    }
    if (editingTheme.accent) {
      editingTheme.glowAccent = generateGlow(editingTheme.accent, 0.7);
    }
    
    if (editingTheme.surface) {
      editingTheme.titleBg = hexToRgba(editingTheme.surface, 0.8);
      editingTheme.contentBg = hexToRgba(editingTheme.surface, 0.6);
      editingTheme.footerBg = hexToRgba(editingTheme.surface, 0.7);
    }
  });
  
  function loadTemplate(templateName: string) {
    const template = themeTemplates[templateName as keyof typeof themeTemplates];
    if (template) {
      editingTheme = { ...template.colors };
      themeName = template.name + ' (Copy)';
      editingThemeId = null;
    }
  }
  
  function saveTheme() {
    if (!themeName.trim()) {
      errorMessage = get(_)('themeEditor.messages.enterName');
      setTimeout(() => errorMessage = '', 3000);
      return;
    }
    
    const completeTheme = editingTheme as ThemeColors;
    
    if (editingThemeId) {
      // Update existing theme
      customThemes.update(editingThemeId, {
        name: themeName,
        colors: completeTheme
      });
      successMessage = get(_)('themeEditor.messages.themeUpdated');
    } else {
      // Create new theme
      customThemes.add({
        name: themeName,
        colors: completeTheme
      });
      successMessage = get(_)('themeEditor.messages.themeSaved');
    }
    
    setTimeout(() => successMessage = '', 3000);
    editingThemeId = null;
  }
  
  function loadThemeForEditing(theme: CustomTheme) {
    editingTheme = { ...theme.colors };
    themeName = theme.name;
    editingThemeId = theme.id;
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }
  
  function applyTheme(theme: CustomTheme) {
    currentTheme.set(theme.colors);
    successMessage = get(_)('themeEditor.messages.themeApplied', { values: { name: theme.name } });
    setTimeout(() => successMessage = '', 3000);
  }
  
  function deleteTheme(id: string) {
    if (confirm(get(_)('themeEditor.messages.deleteConfirm'))) {
      customThemes.delete(id);
      if (editingThemeId === id) {
        editingThemeId = null;
        themeName = 'My Custom Theme';
        editingTheme = { ...synthwaveTheme };
      }
      successMessage = get(_)('themeEditor.messages.themeDeleted');
      setTimeout(() => successMessage = '', 3000);
    }
  }
  
  function exportTheme(id: string) {
    try {
      const json = customThemes.exportTheme(id);
      const blob = new Blob([json], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `theme-${id}.json`;
      a.click();
      URL.revokeObjectURL(url);
      successMessage = get(_)('themeEditor.messages.themeExported');
      setTimeout(() => successMessage = '', 3000);
    } catch (error) {
      errorMessage = get(_)('themeEditor.messages.exportFailed');
      setTimeout(() => errorMessage = '', 3000);
    }
  }
  
  function importTheme() {
    try {
      const imported = customThemes.importTheme(importJson);
      successMessage = get(_)('themeEditor.messages.themeImported', { values: { name: imported.name } });
      showImportDialog = false;
      importJson = '';
      setTimeout(() => successMessage = '', 3000);
    } catch (error) {
      errorMessage = (error as Error).message;
      setTimeout(() => errorMessage = '', 3000);
    }
  }
  
  function duplicateTheme(id: string) {
    const theme = themes.find(t => t.id === id);
    if (theme) {
      customThemes.duplicate(id, `${theme.name} (Copy)`);
      successMessage = get(_)('themeEditor.messages.themeDuplicated');
      setTimeout(() => successMessage = '', 3000);
    }
  }
  
  function resetEditor() {
    editingTheme = { ...synthwaveTheme };
    themeName = 'My Custom Theme';
    editingThemeId = null;
  }
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-7xl mx-auto px-4">
    <!-- Header -->
    <div class="text-center mb-8">
      <h1 class="text-5xl font-black text-transparent bg-clip-text mb-4"
          style="background-image: {$currentTheme.gradientPrimary}; -webkit-background-clip: text; background-clip: text;">
        {$_('themeEditor.title')}
      </h1>
      <p class="text-sm text-white/60">
        {$_('themeEditor.subtitle')}
      </p>
    </div>
    
    <!-- Messages -->
    {#if successMessage}
      <div class="message success-message mb-4">
        ✓ {successMessage}
      </div>
    {/if}
    {#if errorMessage}
      <div class="message error-message mb-4">
        ✗ {errorMessage}
      </div>
    {/if}
    
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Left Column: Editor -->
      <div class="space-y-6">
        <!-- Theme Name and Template -->
        <div class="panel">
          <h2 class="section-title">{$_('themeEditor.themeSettings')}</h2>
          
          <div class="form-group">
            <label for="theme-name" class="form-label">{$_('themeEditor.themeName')}</label>
            <input
              id="theme-name"
              type="text"
              bind:value={themeName}
              placeholder={$_('themeEditor.themeNamePlaceholder')}
              class="text-input"
            />
          </div>
          
          <div class="form-group">
            <label for="existing-theme-select" class="form-label">{$_('themeEditor.editExisting')}</label>
            <select
              id="existing-theme-select"
              onchange={(e) => {
                const selectedId = (e.target as HTMLSelectElement).value;
                if (selectedId) {
                  const theme = themes.find(t => t.id === selectedId);
                  if (theme) loadThemeForEditing(theme);
                }
              }}
              class="select-input"
            >
              <option value="">{$_('themeEditor.selectTheme')}</option>
              {#each themes as theme}
                <option value={theme.id} selected={theme.id === editingThemeId}>
                  {theme.name}
                </option>
              {/each}
            </select>
          </div>
          
          <div class="form-group">
            <label for="template-select" class="form-label">{$_('themeEditor.startFromTemplate')}</label>
            <select
              id="template-select"
              bind:value={selectedTemplate}
              onchange={() => loadTemplate(selectedTemplate)}
              class="select-input"
            >
              {#each Object.keys(themeTemplates) as templateKey}
                <option value={templateKey}>
                  {$_(`themes.${templateKey}`)}
                </option>
              {/each}
            </select>
          </div>
          
          <div class="flex gap-3">
            <Button variant="primary" size="md" onclick={saveTheme}>
              {editingThemeId ? $_('themeEditor.updateTheme') : $_('themeEditor.saveTheme')}
            </Button>
            <Button variant="secondary" size="md" onclick={resetEditor}>
              {$_('themeEditor.reset')}
            </Button>
          </div>
        </div>
        
        <!-- Color Pickers -->
        {#each colorCategories as category}
          <div class="panel">
            <h3 class="section-title">{$_(category.titleKey)}</h3>
            <div class="color-grid">
              {#each category.colors as colorDef}
                <ColorPicker
                  label={$_(colorDef.labelKey)}
                  description={$_(colorDef.descKey)}
                  bind:value={editingTheme[colorDef.key as keyof ThemeColors]!}
                />
              {/each}
            </div>
          </div>
        {/each}
        
        <!-- Bottom Save Button -->
        <div class="panel">
          <div class="flex gap-3">
            <Button variant="primary" size="lg" fullWidth={true} onclick={saveTheme}>
              {editingThemeId ? $_('themeEditor.updateTheme') : $_('themeEditor.saveTheme')}
            </Button>
            <Button variant="secondary" size="lg" onclick={resetEditor}>
              {$_('themeEditor.reset')}
            </Button>
          </div>
        </div>
      </div>
      
      <!-- Right Column: Preview -->
      <div class="space-y-6">
        <div class="panel sticky top-4">
          <ThemePreview theme={editingTheme as ThemeColors} />
        </div>
      </div>
    </div>
    
    <!-- Saved Themes -->
    <div class="mt-12">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-3xl font-bold text-white/90">{$_('themeEditor.savedThemes')}</h2>
        <Button variant="secondary" size="md" onclick={() => showImportDialog = true}>
          {$_('themeEditor.actions.import')}
        </Button>
      </div>
      
      {#if themes.length === 0}
        <div class="panel text-center py-12">
          <p class="text-white/60 text-lg">{$_('themeEditor.noThemesYet')}</p>
        </div>
      {:else}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each themes as theme}
            <div class="theme-card">
              <div class="theme-card-preview" style="background: {theme.colors.background};">
                <div class="flex gap-2 justify-center">
                  <div 
                    class="mini-stone" 
                    style="background: {theme.colors.stonePlayer1};"
                  ></div>
                  <div 
                    class="mini-stone" 
                    style="background: {theme.colors.stonePlayer2};"
                  ></div>
                </div>
                <div 
                  class="color-swatch"
                  style="background: {theme.colors.gradientPrimary};"
                ></div>
              </div>
              
              <div class="theme-card-content">
                <h3 class="theme-card-title">{theme.name}</h3>
                <p class="theme-card-date">
                  {$_('themeEditor.updatedDate', { values: { date: new Date(theme.updatedAt).toLocaleDateString() } })}
                </p>
                
                <div class="theme-card-actions">
                  <button 
                    class="action-btn action-btn-apply"
                    onclick={(e) => { e.stopPropagation(); applyTheme(theme); }}
                  >
                    {$_('themeEditor.actions.apply')}
                  </button>
                  <button 
                    class="action-btn action-btn-edit"
                    onclick={(e) => { e.stopPropagation(); loadThemeForEditing(theme); }}
                  >
                    {$_('themeEditor.actions.edit')}
                  </button>
                  <button 
                    class="action-btn action-btn-duplicate"
                    onclick={(e) => { e.stopPropagation(); duplicateTheme(theme.id); }}
                  >
                    {$_('themeEditor.actions.duplicate')}
                  </button>
                  <button 
                    class="action-btn action-btn-export"
                    onclick={(e) => { e.stopPropagation(); exportTheme(theme.id); }}
                  >
                    {$_('themeEditor.actions.export')}
                  </button>
                  <button 
                    class="action-btn action-btn-delete"
                    onclick={(e) => { e.stopPropagation(); deleteTheme(theme.id); }}
                  >
                    {$_('themeEditor.actions.delete')}
                  </button>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
    
    <!-- Back Button -->
    <div class="flex justify-center mt-12">
      <Button variant="primary" size="lg" onclick={() => goto('/')}>
        {$_('game.menu.back')}
      </Button>
    </div>
  </div>
</div>

<!-- Import Dialog -->
{#if showImportDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="modal-overlay" onclick={() => showImportDialog = false}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <h2 class="modal-title">{$_('themeEditor.importDialog.title')}</h2>
      <p class="modal-description">{$_('themeEditor.importDialog.description')}</p>
      
      <textarea
        bind:value={importJson}
        placeholder={$_('themeEditor.importDialog.placeholder')}
        class="import-textarea"
        rows="10"
      ></textarea>
      
      <div class="modal-actions">
        <Button variant="primary" size="md" onclick={importTheme}>
          {$_('themeEditor.importDialog.import')}
        </Button>
        <Button variant="secondary" size="md" onclick={() => showImportDialog = false}>
          {$_('themeEditor.importDialog.cancel')}
        </Button>
      </div>
    </div>
  </div>
{/if}

<style>
  .panel {
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(12px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 1rem;
    padding: 2rem;
  }
  
  .section-title {
    font-size: 1.5rem;
    font-weight: bold;
    color: rgba(255, 255, 255, 0.9);
    margin-bottom: 1.5rem;
  }
  
  .form-group {
    margin-bottom: 1.5rem;
  }
  
  .form-label {
    display: block;
    font-size: 0.875rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.9);
    margin-bottom: 0.5rem;
  }
  
  .text-input {
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: white;
    font-size: 1rem;
    transition: all 0.2s ease;
  }
  
  .text-input:focus {
    outline: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .select-input {
    width: 100%;
    padding: 0.75rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: white;
    font-size: 1rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .select-input:focus {
    outline: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .select-input option {
    background: #1a1a1a;
    color: white;
  }
  
  .color-grid {
    display: grid;
    gap: 1.5rem;
  }
  
  .message {
    padding: 1rem 1.5rem;
    border-radius: 0.5rem;
    font-weight: 600;
    text-align: center;
  }
  
  .success-message {
    background: rgba(34, 197, 94, 0.2);
    border: 1px solid rgba(34, 197, 94, 0.5);
    color: rgb(134, 239, 172);
  }
  
  .error-message {
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid rgba(239, 68, 68, 0.5);
    color: rgb(252, 165, 165);
  }
  
  .theme-card {
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.75rem;
    overflow: hidden;
    transition: all 0.3s ease;
  }
  
  .theme-card:hover {
    transform: translateY(-4px);
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .theme-card-preview {
    height: 120px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 1rem;
  }
  
  .mini-stone {
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
  }
  
  .color-swatch {
    width: 100%;
    height: 0.5rem;
    border-radius: 0.25rem;
  }
  
  .theme-card-content {
    padding: 1rem;
  }
  
  .theme-card-title {
    font-size: 1.125rem;
    font-weight: bold;
    color: white;
    margin-bottom: 0.25rem;
  }
  
  .theme-card-date {
    font-size: 0.75rem;
    color: rgba(255, 255, 255, 0.5);
    margin-bottom: 1rem;
  }
  
  .theme-card-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }
  
  .action-btn {
    padding: 0.375rem 0.75rem;
    border-radius: 0.375rem;
    border: 1px solid rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.1);
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .action-btn:hover {
    background: rgba(255, 255, 255, 0.2);
    transform: translateY(-1px);
  }
  
  .action-btn-apply {
    background: rgba(34, 197, 94, 0.2);
    border-color: rgba(34, 197, 94, 0.5);
  }
  
  .action-btn-apply:hover {
    background: rgba(34, 197, 94, 0.3);
  }
  
  .action-btn-delete {
    background: rgba(239, 68, 68, 0.2);
    border-color: rgba(239, 68, 68, 0.5);
  }
  
  .action-btn-delete:hover {
    background: rgba(239, 68, 68, 0.3);
  }
  
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 1rem;
  }
  
  .modal-content {
    background: linear-gradient(135deg, rgba(26, 0, 51, 0.95) 0%, rgba(51, 0, 102, 0.95) 100%);
    border: 1px solid rgba(255, 255, 255, 0.3);
    border-radius: 1rem;
    padding: 2rem;
    max-width: 600px;
    width: 100%;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  .modal-title {
    font-size: 1.5rem;
    font-weight: bold;
    color: white;
    margin-bottom: 0.5rem;
  }
  
  .modal-description {
    color: rgba(255, 255, 255, 0.7);
    margin-bottom: 1rem;
  }
  
  .import-textarea {
    width: 100%;
    padding: 1rem;
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: white;
    font-family: 'Courier New', monospace;
    font-size: 0.875rem;
    resize: vertical;
    margin-bottom: 1rem;
  }
  
  .import-textarea:focus {
    outline: none;
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .modal-actions {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
  }
  
  @media (max-width: 1024px) {
    .panel.sticky {
      position: static;
    }
  }
</style>
