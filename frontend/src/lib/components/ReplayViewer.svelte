<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import type { MatchData } from '$lib/stores/matchHistory';
  import { currentTheme } from '$lib/theme/themeStore';
  import GomokuBoard from './GomokuBoard.svelte';
  import Scoreboard from './Scoreboard.svelte';
  import Button from './Button.svelte';
  import Slider from './Slider.svelte';
  
  interface Props {
    match: MatchData;
    onBack?: () => void;
  }
  
  let { match, onBack }: Props = $props();
  
  let board = $state(Array(match.boardSize).fill(null).map(() => Array(match.boardSize).fill(null)));
  let gameInstance: any = $state(null);
  let currentMoveIndex = $state(0);
  let isPlaying = $state(false);
  let playbackSpeed = $state(1000); // milliseconds per move
  let playbackInterval: number | null = null;
  let isToggling = false; // Prevent rapid toggling
  
  // Stats from current replay state
  let player1Captures = $state(0);
  let player2Captures = $state(0);
  let lastMovePosition = $state<{row: number, col: number} | null>(null);
  
  const totalMoves = match.moveHistory.length;
  
  // Use simple getters instead of $derived to avoid reactive loops
  function getCurrentPlayer() {
    return currentMoveIndex % 2 === 0 ? 1 : 2;
  }
  
  function getTurnNumber() {
    return Math.ceil(currentMoveIndex / 2);
  }
  
  onMount(async () => {
    try {
      const wasmModule = await import('$lib/wasm/pkg/gomoku');
      await wasmModule.default();
      
      // Create fresh game instance
      gameInstance = new wasmModule.WasmGameState(match.boardSize, match.winCondition);
      
      updateBoardFromWasm();
    } catch (error) {
      console.error('Failed to initialize replay:', error);
    }
  });
  
  onDestroy(() => {
    stopPlayback();
  });
  
  function updateBoardFromWasm() {
    if (!gameInstance) return;
    
    const size = gameInstance.get_board_size();
    const newBoard = Array(size).fill(null).map(() => Array(size).fill(null));
    
    for (let row = 0; row < size; row++) {
      for (let col = 0; col < size; col++) {
        const player = gameInstance.get_stone_at(row, col);
        if (player !== undefined) {
          newBoard[row][col] = player === 0 ? 'black' : 'white';
        }
      }
    }
    
    board = newBoard;
    player1Captures = gameInstance.get_max_captures();
    player2Captures = gameInstance.get_min_captures();
  }
  
  function applyMove(moveIndex: number) {
    if (!gameInstance || moveIndex >= match.moveHistory.length || moveIndex < 0) return false;
    
    const move = match.moveHistory[moveIndex];
    try {
      gameInstance.make_move_coords(move.row, move.col);
      lastMovePosition = { row: move.row, col: move.col };
      updateBoardFromWasm();
      return true;
    } catch (error) {
      console.error('Error applying move:', error);
      return false;
    }
  }
  
  function undoMove() {
    if (!gameInstance || currentMoveIndex === 0) return false;
    
    try {
      gameInstance.undo_last_move();
      updateBoardFromWasm();
      return true;
    } catch (error) {
      console.error('Error undoing move:', error);
      return false;
    }
  }
  
  function nextMove() {
    if (currentMoveIndex >= totalMoves) {
      stopPlayback();
      return;
    }
    
    if (applyMove(currentMoveIndex)) {
      currentMoveIndex++;
    }
  }
  
  function previousMove() {
    if (currentMoveIndex === 0) return;
    
    if (undoMove()) {
      currentMoveIndex--;
      // Update last move position to the previous move
      if (currentMoveIndex > 0) {
        const prevMove = match.moveHistory[currentMoveIndex - 1];
        lastMovePosition = { row: prevMove.row, col: prevMove.col };
      } else {
        lastMovePosition = null;
      }
    }
  }
  
  function goToStart() {
    stopPlayback();
    
    // Reset game instance
    if (gameInstance) {
      gameInstance.reset();
      currentMoveIndex = 0;
      lastMovePosition = null;
      updateBoardFromWasm();
    }
  }
  
  function goToEnd() {
    stopPlayback();
    
    // Apply all moves
    goToStart();
    while (currentMoveIndex < totalMoves) {
      applyMove(currentMoveIndex);
      currentMoveIndex++;
    }
    
    // Set last move position
    if (currentMoveIndex > 0) {
      const lastMove = match.moveHistory[currentMoveIndex - 1];
      lastMovePosition = { row: lastMove.row, col: lastMove.col };
    }
  }
  
  function goToMove(index: number) {
    stopPlayback();
    
    if (index < 0 || index > totalMoves) return;
    
    // Reset and replay to target move
    goToStart();
    while (currentMoveIndex < index) {
      applyMove(currentMoveIndex);
      currentMoveIndex++;
    }
    
    // Update last move position
    if (currentMoveIndex > 0) {
      const move = match.moveHistory[currentMoveIndex - 1];
      lastMovePosition = { row: move.row, col: move.col };
    } else {
      lastMovePosition = null;
    }
  }
  
  function togglePlayback() {
    if (isToggling) return; // Prevent re-entry
    isToggling = true;
    
    if (isPlaying) {
      stopPlayback();
    } else {
      startPlayback();
    }
    
    // Reset flag after a small delay
    setTimeout(() => {
      isToggling = false;
    }, 100);
  }
  
  function startPlayback() {
    if (currentMoveIndex >= totalMoves) {
      // Don't auto-restart, just reset index
      currentMoveIndex = 0;
      if (gameInstance) {
        gameInstance.reset();
        lastMovePosition = null;
        updateBoardFromWasm();
      }
    }
    
    isPlaying = true;
    playbackInterval = window.setInterval(() => {
      if (currentMoveIndex >= totalMoves) {
        stopPlayback();
      } else {
        nextMove();
      }
    }, playbackSpeed);
  }
  
  function stopPlayback() {
    isPlaying = false;
    if (playbackInterval !== null) {
      clearInterval(playbackInterval);
      playbackInterval = null;
    }
  }
</script>

<div class="h-full w-full flex flex-col overflow-hidden">
  <!-- Header -->
  <div class="flex-shrink-0 px-4 py-3 backdrop-blur-md border-b"
       style="background-color: rgba(17, 24, 39, 0.7); border-color: rgba(139, 92, 246, 0.3);">
    <div class="max-w-7xl mx-auto flex items-center justify-between">
      <div>
        <h2 class="text-xl font-bold text-white">Match Replay</h2>
        <p class="text-sm text-white opacity-70">
          {match.player1Name} vs {match.player2Name}
        </p>
      </div>
      <Button variant="secondary" size="sm" onclick={() => onBack?.()}>
        <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
        Back
      </Button>
    </div>
  </div>

  <!-- Main Content -->
  <div class="flex-1 min-h-0 overflow-hidden">
    <div class="h-full max-w-7xl mx-auto px-4 py-4">
      <!-- Desktop Layout -->
      <div class="hidden md:flex gap-6 h-full">
        <!-- Board -->
        <div class="flex-1 flex items-center justify-center min-w-0">
          <div class="w-full h-full flex items-center justify-center">
            <div style="width: 100%; height: 100%; max-width: min(100%, 100vh - 300px); max-height: min(100%, 100vh - 300px); aspect-ratio: 1/1;">
              <GomokuBoard 
                {board} 
                currentPlayer={getCurrentPlayer() === 1 ? 'black' : 'white'}
                {lastMovePosition}
                canHumanPlay={false}
              />
            </div>
          </div>
        </div>

        <!-- Stats Panel -->
        <div class="flex-shrink-0 w-80 space-y-4 overflow-y-auto">
          <!-- Scoreboard -->
          <Scoreboard
            player1Name={match.player1Name}
            player2Name={match.player2Name}
            player1Captures={player1Captures}
            player2Captures={player2Captures}
            totalMoves={getTurnNumber()}
          />

          <!-- Match Info -->
          <div class="rounded-lg p-4 backdrop-blur-md border"
               style="background: {$currentTheme.surface}20; border-color: {$currentTheme.primary}40;">
            <h3 class="text-sm font-semibold text-white mb-3">Match Info</h3>
            <div class="space-y-2 text-sm text-white opacity-80">
              <div class="flex justify-between">
                <span>Board Size:</span>
                <span>{match.boardSize}x{match.boardSize}</span>
              </div>
              <div class="flex justify-between">
                <span>Win Condition:</span>
                <span>{match.winCondition} in a row</span>
              </div>
              <div class="flex justify-between">
                <span>Total Moves:</span>
                <span>{totalMoves}</span>
              </div>
              {#if match.aiDepth}
                <div class="flex justify-between">
                  <span>AI Depth:</span>
                  <span>{match.aiDepth}</span>
                </div>
              {/if}
              <div class="flex justify-between">
                <span>Status:</span>
                <span class="font-semibold" style="color: {match.winner ? $currentTheme.success : $currentTheme.warning};">
                  {#if match.winner}
                    {match.winner === 1 ? match.player1Name : match.player2Name} Won
                  {:else}
                    {match.status === 'ongoing' ? 'Ongoing' : 'Draw'}
                  {/if}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Mobile Layout -->
      <div class="md:hidden space-y-4">
        <!-- Board -->
        <div class="w-full" style="max-width: min(95vw, calc(100vh - 20rem)); aspect-ratio: 1/1; margin: 0 auto;">
          <GomokuBoard 
            {board} 
            currentPlayer={getCurrentPlayer() === 1 ? 'black' : 'white'}
            {lastMovePosition}
            canHumanPlay={false}
          />
        </div>

        <!-- Scoreboard -->
        <Scoreboard
          player1Name={match.player1Name}
          player2Name={match.player2Name}
          player1Captures={player1Captures}
          player2Captures={player2Captures}
          totalMoves={getTurnNumber()}
        />
      </div>
    </div>
  </div>

  <!-- Media Controls (Fixed at bottom) -->
  <div class="flex-shrink-0 px-4 py-4 backdrop-blur-md border-t"
       style="background-color: rgba(17, 24, 39, 0.9); border-color: rgba(139, 92, 246, 0.3);">
    <div class="max-w-3xl mx-auto space-y-3">
      <!-- Progress Bar -->
      <div class="flex items-center gap-3">
        <span class="text-xs text-white opacity-70 min-w-[3rem]">
          {currentMoveIndex} / {totalMoves}
        </span>
        <input
          type="range"
          min="0"
          max={totalMoves}
          value={currentMoveIndex}
          oninput={(e) => goToMove(parseInt(e.currentTarget.value))}
          class="flex-1 h-2 rounded-lg appearance-none cursor-pointer"
          style="
            background: linear-gradient(to right, 
              {$currentTheme.primary} 0%, 
              {$currentTheme.primary} {(currentMoveIndex / totalMoves) * 100}%, 
              {$currentTheme.surface}40 {(currentMoveIndex / totalMoves) * 100}%, 
              {$currentTheme.surface}40 100%);
          "
        />
        <span class="text-xs text-white opacity-70">Turn {getTurnNumber()}</span>
      </div>

      <!-- Playback Controls -->
      <div class="flex items-center justify-center gap-2">
        <!-- Go to Start -->
        <button
          class="p-2 rounded-lg transition-all duration-200 hover:scale-110"
          style="background: {$currentTheme.surface}20; color: white;"
          onclick={goToStart}
          disabled={currentMoveIndex === 0}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
          </svg>
        </button>

        <!-- Previous Move -->
        <button
          class="p-2 rounded-lg transition-all duration-200 hover:scale-110"
          style="background: {$currentTheme.surface}20; color: white;"
          onclick={previousMove}
          disabled={currentMoveIndex === 0}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
          </svg>
        </button>

        <!-- Play/Pause -->
        <button
          class="p-3 rounded-full transition-all duration-200 hover:scale-110"
          style="background: {$currentTheme.primary}; color: white; box-shadow: {$currentTheme.glowPrimary};"
          onclick={togglePlayback}
        >
          {#if isPlaying}
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z" />
            </svg>
          {:else}
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
              <path d="M8 5v14l11-7z" />
            </svg>
          {/if}
        </button>

        <!-- Next Move -->
        <button
          class="p-2 rounded-lg transition-all duration-200 hover:scale-110"
          style="background: {$currentTheme.surface}20; color: white;"
          onclick={nextMove}
          disabled={currentMoveIndex >= totalMoves}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
          </svg>
        </button>

        <!-- Go to End -->
        <button
          class="p-2 rounded-lg transition-all duration-200 hover:scale-110"
          style="background: {$currentTheme.surface}20; color: white;"
          onclick={goToEnd}
          disabled={currentMoveIndex >= totalMoves}
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
          </svg>
        </button>
      </div>

      <!-- Speed Control -->
      <div class="flex items-center gap-3 justify-center">
        <span class="text-xs text-white opacity-70 min-w-[4rem]">Speed:</span>
        <input
          type="range"
          min="100"
          max="3000"
          step="100"
          value={playbackSpeed}
          oninput={(e) => playbackSpeed = parseInt(e.currentTarget.value)}
          class="w-48 h-2 rounded-lg appearance-none cursor-pointer"
          style="
            background: linear-gradient(to right, 
              {$currentTheme.accent} 0%, 
              {$currentTheme.accent} {((playbackSpeed - 100) / 2900) * 100}%, 
              {$currentTheme.surface}40 {((playbackSpeed - 100) / 2900) * 100}%, 
              {$currentTheme.surface}40 100%);
          "
        />
        <span class="text-xs text-white opacity-70 min-w-[3rem]">
          {(playbackSpeed / 1000).toFixed(1)}s
        </span>
      </div>
    </div>
  </div>
</div>

<style>
  input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    cursor: pointer;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  input[type="range"]::-moz-range-thumb {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    cursor: pointer;
    border: none;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
  }

  button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
</style>
