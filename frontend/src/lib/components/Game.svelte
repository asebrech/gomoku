<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import GomokuBoard from './GomokuBoard.svelte';
  import GameStats from './GameStats.svelte';
  import Button from './Button.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  
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
    player1Name = player1Type === 'ai' ? 'Black AI' : 'Player 1 (Black)',
    player2Name = player2Type === 'ai' ? 'White AI' : 'Player 2 (White)',
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
  let aiMoveDelay = $state(moveDelay);
  let waitingForHumanMove = $state(false);
  let shouldStop = $state(false);
  let isAIThinking = $state(false); // New: Track when AI is computing
  let lastDepthReached = $state(0); // Track the depth the AI reached
  let lastNodesSearched = $state(0); // Track nodes searched
  let lastAIScore = $state(0); // Track AI evaluation score
  
  // Stats tracking
  let totalMoves = $state(0);
  let totalThinkingTime = $state(0);
  let lastMoveTime = $state(0);
  let player1Captures = $state(0);
  let player2Captures = $state(0);
  let aiMoveCount = $state(0); // Track number of AI moves separately
  
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
      
      // Auto-start for AI vs AI
      if (autoStart && isFullAI) {
        startGame();
      } else if (hasHuman) {
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
    } catch (error) {
      console.error('Error updating board:', error);
    }
  }
  
  async function handleCellClick(row: number, col: number) {
    if (!gameInstance || isGameOver || !waitingForHumanMove) {
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
      updateBoardFromWasm();
      
      // Check for game over
      if (gameInstance.is_terminal()) {
        isGameOver = true;
        const winner = gameInstance.get_winner();
        if (winner !== undefined) {
          const winnerName = winner === 0 ? player1Name : player2Name;
          gameStatus = `${winnerName} wins!`;
        } else {
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
        gameStatus = `${currentPlayerName}'s turn`;
        
        // Wait until human makes a move (handled by handleCellClick)
        while (waitingForHumanMove && !isPaused && !isGameOver && !shouldStop) {
          await new Promise(resolve => setTimeout(resolve, 100));
          if (shouldStop) break;
        }
        
        if (isPaused || isGameOver || shouldStop) break;
        
      } else {
        // AI move
        waitingForHumanMove = false;
        gameStatus = `${currentPlayerName} is thinking...`;
        
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
  
  function resetGame() {
    if (gameInstance) {
      shouldStop = false;
      gameInstance.reset();
      updateBoardFromWasm();
      isGameOver = false;
      isPlaying = false;
      isPaused = false;
      waitingForHumanMove = false;
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
      
      // Auto-restart for AI vs AI
      if (autoStart && isFullAI) {
        setTimeout(() => startGame(), 500);
      } else if (hasHuman) {
        startGame();
      }
    }
  }
  
  function undoMove() {
    if (gameInstance) {
      gameInstance.undo_last_move();
      updateBoardFromWasm();
      gameStatus = `${currentPlayerName}'s turn`;
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
</script>

<div class="w-full">
  <div class="text-center mb-6">
    <p class="text-2xl font-bold mb-2" style="color: {isPlaying && !waitingForHumanMove ? '#FF00FF' : '#00FFFF'};">
      {gameStatus}
      {#if isAIThinking}
        <span class="inline-block ml-2 animate-pulse">🤔</span>
      {/if}
    </p>
    <p class="text-lg text-white/80">
      Current turn: <span class="font-bold" style="color: #00FFFF;">{currentPlayerName}</span>
      {#if isAIThinking}
        <span class="ml-2 text-sm text-cyan-400 animate-pulse">Computing...</span>
      {/if}
    </p>
  </div>
  
  <!-- Board and Stats Container -->
  <div class="flex justify-center items-start gap-8 flex-wrap lg:flex-nowrap">
    <div class="flex-shrink-0">
      <GomokuBoard 
        {board} 
        onCellClick={handleCellClick}
        currentPlayer={currentPlayer === 1 ? 'black' : 'white'}
      />
    </div>
    
    <div class="flex-shrink-0 self-start">
      <GameStats
        gameMode={gameModeDisplay()}
        boardSize={$gameSettings.boardSize}
        aiDepth={player1Type === 'ai' || player2Type === 'ai' ? aiDepth : undefined}
        currentPlayer={currentPlayerName}
        {totalMoves}
        {player1Captures}
        {player2Captures}
        {totalThinkingTime}
        {lastMoveTime}
        {aiMoveCount}
        {lastDepthReached}
        {lastNodesSearched}
        {lastAIScore}
      />
    </div>
  </div>
  
  <div class="flex justify-center items-center gap-4 mt-8 flex-wrap">
    {#if showSpeedControl}
      <div class="flex items-center gap-2">
        <label for="speed" class="text-white/90 text-sm">Speed:</label>
        <select 
          id="speed"
          bind:value={aiMoveDelay}
          disabled={isPlaying}
          class="px-3 py-1 bg-white/10 backdrop-blur-md border border-white/20 rounded text-white/90 text-sm disabled:opacity-50"
        >
          <option value={100}>Very Fast</option>
          <option value={300}>Fast</option>
          <option value={500}>Normal</option>
          <option value={1000}>Slow</option>
          <option value={2000}>Very Slow</option>
        </select>
      </div>
    {/if}
    
    {#if isFullAI}
      {#if !isPlaying && !isPaused}
        <Button variant="primary" size="md" onclick={startGame} disabled={!gameInstance || isGameOver}>
          Start AI Battle
        </Button>
      {/if}
      
      {#if isPlaying}
        <Button variant="secondary" size="md" onclick={pauseGame}>
          Pause
        </Button>
      {/if}
      
      {#if isPaused && !isGameOver}
        <Button variant="primary" size="md" onclick={resumeGame}>
          Resume
        </Button>
      {/if}
    {/if}
    
    {#if hasHuman && !isFullAI}
      <Button variant="secondary" size="md" onclick={undoMove} disabled={isPlaying}>
        Undo
      </Button>
    {/if}
    
    <Button variant="secondary" size="md" onclick={resetGame} disabled={isPlaying && !isPaused}>
      New Game
    </Button>
    
    {#if onBack}
      <Button variant="ghost" size="md" onclick={onBack}>
        {$_('game.menu.back')}
      </Button>
    {/if}
  </div>
</div>
