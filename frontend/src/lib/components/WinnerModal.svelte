<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  
  interface Props {
    winnerPlayer: number;
    player1Name: string;
    player2Name: string;
    onClose: () => void;
    onNewGame?: () => void;
  }
  
  let {
    winnerPlayer,
    player1Name,
    player2Name,
    onClose,
    onNewGame
  }: Props = $props();
  
  // Clean player names (remove (Black/White) suffix)
  const winnerName = $derived(winnerPlayer === 1 ? player1Name : player2Name);
  const cleanWinnerName = $derived(winnerName?.replace(/\s*\((Black|White)\)\s*/i, '').trim() || winnerName);
  const winnerColor = $derived(winnerPlayer === 1 ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2);
  
  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }
  
  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }
  
  function handleNewGame() {
    onClose();
    onNewGame?.();
  }
</script>

<!-- Modal Backdrop -->
<div 
  class="fixed inset-0 z-50 flex items-center justify-center backdrop-blur-sm animate-fade-in"
  style="background: rgba(0, 0, 0, 0.75);"
  onclick={handleBackdropClick}
  onkeydown={handleKeyDown}
  role="dialog"
  aria-modal="true"
  aria-labelledby="winner-title"
  tabindex="-1"
>
  <!-- Modal Content -->
  <div 
    class="relative rounded-xl p-6 max-w-sm w-full mx-4 animate-scale-in shadow-2xl"
    style="background: {$currentTheme.background}; border: 3px solid {winnerColor}; box-shadow: 0 0 30px {winnerColor}80, 0 10px 40px rgba(0,0,0,0.5);"
  >
    <!-- Close Button -->
    <button
      onclick={onClose}
      class="absolute top-3 right-3 text-white/60 hover:text-white transition-colors"
      aria-label="Close modal"
    >
      <svg width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M5 5L15 15M5 15L15 5"/>
      </svg>
    </button>
    
    <!-- Winner Display -->
    <div class="text-center mb-5">
      <!-- Decorative Stone SVG -->
      <div class="flex justify-center mb-3">
        <svg width="60" height="60" class="animate-bounce-slow">
          <defs>
            <radialGradient id="winnerGradient">
              <stop offset="30%" stop-color={winnerColor} stop-opacity="0.9" />
              <stop offset="100%" stop-color={winnerColor} stop-opacity="1" />
            </radialGradient>
            <filter id="winnerGlow">
              <feGaussianBlur stdDeviation="4" result="coloredBlur"/>
              <feMerge>
                <feMergeNode in="coloredBlur"/>
                <feMergeNode in="SourceGraphic"/>
              </feMerge>
            </filter>
          </defs>
          <circle
            cx="30"
            cy="30"
            r="26"
            fill="url(#winnerGradient)"
            filter="url(#winnerGlow)"
          />
        </svg>
      </div>
      
      <!-- Winner Text -->
      <h2 
        id="winner-title"
        class="text-3xl font-black mb-1 animate-pulse-slow"
        style="color: {winnerColor}; text-shadow: 0 0 20px {winnerColor}80;"
      >
        {cleanWinnerName}
      </h2>
      <p 
        class="text-xl font-bold"
        style="color: {$currentTheme.textPrimary};"
      >
        {$_('game.stats.wins')}
      </p>
    </div>
    
    <!-- Action Buttons -->
    <div class="flex flex-col gap-2 mt-5">
      {#if onNewGame}
        <Button 
          variant="primary" 
          size="md" 
          onclick={handleNewGame}
          fullWidth
        >
          {$_('game.stats.newGame')}
        </Button>
      {/if}
      <Button 
        variant="secondary" 
        size="md" 
        onclick={onClose}
        fullWidth
      >
        Close
      </Button>
    </div>
  </div>
</div>

<style>
  @keyframes fade-in {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  
  @keyframes scale-in {
    from {
      opacity: 0;
      transform: scale(0.9);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
  
  @keyframes bounce-slow {
    0%, 100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-10px);
    }
  }
  
  @keyframes pulse-slow {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: 0.8;
    }
  }
  
  .animate-fade-in {
    animation: fade-in 0.3s ease-out;
  }
  
  .animate-scale-in {
    animation: scale-in 0.3s ease-out;
  }
  
  .animate-bounce-slow {
    animation: bounce-slow 2s ease-in-out infinite;
  }
  
  .animate-pulse-slow {
    animation: pulse-slow 2s ease-in-out infinite;
  }
</style>
