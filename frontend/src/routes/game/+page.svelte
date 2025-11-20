<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import Button from '$lib/components/Button.svelte';
  import { currentTheme } from '$lib/theme/themeStore';
</script>

<div class="h-full w-full flex items-center justify-center">
  <div class="relative z-10 text-center px-4 max-w-2xl animate-fade-in">
    <h1 class="text-6xl font-black text-transparent bg-clip-text mb-4 tracking-tight"
        style="background-image: {$currentTheme.gradientPrimary}; -webkit-background-clip: text; background-clip: text;">
      {$_('game.menu.title')}
    </h1>
    
    <p class="text-lg text-white/80 mb-12 font-light">
      Choose your game mode
    </p>
    
    <div class="flex flex-col gap-4 w-full max-w-xs sm:max-w-sm mx-auto mb-8">
      <Button variant="primary" size="lg" onclick={() => goto('/game/player-vs-ai')}>
        {$_('game.menu.playerVsAi')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/game/player-vs-player')}>
        {$_('game.menu.playerVsPlayer')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/game/ai-vs-ai')}>
        {$_('game.menu.aiVsAi')}
      </Button>
      
      <div class="mt-4">
        <Button variant="primary" size="lg" onclick={() => goto('/')}>
          {$_('game.menu.back')}
        </Button>
      </div>
    </div>
  </div>
</div>
