<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import GomokuBoard from '$lib/components/GomokuBoard.svelte';
  import Button from '$lib/components/Button.svelte';
  
  let board = $state(Array(19).fill(null).map(() => Array(19).fill(null)));
  let gameInstance: any = null;
  let isPlayerTurn = $state(true);
  let gameStatus = $state('Playing...');
  let isThinking = $state(false);
  
  onMount(async () => {
    try {
      // Import WASM module
      const wasmModule = await import('$lib/wasm/gomoku');
      await wasmModule.default();
      
      // Create game instance
      gameInstance = new wasmModule.GomokuGame();
      console.log('WASM initialized successfully!');
      
      // Initialize board
      updateBoardFromWasm();
    } catch (error) {
      console.error('Failed to load WASM:', error);
      gameStatus = 'Error loading game engine';
    }
  });
  
  function updateBoardFromWasm() {
    if (!gameInstance) return;
    
    try {
      const boardState = JSON.parse(gameInstance.getBoardState());
      board = boardState.board;
    } catch (error) {
      console.error('Error updating board:', error);
    }
  }
  
  async function handleCellClick(row: number, col: number) {
    if (!gameInstance || !isPlayerTurn || isThinking) {
      return;
    }
    
    // Check if position is valid and empty
    if (!board[row] || board[row][col] !== null) {
      return;
    }
    
    try {
      // Make player move
      const moveResult = JSON.parse(gameInstance.makeMove(row, col));
      
      if (!moveResult.success) {
        console.log('Invalid move:', moveResult.error);
        return;
      }
      
      updateBoardFromWasm();
      
      // Check if game is over
      if (gameInstance.isGameOver()) {
        const result = JSON.parse(gameInstance.getGameResult());
        gameStatus = result.winner === 'Black' ? 'You Win! 🎉' : result.winner === 'White' ? 'AI Wins!' : 'Draw!';
        isPlayerTurn = false;
        return;
      }
      
      // AI's turn
      isPlayerTurn = false;
      isThinking = true;
      gameStatus = 'AI is thinking...';
      
      // Let UI update before AI move
      setTimeout(async () => {
        try {
          const aiMoveResult = JSON.parse(gameInstance.getAIMove());
          
          if (aiMoveResult.success) {
            updateBoardFromWasm();
            
            // Check if game is over after AI move
            if (gameInstance.isGameOver()) {
              const result = JSON.parse(gameInstance.getGameResult());
              gameStatus = result.winner === 'Black' ? 'You Win! 🎉' : result.winner === 'White' ? 'AI Wins!' : 'Draw!';
            } else {
              isPlayerTurn = true;
              gameStatus = 'Your turn';
            }
          }
        } catch (error) {
          console.error('AI move error:', error);
          gameStatus = 'AI error - your turn again';
          isPlayerTurn = true;
        }
        isThinking = false;
      }, 300);
      
    } catch (error) {
      console.error('Error making move:', error);
      gameStatus = 'Error - try again';
    }
  }
  
  function resetGame() {
    if (gameInstance) {
      gameInstance.reset();
      updateBoardFromWasm();
      isPlayerTurn = true;
      gameStatus = 'Your turn';
      isThinking = false;
    }
  }
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <div class="text-center mb-6">
      <h1 class="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-2">
        Player vs AI
      </h1>
      <p class="text-2xl font-bold" style="color: {isThinking ? '#FF00FF' : '#00FFFF'};">
        {gameStatus}
      </p>
    </div>
    
    <GomokuBoard {board} onCellClick={handleCellClick} />
    
    <div class="flex justify-center gap-4 mt-8">
      <Button variant="ghost" size="md" onclick={() => goto('/game')}>
        {$_('game.menu.back')}
      </Button>
      <Button variant="secondary" size="md" onclick={resetGame}>
        New Game
      </Button>
    </div>
  </div>
</div>
