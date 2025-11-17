<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  import type { MatchData } from '$lib/stores/matchHistory';
  import { _ } from 'svelte-i18n';
  
  interface Props {
    match: MatchData;
    onReplay?: () => void;
    onResume?: () => void;
    onDelete?: () => void;
  }
  
  let { match, onReplay, onResume, onDelete }: Props = $props();
  
  // Format date and time
  const formatDate = (date: Date) => {
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  };
  
  // Get game mode display
  const gameMode = $derived(() => {
    if (match.player1Type === 'ai' && match.player2Type === 'ai') return $_('history.match.gameMode.aiVsAi');
    if (match.player1Type === 'human' && match.player2Type === 'human') return $_('history.match.gameMode.playerVsPlayer');
    return $_('history.match.gameMode.playerVsAi');
  });
  
  // Get status display
  const statusDisplay = $derived(() => {
    if (match.status === 'ongoing') return $_('history.match.status.ongoing');
    if (match.winner === null) return $_('history.match.status.draw');
    const winnerName = match.winner === 1 ? match.player1Name : match.player2Name;
    return $_('history.match.status.won', { values: { winner: winnerName } });
  });
  
  // Get status color
  const statusColor = $derived(() => {
    if (match.status === 'ongoing') return $currentTheme.warning || '#FFA500';
    if (match.winner === null) return $currentTheme.textSecondary;
    return $currentTheme.success || '#10B981';
  });
  
  // Clean player names (remove (Black)/(White) suffixes)
  const cleanPlayerName = (name: string) => {
    return name.replace(/\s*\((Black|White)\)\s*/i, '').trim();
  };
</script>

<div 
  class="rounded-lg p-4 backdrop-blur-md border transition-all duration-200 hover:scale-[1.02]"
  style="
    background: {$currentTheme.surface}20;
    border-color: {$currentTheme.primary}40;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  "
>
  <!-- Header: Date and Status -->
  <div class="flex justify-between items-start mb-3">
    <div>
      <p class="text-sm opacity-70 text-white">
        {formatDate(match.date)}
      </p>
      <p class="text-xs opacity-50 mt-0.5 text-white">
        {gameMode()}
      </p>
    </div>
    <div 
      class="px-3 py-1 rounded-full text-sm font-semibold"
      style="
        background: {statusColor()}20;
        color: {statusColor()};
        border: 1px solid {statusColor()}40;
      "
    >
      {statusDisplay()}
    </div>
  </div>
  
  <!-- Players -->
  <div class="mb-3 space-y-2">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div 
          class="w-4 h-4 rounded-full"
          style="background: {$currentTheme.stonePlayer1};"
        ></div>
        <span class="text-sm font-medium text-white">
          {cleanPlayerName(match.player1Name)}
        </span>
        {#if match.player1Type === 'ai'}
          <span class="text-xs opacity-50 text-white">
            (AI)
          </span>
        {/if}
      </div>
      <span class="text-sm font-bold text-white">
        {match.player1Captures} 
        <span class="text-xs opacity-70">{$_('history.match.info.captures')}</span>
      </span>
    </div>
    
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div 
          class="w-4 h-4 rounded-full"
          style="background: {$currentTheme.stonePlayer2};"
        ></div>
        <span class="text-sm font-medium text-white">
          {cleanPlayerName(match.player2Name)}
        </span>
        {#if match.player2Type === 'ai'}
          <span class="text-xs opacity-50 text-white">
            (AI)
          </span>
        {/if}
      </div>
      <span class="text-sm font-bold text-white">
        {match.player2Captures}
        <span class="text-xs opacity-70">{$_('history.match.info.captures')}</span>
      </span>
    </div>
  </div>
  
  <!-- Game Info -->
  <div class="flex gap-4 mb-3 text-xs opacity-70 text-white">
    <span>{$_('history.match.info.board')}: {match.boardSize}x{match.boardSize}</span>
    <span>{$_('history.match.info.moves')}: {match.totalMoves}</span>
    {#if match.aiDepth}
      <span>{$_('history.match.info.aiDepth')}: {match.aiDepth}</span>
    {/if}
  </div>
  
  <!-- Actions -->
  <div class="flex gap-2">
    {#if match.status === 'ongoing'}
      <Button 
        variant="primary" 
        size="sm" 
        fullWidth={true}
        onclick={onResume}
      >
        <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        {$_('history.match.resume')}
      </Button>
      <Button 
        variant="secondary" 
        size="sm"
        onclick={onReplay}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      </Button>
    {:else}
      <Button 
        variant="secondary" 
        size="sm" 
        fullWidth={true}
        onclick={onReplay}
      >
        <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        {$_('history.match.replay')}
      </Button>
    {/if}
    <Button 
      variant="ghost" 
      size="sm"
      onclick={onDelete}
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
    </Button>
  </div>
</div>
