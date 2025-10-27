<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import Button from '$lib/components/Button.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  
  let boardSize = $state($gameSettings.boardSize);
  let aiDepth = $state($gameSettings.aiDepth);
  let aiMaxThinkingTime = $state($gameSettings.aiMaxThinkingTime);
  
  let saveStatus = $state('');
  
  function saveSettings() {
    gameSettings.set({
      boardSize,
      winCondition: 5, // Always 5 in a row
      aiDepth,
      aiMaxThinkingTime
    });
    saveStatus = 'Settings saved!';
    setTimeout(() => {
      saveStatus = '';
    }, 2000);
  }
  
  function resetToDefaults() {
    gameSettings.reset();
    boardSize = $gameSettings.boardSize;
    aiDepth = $gameSettings.aiDepth;
    aiMaxThinkingTime = $gameSettings.aiMaxThinkingTime;
    saveStatus = 'Reset to defaults!';
    setTimeout(() => {
      saveStatus = '';
    }, 2000);
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

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-2xl mx-auto px-4">
    <div class="text-center mb-8">
      <h1 class="text-5xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-4">
        {$_('settings.title')}
      </h1>
      {#if saveStatus}
        <p class="text-lg font-semibold" style="color: #00FFFF;">
          {saveStatus}
        </p>
      {/if}
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
          label="Board Size"
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
          label="AI Search Depth"
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
          label="AI Max Thinking Time"
          unit="ms"
          showInput={true}
        />
      </div>
    </div>
    
    <!-- Buttons -->
    <div class="flex justify-center gap-4 mt-8 flex-wrap">
      <Button variant="ghost" size="md" onclick={() => goto('/')}>
        {$_('game.menu.back')}
      </Button>
      <Button variant="secondary" size="md" onclick={resetToDefaults}>
        Reset to Defaults
      </Button>
      <Button variant="primary" size="lg" onclick={saveSettings}>
        Save Settings
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
