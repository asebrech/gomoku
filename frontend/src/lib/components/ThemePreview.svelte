<script lang="ts">
  import type { ThemeColors } from '$lib/theme/themeStore';
  
  interface Props {
    theme: ThemeColors;
  }
  
  let { theme }: Props = $props();
</script>

<div class="theme-preview-container">
  <h3 class="preview-title">Live Preview</h3>
  
  <div 
    class="preview-window" 
    style="background: {theme.background};"
  >
    <!-- Header -->
    <div 
      class="preview-header"
      style="background: {theme.titleBg}; color: {theme.textPrimary};"
    >
      <div class="preview-logo" style="color: {theme.primary};">GOMOKU</div>
    </div>
    
    <!-- Content Area -->
    <div class="preview-content">
      <!-- Surface Card -->
      <div 
        class="preview-card"
        style="background: {theme.surface}; border-color: {theme.primary}33;"
      >
        <h4 style="color: {theme.textPrimary};">Sample Card</h4>
        <p style="color: {theme.textSecondary};">This is how text will appear</p>
        
        <!-- Buttons -->
        <div class="preview-buttons">
          <button 
            class="preview-button"
            style="background: {theme.buttonNormal}; color: {theme.textPrimary};"
          >
            Normal
          </button>
          <button 
            class="preview-button"
            style="background: {theme.buttonHovered}; color: {theme.textPrimary};"
          >
            Hovered
          </button>
          <button 
            class="preview-button"
            style="background: {theme.buttonPressed}; color: {theme.textPrimary};"
          >
            Pressed
          </button>
        </div>
      </div>
      
      <!-- Game Stones Preview -->
      <div class="preview-stones">
        <div class="stone-label" style="color: {theme.textPrimary};">Game Stones:</div>
        <div class="stones-row">
          <div 
            class="preview-stone"
            style="background: {theme.stonePlayer1}; box-shadow: 0 0 10px {theme.stonePlayer1}88;"
          >
            <span class="stone-label-text" style="color: {theme.textPrimary};">P1</span>
          </div>
          <div 
            class="preview-stone"
            style="background: {theme.stonePlayer2}; box-shadow: 0 0 10px {theme.stonePlayer2}88;"
          >
            <span class="stone-label-text" style="color: {theme.textPrimary};">P2</span>
          </div>
        </div>
      </div>
      
      <!-- Gradient Examples -->
      <div class="preview-gradients">
        <div 
          class="gradient-bar"
          style="background: {theme.gradientPrimary};"
        >
          <span style="color: white; text-shadow: 0 1px 2px rgba(0,0,0,0.5);">Primary Gradient</span>
        </div>
        <div 
          class="gradient-bar"
          style="background: {theme.gradientSecondary};"
        >
          <span style="color: white; text-shadow: 0 1px 2px rgba(0,0,0,0.5);">Secondary Gradient</span>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  .theme-preview-container {
    width: 100%;
  }
  
  .preview-title {
    font-size: 1.25rem;
    font-weight: bold;
    color: rgba(255, 255, 255, 0.9);
    margin-bottom: 1rem;
  }
  
  .preview-window {
    border-radius: 1rem;
    overflow: hidden;
    border: 2px solid rgba(255, 255, 255, 0.2);
    min-height: 400px;
  }
  
  .preview-header {
    padding: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(8px);
  }
  
  .preview-logo {
    font-size: 1.5rem;
    font-weight: 900;
    letter-spacing: 0.1em;
  }
  
  .preview-content {
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .preview-card {
    padding: 1.5rem;
    border-radius: 0.75rem;
    border: 1px solid;
  }
  
  .preview-card h4 {
    font-size: 1.125rem;
    font-weight: bold;
    margin-bottom: 0.5rem;
  }
  
  .preview-card p {
    font-size: 0.875rem;
    margin-bottom: 1rem;
  }
  
  .preview-buttons {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }
  
  .preview-button {
    padding: 0.5rem 1rem;
    border-radius: 0.5rem;
    border: none;
    font-weight: 600;
    font-size: 0.875rem;
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .preview-button:hover {
    transform: translateY(-1px);
  }
  
  .preview-stones {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .stone-label {
    font-size: 0.875rem;
    font-weight: 600;
  }
  
  .stones-row {
    display: flex;
    gap: 1rem;
  }
  
  .preview-stone {
    width: 3rem;
    height: 3rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 0.75rem;
  }
  
  .stone-label-text {
    font-size: 0.75rem;
    font-weight: bold;
  }
  
  .preview-gradients {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  
  .gradient-bar {
    padding: 0.75rem;
    border-radius: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 600;
    font-size: 0.875rem;
  }
  
  @media (max-width: 640px) {
    .preview-buttons {
      flex-direction: column;
    }
    
    .preview-button {
      width: 100%;
    }
  }
</style>
