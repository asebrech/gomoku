<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  import { _ } from 'svelte-i18n';
  
  interface Props {
    player1Name: string;
    player2Name: string;
    player1Captures: number;
    player2Captures: number;
    totalMoves: number;
  }
  
  let {
    player1Name,
    player2Name,
    player1Captures,
    player2Captures,
    totalMoves
  }: Props = $props();
  
  // Clean player names (remove (Black/White) suffix)
  const cleanPlayer1Name = $derived(player1Name?.replace(/\s*\((Black|White)\)\s*/i, '').trim() || player1Name);
  const cleanPlayer2Name = $derived(player2Name?.replace(/\s*\((Black|White)\)\s*/i, '').trim() || player2Name);
</script>

<!-- Responsive Horizontal Scoreboard -->
<div 
  class="rounded-lg p-3 w-full flex justify-between items-center"
  style="background: {$currentTheme.background}99; border: 2px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};"
>
  <!-- Player 1 Score -->
  <div class="flex items-center gap-2">
    <div class="w-5 h-5 md:w-6 md:h-6 rounded-full flex-shrink-0" style="background: {$currentTheme.stonePlayer1};"></div>
    <div class="text-center">
      <div class="text-xs text-white/60 truncate max-w-[80px] md:max-w-none">{cleanPlayer1Name} {$_('game.scoreboard.captures')}</div>
      <div class="text-lg font-bold" style="color: {$currentTheme.stonePlayer1};">
        {player1Captures}
      </div>
    </div>
  </div>

  <!-- Total Moves in Center -->
  <div class="text-center flex-shrink-0">
    <div class="text-xs text-white/60">{$_('game.stats.turn')}</div>
    <div 
      class="text-2xl font-bold"
      style="color: {$currentTheme.primary}; text-shadow: {$currentTheme.glowPrimary};"
    >
      {totalMoves}
    </div>
  </div>

  <!-- Player 2 Score -->
  <div class="flex items-center gap-2">
    <div class="text-center">
      <div class="text-xs text-white/60 truncate max-w-[80px] md:max-w-none">{cleanPlayer2Name} {$_('game.scoreboard.captures')}</div>
      <div class="text-lg font-bold" style="color: {$currentTheme.stonePlayer2};">
        {player2Captures}
      </div>
    </div>
    <div class="w-5 h-5 md:w-6 md:h-6 rounded-full flex-shrink-0" style="background: {$currentTheme.stonePlayer2};"></div>
  </div>
</div>
