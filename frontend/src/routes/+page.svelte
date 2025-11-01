<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import Button from '$lib/components/Button.svelte';
  import logo from '$lib/assets/logo/gomoku-logo.png';
</script>

<div class="flex items-center justify-center min-h-[calc(100vh-4rem)]">
  <div class="relative z-10 text-center px-4 max-w-2xl animate-fade-in">
    <div class="mb-8">
      <img src={logo} alt="Gomoku Logo" class="w-32 h-auto mx-auto drop-shadow-2xl animate-float" />
    </div>
    
    <h1 class="text-7xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-4 tracking-tight">
      {$_('home.title')}
    </h1>
    
    <p class="text-xl text-white/90 mb-12 font-light tracking-wide">
      {$_('home.subtitle')}
    </p>
    
    <div class="flex flex-col gap-4 max-w-xs mx-auto mb-8">
      <Button variant="primary" size="lg" onclick={() => goto('/game')}>
        {$_('home.play')}
      </Button>
      
      <Button variant="secondary" size="md" onclick={() => goto('/settings')}>
        {$_('home.settings')}
      </Button>
      
      <Button variant="ghost" size="md" onclick={() => goto('/credits')}>
        {$_('home.credits')}
      </Button>
    </div>
  </div>
</div>