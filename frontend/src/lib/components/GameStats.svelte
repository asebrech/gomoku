<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    gameMode: string;
    boardSize: number;
    aiDepth?: number;
    currentPlayer: string;
    totalMoves: number;
    player1Captures?: number;
    player2Captures?: number;
    totalThinkingTime?: number;
    lastMoveTime?: number;
    aiMoveCount?: number;
    lastDepthReached?: number;
    lastNodesSearched?: number;
    lastAIScore?: number;
  }
  
  let {
    gameMode,
    boardSize,
    aiDepth,
    currentPlayer,
    totalMoves,
    player1Captures = 0,
    player2Captures = 0,
    totalThinkingTime = 0,
    lastMoveTime = 0,
    aiMoveCount = 0,
    lastDepthReached = 0,
    lastNodesSearched = 0,
    lastAIScore = 0
  }: Props = $props();
  
  function formatTime(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
  
  function formatNodes(nodes: number): string {
    if (nodes < 1000) return `${nodes}`;
    if (nodes < 1000000) return `${(nodes / 1000).toFixed(1)}K`;
    return `${(nodes / 1000000).toFixed(1)}M`;
  }
</script>

<div class="flex items-center justify-center w-full p-8">
  <div 
    class="rounded-lg p-6 min-w-[300px]"
    style="background: {$currentTheme.background}99; border: 3px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};"
  >
    <h2 
      class="text-2xl font-bold text-center mb-6"
      style="color: {$currentTheme.primary};"
    >
      Game Stats
    </h2>
  
  <div class="space-y-3">
    <!-- Game Mode -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">Game Mode:</span>
      <span class="font-bold" style="color: {$currentTheme.secondary};">
        {gameMode}
      </span>
    </div>
    
    <!-- Board Size -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">Board Size:</span>
      <span class="font-bold" style="color: {$currentTheme.secondary};">
        {boardSize}x{boardSize}
      </span>
    </div>
    
    {#if aiDepth}
      <!-- AI Depth -->
      <div class="flex justify-between items-center">
        <span class="text-white/80 font-medium">AI Max Depth:</span>
        <span class="font-bold" style="color: {$currentTheme.secondary};">
          {aiDepth}
        </span>
      </div>
    {/if}
    
    <div class="border-t my-3" style="border-color: {$currentTheme.primary}33;"></div>
    
    <!-- Current Player -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">Current Turn:</span>
      <span class="font-bold" style="color: {$currentTheme.primary};">
        {currentPlayer}
      </span>
    </div>
    
    <!-- Total Moves -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">Total Moves:</span>
      <span class="font-bold" style="color: {$currentTheme.secondary};">
        {totalMoves}
      </span>
    </div>
    
    <!-- Captures -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">Black Captures:</span>
      <span class="font-bold" style="color: {$currentTheme.stonePlayer1};">
        {player1Captures}
      </span>
    </div>
    
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium">White Captures:</span>
      <span class="font-bold" style="color: {$currentTheme.stonePlayer2};">
        {player2Captures}
      </span>
    </div>
    
    {#if aiDepth !== undefined}
      <div class="border-t my-3" style="border-color: {$currentTheme.primary}33;"></div>
      
      <!-- Actual Depth Reached -->
      {#if lastDepthReached > 0}
        <div class="flex justify-between items-center">
          <span class="text-white/80 font-medium">Depth Reached:</span>
          <span class="font-bold text-xl" style="color: {$currentTheme.accent};">
            {lastDepthReached}
          </span>
        </div>
      {/if}
      
      <!-- Last Move Time -->
      <div class="flex justify-between items-center">
        <span class="text-white/80 font-medium">AI Last Move:</span>
        <span class="font-bold text-xl" style="color: {$currentTheme.secondary};">
          {lastMoveTime > 0 ? formatTime(lastMoveTime) : '--'}
        </span>
      </div>
      
      <!-- Nodes Searched -->
      {#if lastNodesSearched > 0}
        <div class="flex justify-between items-center">
          <span class="text-white/80 font-medium text-sm">Nodes Searched:</span>
          <span class="font-bold" style="color: {$currentTheme.secondary};">
            {formatNodes(lastNodesSearched)}
          </span>
        </div>
      {/if}
      
      <!-- AI Evaluation Score -->
      {#if lastAIScore !== 0}
        <div class="flex justify-between items-center">
          <span class="text-white/80 font-medium text-sm">Evaluation:</span>
          <span class="font-bold" style="color: {lastAIScore > 0 ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2};">
            {lastAIScore > 0 ? '+' : ''}{lastAIScore}
          </span>
        </div>
      {/if}
    {/if}
  </div>
</div>
</div>
