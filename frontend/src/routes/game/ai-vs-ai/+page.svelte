<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import GomokuBoard from '$lib/components/GomokuBoard.svelte';
  import Button from '$lib/components/Button.svelte';
  
  let board = $state(Array(19).fill(null).map(() => Array(19).fill(null)));
  let gameInstance: any = $state(null);
  let gameStatus = $state('Initializing...');
  let isPlaying = $state(false);
  let isPaused = $state(false);
  let isGameOver = $state(false);
  let currentPlayer = $state('Black AI');
  let moveDelay = $state(500); // Delay between moves in milliseconds
  
  onMount(async () => {
    try {
      // Import WASM module
      const wasmModule = await import('$lib/wasm/gomoku');
      await wasmModule.default();
      
      // Create game instance with default 19x19 board and 5 in a row to win
      gameInstance = new wasmModule.GomokuGame(19, 5);
      console.log('WASM initialized successfully for AI vs AI!');
      
      // Initialize board
      updateBoardFromWasm();
      isGameOver = false;
      gameStatus = 'Ready to start AI vs AI game';
    } catch (error) {
      console.error('Failed to load WASM:', error);
      gameStatus = 'Error loading game engine';
    }
  });
  
  function updateBoardFromWasm() {
    if (!gameInstance) return;
    
    try {
      const boardState = JSON.parse(gameInstance.getBoardState());
      
      // Convert the board state to our 2D array format
      const newBoard = Array(19).fill(null).map(() => Array(19).fill(null));
      
      // Max player (first player) uses black stones
      boardState.max_positions.forEach(([row, col]: [number, number]) => {
        newBoard[row][col] = 'black';
      });
      
      // Min player (second player) uses white stones
      boardState.min_positions.forEach(([row, col]: [number, number]) => {
        newBoard[row][col] = 'white';
      });
      
      board = newBoard;
      
      // Update current player display
      currentPlayer = boardState.current_player === 'Max' ? 'Black AI' : 'White AI';
    } catch (error) {
      console.error('Error updating board:', error);
    }
  }
  
  async function playAIvsAI() {
    if (!gameInstance || isPlaying) return;
    
    isPlaying = true;
    isPaused = false;
    isGameOver = false;
    gameStatus = 'AI vs AI game in progress...';
    
    while (!isGameOver && !isPaused) {
      try {
        // Get current player before move
        const boardState = JSON.parse(gameInstance.getBoardState());
        const playerName = boardState.current_player === 'Max' ? 'Black AI' : 'White AI';
        gameStatus = `${playerName} is thinking...`;
        
        // Wait for the delay to make it visible
        await new Promise(resolve => setTimeout(resolve, moveDelay));
        
        if (isPaused) break;
        
        // Get AI move with depth 3 (moderate difficulty)
        const aiMoveResult = JSON.parse(gameInstance.getAIMove(3));
        
        if (!aiMoveResult || aiMoveResult.row === undefined) {
          console.error('AI failed to find a move');
          gameStatus = 'AI error - no move found';
          break;
        }
        
        // Make the AI move
        const moveResult = JSON.parse(gameInstance.makeMove(aiMoveResult.row, aiMoveResult.col));
        
        if (!moveResult.success) {
          console.error('AI move failed:', moveResult.error);
          gameStatus = `AI move error: ${moveResult.error}`;
          break;
        }
        
        // Update the board display
        updateBoardFromWasm();
        
        // Check if game is over
        if (gameInstance.isGameOver()) {
          isGameOver = true;
          const result = JSON.parse(gameInstance.getGameResult());
          const winnerName = result.winner === 'Max' ? 'Black AI' : result.winner === 'Min' ? 'White AI' : 'Draw';
          gameStatus = `Game Over! Winner: ${winnerName} 🎉`;
          isPlaying = false;
          break;
        }
        
      } catch (error) {
        console.error('Error during AI move:', error);
        gameStatus = `Error: ${error}`;
        isPlaying = false;
        break;
      }
    }
    
    if (isPaused) {
      gameStatus = 'Game paused';
    }
    
    isPlaying = false;
  }
  
  function startGame() {
    if (!gameInstance) return;
    playAIvsAI();
  }
  
  function pauseGame() {
    isPaused = true;
    isPlaying = false;
  }
  
  function resumeGame() {
    if (!gameInstance || isGameOver) return;
    playAIvsAI();
  }
  
  function resetGame() {
    if (gameInstance) {
      gameInstance.reset();
      updateBoardFromWasm();
      isGameOver = false;
      gameStatus = 'Ready to start AI vs AI game';
      isPlaying = false;
      isPaused = false;
    }
  }
  
  function handleCellClick(row: number, col: number) {
    // No manual moves allowed in AI vs AI mode
    console.log('Manual moves disabled in AI vs AI mode');
  }
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <div class="text-center mb-6">
      <h1 class="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-2">
        AI vs AI
      </h1>
      <p class="text-xl font-bold mb-2" style="color: {isPlaying ? '#FF00FF' : '#00FFFF'};">
        {gameStatus}
      </p>
      <p class="text-lg text-white/80">
        Current turn: <span class="font-bold" style="color: #00FFFF;">{currentPlayer}</span>
      </p>
    </div>
    
    <GomokuBoard {board} onCellClick={handleCellClick} />
    
    <div class="flex justify-center items-center gap-4 mt-8 flex-wrap">
      <div class="flex items-center gap-2">
        <label for="speed" class="text-white/90 text-sm">Speed:</label>
        <select 
          id="speed"
          bind:value={moveDelay}
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
      
      <Button variant="secondary" size="md" onclick={resetGame} disabled={isPlaying}>
        New Game
      </Button>
      
      <Button variant="ghost" size="md" onclick={() => goto('/game')}>
        {$_('game.menu.back')}
      </Button>
    </div>
  </div>
</div>
