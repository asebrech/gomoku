<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
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
    <svg width="24" height="24" class="inline-block flex-shrink-0 md:w-5 md:h-5">
      <defs>
        <radialGradient id="scorePlayer1Gradient">
          <stop offset="30%" stop-color={$currentTheme.stonePlayer1} stop-opacity="0.9" />
          <stop offset="100%" stop-color={$currentTheme.stonePlayer1} stop-opacity="1" />
        </radialGradient>
      </defs>
      <circle cx="12" cy="12" r="10" fill="url(#scorePlayer1Gradient)" />
    </svg>
    <div class="text-center">
      <div class="text-xs text-white/60 truncate max-w-[80px] md:max-w-none">{cleanPlayer1Name}</div>
      <div class="text-lg font-bold" style="color: {$currentTheme.stonePlayer1};">
        {player1Captures}
      </div>
    </div>
  </div>

  <!-- Total Moves in Center -->
  <div class="text-center flex-shrink-0">
    <div class="text-xs text-white/60">Turn</div>
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
      <div class="text-xs text-white/60 truncate max-w-[80px] md:max-w-none">{cleanPlayer2Name}</div>
      <div class="text-lg font-bold" style="color: {$currentTheme.stonePlayer2};">
        {player2Captures}
      </div>
    </div>
    <svg width="24" height="24" class="inline-block flex-shrink-0 md:w-5 md:h-5">
      <defs>
        <radialGradient id="scorePlayer2Gradient">
          <stop offset="30%" stop-color={$currentTheme.stonePlayer2} stop-opacity="0.9" />
          <stop offset="100%" stop-color={$currentTheme.stonePlayer2} stop-opacity="1" />
        </radialGradient>
      </defs>
      <circle cx="12" cy="12" r="10" fill="url(#scorePlayer2Gradient)" />
    </svg>
  </div>
</div>
