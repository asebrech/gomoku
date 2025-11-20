<script lang="ts">
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import Button from '$lib/components/Button.svelte';
  import { currentTheme } from '$lib/theme/themeStore';
  import logo from '$lib/assets/logo/gomoku-logo.png';
</script>

<div class="h-full w-full flex items-center justify-center">
  <div class="relative z-10 text-center px-4 max-w-2xl animate-fade-in">
    <div class="mb-12">
      <img src={logo} alt="Gomoku Logo" class="w-64 sm:w-64 md:w-80 h-auto mx-auto drop-shadow-2xl animate-float" />
    </div>
    
    <div class="flex flex-col gap-4 w-full max-w-xs sm:max-w-sm mx-auto mb-8">
      <Button variant="primary" size="lg" onclick={() => goto('/game')}>
        {$_('home.play')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/history')}>
        <svg class="w-5 h-5 inline-block mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        {$_('history.title')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/settings')}>
        {$_('home.settings')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/themes')}>
        {$_('home.themeEditor')}
      </Button>
      
      <Button variant="primary" size="lg" onclick={() => goto('/credits')}>
        {$_('home.credits')}
      </Button>
    </div>
  </div>
</div>