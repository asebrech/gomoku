<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
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
      title: 'Core Colors',
      colors: [
        { key: 'primary', label: 'Primary', description: 'Main brand color' },
        { key: 'secondary', label: 'Secondary', description: 'Secondary accent' },
        { key: 'accent', label: 'Accent', description: 'Highlight color' },
      ]
    },
    {
      title: 'Background Colors',
      colors: [
        { key: 'background', label: 'Background', description: 'Main background' },
        { key: 'surface', label: 'Surface', description: 'Card/panel background' },
      ]
    },
    {
      title: 'Text Colors',
      colors: [
        { key: 'textPrimary', label: 'Primary Text', description: 'Main text color' },
        { key: 'textSecondary', label: 'Secondary Text', description: 'Muted text' },
      ]
    },
    {
      title: 'Button States',
      colors: [
        { key: 'buttonNormal', label: 'Normal', description: 'Default button' },
        { key: 'buttonHovered', label: 'Hovered', description: 'Hover state' },
        { key: 'buttonPressed', label: 'Pressed', description: 'Active/pressed state' },
      ]
    },
    {
      title: 'Game Elements',
      colors: [
        { key: 'stonePlayer1', label: 'Player 1 Stone', description: 'First player stone' },
        { key: 'stonePlayer2', label: 'Player 2 Stone', description: 'Second player stone' },
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
      errorMessage = 'Please enter a theme name';
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
      successMessage = 'Theme updated successfully!';
    } else {
      // Create new theme
      customThemes.add({
        name: themeName,
        colors: completeTheme
      });
      successMessage = 'Theme created successfully!';
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
    successMessage = `Applied theme: ${theme.name}`;
    setTimeout(() => successMessage = '', 3000);
  }
  
  function deleteTheme(id: string) {
    if (confirm('Are you sure you want to delete this theme?')) {
      customThemes.delete(id);
      if (editingThemeId === id) {
        editingThemeId = null;
        themeName = 'My Custom Theme';
        editingTheme = { ...synthwaveTheme };
      }
      successMessage = 'Theme deleted';
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
      successMessage = 'Theme exported!';
      setTimeout(() => successMessage = '', 3000);
    } catch (error) {
      errorMessage = 'Failed to export theme';
      setTimeout(() => errorMessage = '', 3000);
    }
  }
  
  function importTheme() {
    try {
      const imported = customThemes.importTheme(importJson);
      successMessage = `Imported theme: ${imported.name}`;
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
      successMessage = 'Theme duplicated!';
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
        Theme Editor
      </h1>
      <p class="text-sm text-white/60">
        Create and customize your own themes
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
          <h2 class="section-title">Theme Settings</h2>
          
          <div class="form-group">
            <label for="theme-name" class="form-label">Theme Name</label>
            <input
              id="theme-name"
              type="text"
              bind:value={themeName}
              placeholder="Enter theme name..."
              class="text-input"
            />
          </div>
          
          <div class="form-group">
            <label for="existing-theme-select" class="form-label">Edit Existing Theme</label>
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
              <option value="">-- Select a theme to edit --</option>
              {#each themes as theme}
                <option value={theme.id} selected={theme.id === editingThemeId}>
                  {theme.name}
                </option>
              {/each}
            </select>
          </div>
          
          <div class="form-group">
            <label for="template-select" class="form-label">Start from Template</label>
            <select
              id="template-select"
              bind:value={selectedTemplate}
              onchange={() => loadTemplate(selectedTemplate)}
              class="select-input"
            >
              {#each Object.keys(themeTemplates) as templateKey}
                <option value={templateKey}>
                  {themeTemplates[templateKey as keyof typeof themeTemplates].name}
                </option>
              {/each}
            </select>
          </div>
          
          <div class="flex gap-3">
            <Button variant="primary" size="md" onclick={saveTheme}>
              {editingThemeId ? 'Update Theme' : 'Save Theme'}
            </Button>
            <Button variant="secondary" size="md" onclick={resetEditor}>
              Reset
            </Button>
          </div>
        </div>
        
        <!-- Color Pickers -->
        {#each colorCategories as category}
          <div class="panel">
            <h3 class="section-title">{category.title}</h3>
            <div class="color-grid">
              {#each category.colors as colorDef}
                <ColorPicker
                  label={colorDef.label}
                  description={colorDef.description}
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
              {editingThemeId ? 'Update Theme' : 'Save Theme'}
            </Button>
            <Button variant="secondary" size="lg" onclick={resetEditor}>
              Reset
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
        <h2 class="text-3xl font-bold text-white/90">Saved Themes</h2>
        <Button variant="secondary" size="md" onclick={() => showImportDialog = true}>
          Import Theme
        </Button>
      </div>
      
      {#if themes.length === 0}
        <div class="panel text-center py-12">
          <p class="text-white/60 text-lg">No custom themes yet. Create your first one!</p>
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
                  Updated: {new Date(theme.updatedAt).toLocaleDateString()}
                </p>
                
                <div class="theme-card-actions">
                  <button 
                    class="action-btn action-btn-apply"
                    onclick={(e) => { e.stopPropagation(); applyTheme(theme); }}
                  >
                    Apply
                  </button>
                  <button 
                    class="action-btn action-btn-edit"
                    onclick={(e) => { e.stopPropagation(); loadThemeForEditing(theme); }}
                  >
                    Edit
                  </button>
                  <button 
                    class="action-btn action-btn-duplicate"
                    onclick={(e) => { e.stopPropagation(); duplicateTheme(theme.id); }}
                  >
                    Duplicate
                  </button>
                  <button 
                    class="action-btn action-btn-export"
                    onclick={(e) => { e.stopPropagation(); exportTheme(theme.id); }}
                  >
                    Export
                  </button>
                  <button 
                    class="action-btn action-btn-delete"
                    onclick={(e) => { e.stopPropagation(); deleteTheme(theme.id); }}
                  >
                    Delete
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
        Back to Menu
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
      <h2 class="modal-title">Import Theme</h2>
      <p class="modal-description">Paste your theme JSON below:</p>
      
      <textarea
        bind:value={importJson}
        placeholder="Paste theme JSON here..."
        class="import-textarea"
        rows="10"
      ></textarea>
      
      <div class="modal-actions">
        <Button variant="primary" size="md" onclick={importTheme}>
          Import
        </Button>
        <Button variant="secondary" size="md" onclick={() => showImportDialog = false}>
          Cancel
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
