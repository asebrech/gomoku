<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from './Button.svelte';
  import Toggle from './Toggle.svelte';
  import { _ } from 'svelte-i18n';
  
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
    // New props for controls
    showDoubleThree?: boolean;
    onToggleDoubleThree?: (value: boolean) => void;
    isPlaying?: boolean;
    isPaused?: boolean;
    isGameOver?: boolean;
    isFullAI?: boolean;
    hasHuman?: boolean;
    gameInstance?: any;
    onStartGame?: () => void;
    onPauseGame?: () => void;
    onResumeGame?: () => void;
    onUndoMove?: () => void;
    onResetGame?: () => void;
    onBack?: () => void;
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
    lastAIScore = 0,
    // Controls
    showDoubleThree = false,
    onToggleDoubleThree,
    isPlaying = false,
    isPaused = false,
    isGameOver = false,
    isFullAI = false,
    hasHuman = false,
    gameInstance,
    onStartGame,
    onPauseGame,
    onResumeGame,
    onUndoMove,
    onResetGame,
    onBack
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

<div class="flex flex-col items-center justify-start w-full h-full overflow-y-auto">
  <!-- Turn Display -->
  <div class="mb-2 flex-shrink-0">
    <p 
      class="text-2xl font-bold text-center"
      style="color: {$currentTheme.primary}; text-shadow: {$currentTheme.glowPrimary};"
    >
      Turn {totalMoves}
    </p>
  </div>
  
  <div 
    class="rounded-lg p-4 min-w-[280px] max-w-[320px] flex-shrink-0"
    style="background: {$currentTheme.background}99; border: 3px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};"
  >
    <h2 
      class="text-xl font-bold text-center mb-4"
      style="color: {$currentTheme.primary};"
    >
      Game Stats
    </h2>
  
  <div class="space-y-2">
    <!-- Game Mode -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium text-sm">Game Mode:</span>
      <span class="font-bold" style="color: {$currentTheme.secondary};">
        {gameMode}
      </span>
    </div>
    
    <!-- Board Size -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium text-sm">Board Size:</span>
      <span class="font-bold" style="color: {$currentTheme.secondary};">
        {boardSize}x{boardSize}
      </span>
    </div>
    
    {#if aiDepth}
      <!-- AI Depth -->
      <div class="flex justify-between items-center">
        <span class="text-white/80 font-medium text-sm">AI Max Depth:</span>
        <span class="font-bold" style="color: {$currentTheme.secondary};">
          {aiDepth}
        </span>
      </div>
    {/if}
    
    <div class="border-t my-2" style="border-color: {$currentTheme.primary}33;"></div>
    
    <!-- Current Player -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium text-sm">Current Turn:</span>
      <span class="font-bold" style="color: {$currentTheme.primary};">
        {currentPlayer}
      </span>
    </div>
    
    <!-- Captures -->
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium text-sm">Black Captures:</span>
      <span class="font-bold" style="color: {$currentTheme.stonePlayer1};">
        {player1Captures}
      </span>
    </div>
    
    <div class="flex justify-between items-center">
      <span class="text-white/80 font-medium text-sm">White Captures:</span>
      <span class="font-bold" style="color: {$currentTheme.stonePlayer2};">
        {player2Captures}
      </span>
    </div>
    
    {#if aiDepth !== undefined}
      <div class="border-t my-2" style="border-color: {$currentTheme.primary}33;"></div>
      
      <!-- Actual Depth Reached -->
      {#if lastDepthReached > 0}
        <div class="flex justify-between items-center">
          <span class="text-white/80 font-medium text-sm">Depth Reached:</span>
          <span class="font-bold text-lg" style="color: {$currentTheme.accent};">
            {lastDepthReached}
          </span>
        </div>
      {/if}
      
      <!-- Last Move Time -->
      <div class="flex justify-between items-center">
        <span class="text-white/80 font-medium text-sm">AI Last Move:</span>
        <span class="font-bold text-lg" style="color: {$currentTheme.secondary};">
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
  
  <!-- Divider -->
  <div class="border-t my-3" style="border-color: {$currentTheme.primary}33;"></div>
  
  <!-- Game Controls -->
  <div class="space-y-2">
    <h3 
      class="text-base font-bold text-center mb-2"
      style="color: {$currentTheme.primary};"
    >
      Controls
    </h3>
    
    <!-- Double-three visibility toggle -->
    <div class="flex justify-center">
      <Toggle 
        checked={showDoubleThree}
        onchange={onToggleDoubleThree}
        label="Show Double-Three" 
        id="double-three-toggle"
      />
    </div>
    
    <!-- AI vs AI Controls -->
    {#if isFullAI}
      {#if !isPlaying && !isPaused}
        <Button variant="primary" size="sm" onclick={onStartGame} disabled={!gameInstance || isGameOver} fullWidth>
          Start Match
        </Button>
      {/if}
      
      {#if isPlaying}
        <Button variant="secondary" size="sm" onclick={onPauseGame} fullWidth>
          Pause
        </Button>
      {/if}
      
      {#if isPaused && !isGameOver}
        <Button variant="primary" size="sm" onclick={onResumeGame} fullWidth>
          Resume
        </Button>
      {/if}
    {/if}
    
    <!-- Undo button for human games -->
    {#if hasHuman && !isFullAI}
      <Button variant="secondary" size="sm" onclick={onUndoMove} disabled={isPlaying} fullWidth>
        Undo Move
      </Button>
    {/if}
    
    <!-- New Game -->
    <Button variant="secondary" size="sm" onclick={onResetGame} disabled={isPlaying && !isPaused} fullWidth>
      New Game
    </Button>
    
    <!-- Back button -->
    {#if onBack}
      <Button variant="ghost" size="sm" onclick={onBack} fullWidth>
        {$_('game.menu.back')}
      </Button>
    {/if}
  </div>
  </div>
</div>
