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
      // Import WASM module
      const wasmModule = await import('$lib/wasm/gomoku');
      await wasmModule.default();
      
      // Create game instance with settings from store
      gameInstance = new wasmModule.GomokuGame($gameSettings.boardSize, $gameSettings.winCondition);
      console.log(`WASM initialized successfully! Board: ${$gameSettings.boardSize}x${$gameSettings.boardSize}, Win: ${$gameSettings.winCondition}`);
      
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
      console.error('Failed to load WASM:', error);
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
      const boardState = JSON.parse(gameInstance.getBoardState());
      const size = boardState.size;
      
      // Convert the board state to our 2D array format
      const newBoard = Array(size).fill(null).map(() => Array(size).fill(null));
      
      // Max player (player 1) uses black stones
      boardState.max_positions.forEach(([row, col]: [number, number]) => {
        newBoard[row][col] = 'black';
      });
      
      // Min player (player 2) uses white stones
      boardState.min_positions.forEach(([row, col]: [number, number]) => {
        newBoard[row][col] = 'white';
      });
      
      board = newBoard;
      
      // Update current player
      currentPlayer = boardState.current_player === 'Max' ? 1 : 2;
      currentPlayerName = currentPlayer === 1 ? player1Name : player2Name;
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
      const moveResult = JSON.parse(gameInstance.makeMove(row, col));
      
      if (moveResult.success) {
        totalMoves++;
        updateBoardFromWasm();
        
        // Update captures
        const boardState = JSON.parse(gameInstance.getBoardState());
        player1Captures = boardState.max_captures || 0;
        player2Captures = boardState.min_captures || 0;
        
        // Check for game over (use moveResult which already has game_over status)
        if (moveResult.game_over) {
          isGameOver = true;
          if (moveResult.winner) {
            const winnerName = moveResult.winner === 'Max' ? player1Name : player2Name;
            gameStatus = `${winnerName} wins! (${moveResult.win_reason})`;
          } else {
            gameStatus = 'Game Over - Draw';
          }
          isPlaying = false;
        }
        
        waitingForHumanMove = false;
        return true;
      }
      return false;
    } catch (error) {
      console.error('Move error:', error);
      return false;
    }
  }
  
  async function getAIMove(): Promise<{row: number, col: number} | null> {
    if (!gameInstance) return null;
    
    try {
      const startTime = performance.now();
      // Pass depth and time limit in milliseconds
      const aiMoveResult = JSON.parse(gameInstance.getAIMove(aiDepth, $gameSettings.aiMaxThinkingTime));
      const endTime = performance.now();
      
      // Track AI thinking time
      lastMoveTime = Math.round(endTime - startTime);
      totalThinkingTime += lastMoveTime;
      aiMoveCount++;
      
      if (!aiMoveResult || aiMoveResult.row === undefined) {
        console.error('AI failed to find a move');
        return null;
      }
      
      return { row: aiMoveResult.row, col: aiMoveResult.col };
    } catch (error) {
      console.error('AI move error:', error);
      return null;
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
      
      // Auto-restart for AI vs AI
      if (autoStart && isFullAI) {
        setTimeout(() => startGame(), 500);
      } else if (hasHuman) {
        startGame();
      }
    }
  }
  
  function undoMove() {
    if (gameInstance && gameInstance.undoMove()) {
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
    </p>
    <p class="text-lg text-white/80">
      Current turn: <span class="font-bold" style="color: #00FFFF;">{currentPlayerName}</span>
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
