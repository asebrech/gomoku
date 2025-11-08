<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { untrack } from 'svelte';
  import GomokuBoard from './GomokuBoard.svelte';
  import GameStats from './GameStats.svelte';
  import Scoreboard from './Scoreboard.svelte';
  import Button from './Button.svelte';
  import Toggle from './Toggle.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    player1Type: 'human' | 'ai';
    player2Type: 'human' | 'ai';
    player1Name?: string;
    player2Name?: string;
    aiDepth?: number;
    autoStart?: boolean;
    moveDelay?: number;
    showSpeedControl?: boolean;
    onBack?: () => void;
  }
  
  let {
    player1Type,
    player2Type,
    player1Name = player1Type === 'ai' ? 'AI 1' : 'Player 1',
    player2Name = player2Type === 'ai' ? 'AI 2' : 'Player 2',
    aiDepth = $gameSettings.aiDepth,
    autoStart = false,
    moveDelay = 500,
    showSpeedControl = false,
    onBack
  }: Props = $props();
  
  let board = $state(Array(19).fill(null).map(() => Array(19).fill(null)));
  let gameInstance: any = $state(null);
  let gameStatus = $state('Initializing...');
  let isPlaying = $state(false);
  let isPaused = $state(false);
  let isGameOver = $state(false);
  let currentPlayer = $state(1); // 1 or 2
  let currentPlayerName = $state(player1Name);
  let winnerPlayer = $state<number | null>(null); // Track winner (1 or 2)
  let aiMoveDelay = $state(moveDelay);
  let waitingForHumanMove = $state(false);
  let shouldStop = $state(false);
  let isAIThinking = $state(false); // New: Track when AI is computing
  let lastDepthReached = $state(0); // Track the depth the AI reached
  let lastNodesSearched = $state(0); // Track nodes searched
  let lastAIScore = $state(0); // Track AI evaluation score
  let needsAIContinue = $state(false); // Track if we need AI to continue after undo
  
  // Double-three visualization
  let showDoubleThree = $state($gameSettings.showDoubleThree);
  let doubleThreePositions = $state<Array<{row: number, col: number}>>([]);
  
  // AI hint feature
  let showAIHint = $state($gameSettings.showAIHint);
  let aiHintPosition = $state<{row: number, col: number} | null>(null);
  let isCalculatingHint = $state(false);
  
  // Stats tracking
  let totalMoves = $state(0);
  let totalThinkingTime = $state(0);
  let lastMoveTime = $state(0);
  let player1Captures = $state(0);
  let player2Captures = $state(0);
  let aiMoveCount = $state(0); // Track number of AI moves separately
  
  // Last move tracking for visual effect
  let lastMovePosition = $state<{row: number, col: number} | null>(null);
  
  const isFullAI = player1Type === 'ai' && player2Type === 'ai';
  const hasHuman = player1Type === 'human' || player2Type === 'human';
  
  // Generate game mode display name
  const gameModeDisplay = $derived(() => {
    if (player1Type === 'ai' && player2Type === 'ai') return 'AI vs AI';
    if (player1Type === 'human' && player2Type === 'human') return 'Player vs Player';
    return 'Player vs AI';
  });
  
  onMount(async () => {
    try {
      // Import WASM module (already initialized in root layout)
      const wasmModule = await import('$lib/wasm/pkg/gomoku');
      
      // Create game instance with settings from store
      gameInstance = new wasmModule.WasmGameState($gameSettings.boardSize, $gameSettings.winCondition);
      console.log(`Game initialized! Board: ${$gameSettings.boardSize}x${$gameSettings.boardSize}, Win: ${$gameSettings.winCondition}`);
      
      // Initialize board with correct size
      board = Array($gameSettings.boardSize).fill(null).map(() => Array($gameSettings.boardSize).fill(null));
      updateBoardFromWasm();
      isGameOver = false;
      gameStatus = 'Ready to start';
      
      // Auto-start only for AI vs AI mode when autoStart is true
      if (autoStart && isFullAI) {
        startGame();
      }
    } catch (error) {
      console.error('Failed to initialize game:', error);
      gameStatus = 'Error loading game engine';
    }
  });
  
  onDestroy(() => {
    // Stop the game loop when component is destroyed
    shouldStop = true;
    isPlaying = false;
    isPaused = true;
    waitingForHumanMove = false;
    console.log('Game component destroyed, stopping game loop');
  });
  
  function updateBoardFromWasm() {
    if (!gameInstance) return;
    
    try {
      const size = gameInstance.get_board_size();
      
      // Convert the board state to our 2D array format
      const newBoard = Array(size).fill(null).map(() => Array(size).fill(null));
      
      // Iterate through all positions and get the player at each
      for (let row = 0; row < size; row++) {
        for (let col = 0; col < size; col++) {
          const player = gameInstance.get_stone_at(row, col);
          if (player !== undefined) {
            // Player.Max (0) = black, Player.Min (1) = white
            newBoard[row][col] = player === 0 ? 'black' : 'white';
          }
        }
      }
      
      board = newBoard;
      
      // Update current player (0 = Max/Player1, 1 = Min/Player2)
      const wasmCurrentPlayer = gameInstance.get_current_player();
      currentPlayer = wasmCurrentPlayer === 0 ? 1 : 2;
      currentPlayerName = currentPlayer === 1 ? player1Name : player2Name;
      
      // Update captures
      player1Captures = gameInstance.get_max_captures();
      player2Captures = gameInstance.get_min_captures();
      
      // Update double-three positions if the toggle is on and game is not over
      if (showDoubleThree && !isGameOver) {
        updateDoubleThreePositions();
      }
    } catch (error) {
      console.error('Error updating board:', error);
    }
  }
  
  function updateDoubleThreePositions() {
    // Guard against calling when game instance doesn't exist or game is over
    if (!gameInstance || isGameOver) {
      doubleThreePositions = [];
      return;
    }
    
    // Only update if showDoubleThree is enabled
    if (!showDoubleThree) {
      doubleThreePositions = [];
      return;
    }
    
    try {
      const positions = gameInstance.get_double_three_positions();
      if (!positions) {
        doubleThreePositions = [];
        return;
      }
      
      doubleThreePositions = [];
      for (let i = 0; i < positions.length; i++) {
        const pos = positions[i];
        if (pos && typeof pos.row === 'number' && typeof pos.col === 'number') {
          doubleThreePositions.push({ row: pos.row, col: pos.col });
        }
      }
    } catch (error) {
      console.error('Error getting double-three positions:', error);
      doubleThreePositions = [];
    }
  }
  
  async function calculateAIHint() {
    // Only calculate hints for human players during their turn
    if (!gameInstance || isGameOver || !showAIHint || !waitingForHumanMove) {
      aiHintPosition = null;
      return;
    }
    
    try {
      isCalculatingHint = true;
      
      // Use the same AI params as configured
      const hintResult = gameInstance.get_ai_move(aiDepth, $gameSettings.aiMaxThinkingTime);
      
      if (hintResult && typeof hintResult.row === 'number' && typeof hintResult.col === 'number') {
        aiHintPosition = { row: hintResult.row, col: hintResult.col };
      } else {
        aiHintPosition = null;
      }
    } catch (error) {
      console.error('Error calculating AI hint:', error);
      aiHintPosition = null;
    } finally {
      isCalculatingHint = false;
    }
  }
  
  async function handleCellClick(row: number, col: number) {
    if (!gameInstance || isGameOver || needsAIContinue) {
      return;
    }
    
    // If game hasn't started yet and this is a game with humans, start it
    if (!isPlaying && hasHuman) {
      startGame();
      // Wait a moment for the game to start
      await new Promise(resolve => setTimeout(resolve, 50));
    }
    
    // Now check if we're waiting for human move
    if (!waitingForHumanMove) {
      return;
    }
    
    // Check if position is valid and empty
    if (!board[row] || board[row][col] !== null) {
      return;
    }
    
    // Make the human move
    await makeMove(row, col);
  }
  
  async function makeMove(row: number, col: number) {
    if (!gameInstance || isGameOver) return false;
    
    try {
      // Check if move is legal first
      if (!gameInstance.is_move_legal_coords(row, col)) {
        return false;
      }
      
      // Make the move
      gameInstance.make_move_coords(row, col);
      totalMoves++;
      
      // Track the last move position for visual effect
      lastMovePosition = { row, col };
      
      updateBoardFromWasm();
      
      // Clear AI hint after move
      aiHintPosition = null;
      
      // Check for game over
      if (gameInstance.is_terminal()) {
        isGameOver = true;
        const winner = gameInstance.get_winner();
        if (winner !== undefined) {
          winnerPlayer = winner === 0 ? 1 : 2; // Convert to player number
          gameStatus = 'wins!';
        } else {
          winnerPlayer = null;
          gameStatus = 'Game Over - Draw';
        }
        isPlaying = false;
      }
      
      waitingForHumanMove = false;
      return true;
    } catch (error) {
      console.error('Move error:', error);
      return false;
    }
  }
  
  async function getAIMove(): Promise<{row: number, col: number} | null> {
    if (!gameInstance) return null;
    
    try {
      isAIThinking = true; // Set thinking state
      const startTime = performance.now();
      
      // Yield to the browser before heavy computation to allow UI updates
      await new Promise(resolve => setTimeout(resolve, 0));
      
      // Pass time limit as a regular number (f64 in Rust)
      const aiMoveResult = gameInstance.get_ai_move(aiDepth, $gameSettings.aiMaxThinkingTime);
      const endTime = performance.now();
      
      // Yield again after computation to let the browser update
      await new Promise(resolve => setTimeout(resolve, 0));
      
      if (!aiMoveResult) {
        console.error('AI failed to find a move');
        return null;
      }
      
      // Track AI thinking time and stats
      lastMoveTime = Math.round(endTime - startTime);
      totalThinkingTime += lastMoveTime;
      aiMoveCount++;
      
      // Track AI search stats
      lastDepthReached = aiMoveResult.depth_reached;
      lastNodesSearched = Math.round(aiMoveResult.nodes_searched);
      lastAIScore = aiMoveResult.score;
      
      console.log(`AI Move: depth=${lastDepthReached}, nodes=${lastNodesSearched}, score=${lastAIScore}, time=${lastMoveTime}ms`);
      
      return { row: aiMoveResult.row, col: aiMoveResult.col };
    } catch (error) {
      console.error('AI move error:', error);
      return null;
    } finally {
      isAIThinking = false; // Clear thinking state
    }
  }
  
  async function playGameLoop() {
    if (!gameInstance || !isPlaying) return;
    
    while (!isGameOver && !isPaused && !shouldStop && isPlaying) {
      // Check if we should stop at the beginning of each iteration
      if (shouldStop) {
        break;
      }
      
      const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
      
      if (currentPlayerType === 'human') {
        // Wait for human move
        waitingForHumanMove = true;
        gameStatus = 'Your turn';
        
        // Wait until human makes a move (handled by handleCellClick)
        while (waitingForHumanMove && !isPaused && !isGameOver && !shouldStop) {
          await new Promise(resolve => setTimeout(resolve, 100));
          if (shouldStop) break;
        }
        
        if (isPaused || isGameOver || shouldStop) break;
        
      } else {
        // AI move
        waitingForHumanMove = false;
        
        // If we need to wait for continue button after undo, wait here
        while (needsAIContinue && !shouldStop && !isPaused) {
          await new Promise(resolve => setTimeout(resolve, 100));
          if (shouldStop || isPaused) break;
        }
        
        if (isPaused || shouldStop) break;
        
        gameStatus = 'AI is thinking...';
        
        // Wait for the delay to make it visible (but check shouldStop during the wait)
        const startTime = Date.now();
        while (Date.now() - startTime < aiMoveDelay && !shouldStop && !isPaused) {
          await new Promise(resolve => setTimeout(resolve, 50));
        }
        
        if (isPaused || shouldStop) break;
        
        const aiMove = await getAIMove();
        
        if (!aiMove) {
          gameStatus = 'AI error - no move found';
          isPlaying = false;
          break;
        }
        
        const success = await makeMove(aiMove.row, aiMove.col);
        
        if (!success) {
          gameStatus = 'AI move failed';
          isPlaying = false;
          break;
        }
        
        if (isGameOver || shouldStop) break;
      }
    }
    
    if (isPaused) {
      gameStatus = 'Game paused';
    }
    
    isPlaying = false;
    waitingForHumanMove = false;
  }
  
  function startGame() {
    if (!gameInstance || isPlaying) return;
    shouldStop = false;
    isPlaying = true;
    isPaused = false;
    isGameOver = false;
    playGameLoop();
  }
  
  function pauseGame() {
    isPaused = true;
    isPlaying = false;
    waitingForHumanMove = false;
  }
  
  function resumeGame() {
    if (!gameInstance || isGameOver) return;
    shouldStop = false;
    isPlaying = true;
    isPaused = false;
    playGameLoop();
  }
  
  async function resetGame() {
    if (gameInstance) {
      // First stop any running game loop
      shouldStop = true;
      isPlaying = false;
      
      // Wait a moment for any running loop to stop
      await new Promise(resolve => setTimeout(resolve, 100));
      
      // Now reset the game state
      shouldStop = false;
      gameInstance.reset();
      updateBoardFromWasm();
      isGameOver = false;
      isPaused = false;
      waitingForHumanMove = false;
      needsAIContinue = false;
      winnerPlayer = null;
      gameStatus = 'Ready to start';
      
      // Reset stats
      totalMoves = 0;
      totalThinkingTime = 0;
      lastMoveTime = 0;
      player1Captures = 0;
      player2Captures = 0;
      aiMoveCount = 0;
      lastDepthReached = 0;
      lastNodesSearched = 0;
      lastAIScore = 0;
      lastMovePosition = null;
      aiHintPosition = null;
      
      // Auto-restart for AI vs AI only
      if (autoStart && isFullAI) {
        setTimeout(() => startGame(), 500);
      }
    }
  }
  
  function undoMove() {
    if (gameInstance && totalMoves > 0) {
      gameInstance.undo_last_move();
      totalMoves--;
      updateBoardFromWasm();
      gameStatus = 'Move undone';
      
      // Clear AI hint and last move position after undo
      aiHintPosition = null;
      lastMovePosition = null;
      
      // Check whose turn it is now after undo
      const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
      
      // If it's now an AI's turn, we need a continue button
      if (currentPlayerType === 'ai' && hasHuman) {
        needsAIContinue = true;
        waitingForHumanMove = false;
        gameStatus = 'Click Continue to let AI play';
      } else {
        // It's a human's turn
        needsAIContinue = false;
        waitingForHumanMove = true;
        gameStatus = 'Your turn';
      }
    }
  }
  
  function continueAfterUndo() {
    if (needsAIContinue) {
      needsAIContinue = false;
      // Continue the game loop - it will handle the AI turn
      // Don't need to call playGameLoop again, just let the existing loop continue
    }
  }
  
  $effect(() => {
    // When human makes a move, continue the game loop
    if (waitingForHumanMove) {
      const checkMove = setInterval(() => {
        if (!waitingForHumanMove) {
          clearInterval(checkMove);
        }
      }, 100);
      return () => clearInterval(checkMove);
    }
  });
  
  $effect(() => {
    // Update double-three positions when toggle changes or board updates
    // Only run if game instance exists and game is not over
    // Use untrack to prevent infinite loops when updating doubleThreePositions
    if (gameInstance && !isGameOver && showDoubleThree) {
      untrack(() => {
        updateDoubleThreePositions();
      });
    } else if (!showDoubleThree) {
      // Clear positions when toggle is off
      untrack(() => {
        doubleThreePositions = [];
      });
    }
  });
  
  $effect(() => {
    // Calculate AI hint when toggle is on and it's human's turn
    if (gameInstance && !isGameOver && showAIHint && waitingForHumanMove && hasHuman) {
      calculateAIHint();
    } else if (!showAIHint) {
      // Clear hint when toggle is off
      untrack(() => {
        aiHintPosition = null;
      });
    }
  });
</script>

<div class="w-full max-h-[calc(100vh-5rem)] flex flex-col items-center">
  <!-- Desktop Layout: Board and Stats Side by Side -->
  <div class="hidden md:flex justify-center items-stretch gap-6 flex-1 min-h-0 px-4 pb-4 pt-2 w-full max-w-screen-xl">
    <!-- Board Column with Scoreboard -->
    <div class="flex flex-col items-center gap-3 flex-1 min-w-0 max-w-[600px]">
      <!-- Board Container with aspect ratio constraint -->
      <div class="w-full flex-shrink-0 flex items-center justify-center" style="aspect-ratio: 1/1; max-width: min(100%, calc(100vh - 16rem)); max-height: calc(100vh - 16rem);">
        <div class="w-full h-full">
          <GomokuBoard 
            {board} 
            onCellClick={handleCellClick}
            currentPlayer={currentPlayer === 1 ? 'black' : 'white'}
            {doubleThreePositions}
            {aiHintPosition}
            {lastMovePosition}
          />
        </div>
      </div>
      
      <!-- Desktop Scoreboard (under board) -->
      <div class="w-full flex-shrink-0">
        <Scoreboard
          player1Name={player1Name}
          player2Name={player2Name}
          player1Captures={player1Captures}
          player2Captures={player2Captures}
          totalMoves={totalMoves}
        />
      </div>
    </div>
    
    <!-- Stats Panel -->
    <div class="flex-shrink-0">
      <GameStats
        gameMode={gameModeDisplay()}
        boardSize={$gameSettings.boardSize}
        aiDepth={player1Type === 'ai' || player2Type === 'ai' ? aiDepth : undefined}
        currentPlayer={currentPlayerName}
        winnerPlayer={winnerPlayer}
        player1Name={player1Name}
        player2Name={player2Name}
        {totalMoves}
        {player1Captures}
        {player2Captures}
        {totalThinkingTime}
        {lastMoveTime}
        {aiMoveCount}
        {lastDepthReached}
        {lastNodesSearched}
        {lastAIScore}
        {showDoubleThree}
        onToggleDoubleThree={(value) => {
          showDoubleThree = value;
          gameSettings.update(s => ({ ...s, showDoubleThree: value }));
        }}
        {showAIHint}
        onToggleAIHint={(value) => {
          showAIHint = value;
          gameSettings.update(s => ({ ...s, showAIHint: value }));
        }}
        {isCalculatingHint}
        {isPlaying}
        {isPaused}
        {isGameOver}
        {isFullAI}
        {hasHuman}
        {gameInstance}
        {needsAIContinue}
        onStartGame={startGame}
        onPauseGame={pauseGame}
        onResumeGame={resumeGame}
        onUndoMove={undoMove}
        onContinueAI={continueAfterUndo}
        onResetGame={resetGame}
        {onBack}
      />
    </div>
  </div>

  <!-- Mobile Layout: Board on Top, Compact Stats Below -->
  <div class="md:hidden flex flex-col items-center w-full h-[calc(100vh-5rem)] px-2 pb-4 pt-2 gap-3 overflow-y-auto">
    <!-- Board -->
    <div class="w-full flex-shrink-0" style="max-width: min(95vw, calc(100vh - 20rem)); aspect-ratio: 1/1;">
      <GomokuBoard 
        {board} 
        onCellClick={handleCellClick}
        currentPlayer={currentPlayer === 1 ? 'black' : 'white'}
        {doubleThreePositions}
        {aiHintPosition}
        {lastMovePosition}
      />
    </div>
    
    <!-- Mobile Compact Game Info -->
    <div class="flex-shrink-0" style="width: 100%; max-width: min(95vw, calc(100vh - 20rem));">
      <!-- Game Over Message -->
      {#if isGameOver && winnerPlayer !== null}
        {@const winnerName = winnerPlayer === 1 ? player1Name : player2Name}
        {@const cleanWinnerName = winnerName?.replace(/\s*\((Black|White)\)\s*/i, '').trim() || winnerName}
        {@const winnerColor = winnerPlayer === 1 ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2}
        <div 
          class="rounded-lg px-4 py-3 mb-3 text-center animate-fade-in"
          style="background: {$currentTheme.background}; border: 3px solid {winnerColor}; box-shadow: 0 0 20px {winnerColor}60;"
        >
          <div class="flex items-center justify-center gap-2 mb-1">
            <svg width="24" height="24" class="inline-block">
              <defs>
                <radialGradient id="winnerMobileGradient">
                  <stop offset="30%" stop-color={winnerColor} stop-opacity="0.9" />
                  <stop offset="100%" stop-color={winnerColor} stop-opacity="1" />
                </radialGradient>
                <filter id="winnerMobileShadow">
                  <feDropShadow dx="1" dy="2" stdDeviation="2" flood-opacity="0.7"/>
                </filter>
              </defs>
              <circle
                cx="12"
                cy="12"
                r="10"
                fill="url(#winnerMobileGradient)"
                filter="url(#winnerMobileShadow)"
              />
            </svg>
            <p 
              class="text-xl font-bold"
              style="color: {winnerColor};"
            >
              {cleanWinnerName} Wins!
            </p>
            <svg width="24" height="24" class="inline-block">
              <circle
                cx="12"
                cy="12"
                r="10"
                fill="url(#winnerMobileGradient)"
                filter="url(#winnerMobileShadow)"
              />
            </svg>
          </div>
        </div>
      {/if}

      <!-- Mobile Scoreboard -->
      <div class="mb-3">
        <Scoreboard
          player1Name={player1Name}
          player2Name={player2Name}
          player1Captures={player1Captures}
          player2Captures={player2Captures}
          totalMoves={totalMoves}
        />
      </div>

      <!-- Mobile Controls -->
      <div class="flex flex-wrap gap-2 justify-center">
        <!-- Game Controls - Start button only for AI vs AI -->
        {#if !isPlaying && !isPaused && isFullAI}
          <Button variant="primary" size="sm" onclick={startGame} disabled={!gameInstance || isGameOver}>
            Start Match
          </Button>
        {/if}
        
        <!-- AI vs AI Controls -->
        {#if isFullAI}
          {#if isPlaying}
            <Button variant="primary" size="sm" onclick={pauseGame}>
              Pause
            </Button>
          {/if}
          
          {#if isPaused && !isGameOver}
            <Button variant="primary" size="sm" onclick={resumeGame}>
              Resume
            </Button>
          {/if}
        {/if}
        
        <!-- Undo button -->
        <Button variant="primary" size="sm" onclick={undoMove} disabled={totalMoves === 0 || isGameOver}>
          Undo
        </Button>
        
        <!-- Continue button after undo (when it's AI's turn) -->
        {#if needsAIContinue && hasHuman && !isFullAI}
          <Button variant="primary" size="sm" onclick={continueAfterUndo}>
            Continue
          </Button>
        {/if}
        
        <!-- New Game -->
        <Button variant="primary" size="sm" onclick={async () => await resetGame()}>
          New Game
        </Button>
        
        <!-- Back button -->
        {#if onBack}
          <Button variant="primary" size="sm" onclick={onBack}>
            Back
          </Button>
        {/if}
      </div>
    </div>
  </div>
</div>
