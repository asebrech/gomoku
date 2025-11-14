<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import Button from '$lib/components/Button.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  import { currentTheme } from '$lib/theme/themeStore';
  
  let boardSize = $state($gameSettings.boardSize);
  let aiDepth = $state($gameSettings.aiDepth);
  let aiMaxThinkingTime = $state($gameSettings.aiMaxThinkingTime);
  let showDoubleThree = $state($gameSettings.showDoubleThree);
  let showAIHint = $state($gameSettings.showAIHint);
  
  // Auto-save settings whenever any value changes
  $effect(() => {
    gameSettings.set({
      boardSize,
      winCondition: 5, // Always 5 in a row
      aiDepth,
      aiMaxThinkingTime,
      showDoubleThree,
      showAIHint
    });
  });
  
  function resetToDefaults() {
    gameSettings.reset();
    boardSize = $gameSettings.boardSize;
    aiDepth = $gameSettings.aiDepth;
    aiMaxThinkingTime = $gameSettings.aiMaxThinkingTime;
    showDoubleThree = $gameSettings.showDoubleThree;
    showAIHint = $gameSettings.showAIHint;
  }
  
  // Validate board size
  $effect(() => {
    if (boardSize < 9) boardSize = 9;
    if (boardSize > 19) boardSize = 19;
  });
  
  // Validate AI depth
  $effect(() => {
    if (aiDepth < 2) aiDepth = 2;
    if (aiDepth > 25) aiDepth = 25;
  });
  
  // Validate AI max thinking time (in milliseconds)
  $effect(() => {
    if (aiMaxThinkingTime < 0) aiMaxThinkingTime = 0;
    if (aiMaxThinkingTime > 5000) aiMaxThinkingTime = 5000;
    // Round to nearest 50ms
    aiMaxThinkingTime = Math.round(aiMaxThinkingTime / 50) * 50;
  });
</script>

<div class="h-full w-full overflow-y-auto">
  <div class="max-w-2xl mx-auto px-4 py-8">
    <div class="text-center mb-8">
      <h1 class="text-5xl font-black text-transparent bg-clip-text mb-4"
          style="background-image: {$currentTheme.gradientPrimary}; -webkit-background-clip: text; background-clip: text;">
        {$_('settings.title')}
      </h1>
      <p class="text-sm text-white/60">
        {$_('settings.autoSave')}
      </p>
    </div>
    
    <div class="bg-white/10 backdrop-blur-md border border-white/20 rounded-2xl p-8 space-y-8">
      <!-- Board Size -->
      <div class="setting-section">
        <Slider
          id="boardSize"
          bind:value={boardSize}
          min={9}
          max={19}
          step={1}
          label={$_('settings.boardSize')}
          unit="x{boardSize}"
          showInput={true}
        />
      </div>
      
      <!-- AI Depth -->
      <div class="setting-section">
        <Slider
          id="aiDepth"
          bind:value={aiDepth}
          min={2}
          max={25}
          step={1}
          label={$_('settings.aiDepth')}
          showInput={true}
        />
      </div>
      
      <!-- AI Max Thinking Time -->
      <div class="setting-section">
        <Slider
          id="aiMaxThinkingTime"
          bind:value={aiMaxThinkingTime}
          min={0}
          max={5000}
          step={50}
          label={$_('settings.aiMaxThinkingTime')}
          unit="ms"
          showInput={true}
        />
      </div>
      
      <!-- Divider -->
      <div class="border-t border-white/20 my-4"></div>
      
      <!-- Visual Aids Section -->
      <div class="setting-section">
        <h3 class="text-xl font-bold text-white/90 mb-4">{$_('settings.visualAids')}</h3>
        
        <!-- Double-Three Display -->
        <div class="flex items-center justify-between py-3">
          <div>
            <label for="show-double-three" class="text-white/90 font-medium cursor-pointer">
              {$_('settings.showDoubleThree')}
            </label>
            <p class="setting-hint">
              {$_('settings.showDoubleThreeHint')}
            </p>
          </div>
          <Toggle 
            id="show-double-three"
            bind:checked={showDoubleThree}
          />
        </div>
        
        <!-- AI Hint Display -->
        <div class="flex items-center justify-between py-3">
          <div>
            <label for="show-ai-hint" class="text-white/90 font-medium cursor-pointer">
              {$_('settings.showAIHint')}
            </label>
            <p class="setting-hint">
              {$_('settings.showAIHintDescription')}
            </p>
          </div>
          <Toggle 
            id="show-ai-hint"
            bind:checked={showAIHint}
          />
        </div>
      </div>
    </div>
    
    <!-- Buttons -->
    <div class="flex justify-center gap-4 mt-8 flex-wrap">
      <Button variant="primary" size="lg" onclick={() => goto('/')}>
        {$_('game.menu.back')}
      </Button>
      <Button variant="primary" size="lg" onclick={() => goto('/themes')}>
        {$_('settings.themeEditor')}
      </Button>
      <Button variant="primary" size="lg" onclick={resetToDefaults}>
        {$_('settings.resetToDefaults')}
      </Button>
    </div>
  </div>
</div>

<style>
  .setting-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .setting-hint {
    font-size: 0.875rem;
    color: rgba(255, 255, 255, 0.6);
    margin-left: 0.25rem;
  }
</style>
