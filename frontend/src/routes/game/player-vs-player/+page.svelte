<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { _ } from 'svelte-i18n';
  import GomokuBoard from '$lib/components/GomokuBoard.svelte';
  import Button from '$lib/components/Button.svelte';
  
  let board = $state(Array(19).fill(null).map(() => Array(19).fill(null)));
  let gameInstance: any = null;
  let gameStatus = $state('Black\'s turn');
  let currentPlayer = $state('Black');
  
  onMount(async () => {
    try {
      // Import WASM module
      const wasmModule = await import('$lib/wasm/gomoku');
      await wasmModule.default();
      
      // Create game instance with default 19x19 board and 5 in a row to win
      gameInstance = new wasmModule.GomokuGame(19, 5);
      console.log('WASM initialized successfully for Player vs Player!');
      
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
      currentPlayer = boardState.current_player === 'Max' ? 'Black' : 'White';
    } catch (error) {
      console.error('Error updating board:', error);
    }
  }
  
  async function handleCellClick(row: number, col: number) {
    if (!gameInstance) {
      return;
    }
    
    // Check if position is valid and empty
    if (!board[row] || board[row][col] !== null) {
      return;
    }
    
    // Check if game is already over
    if (gameInstance.isGameOver()) {
      return;
    }
    
    try {
      // Make the move
      const moveResult = JSON.parse(gameInstance.makeMove(row, col));
      
      if (!moveResult.success) {
        console.log('Invalid move:', moveResult.error);
        gameStatus = moveResult.error || 'Invalid move';
        return;
      }
      
      updateBoardFromWasm();
      
      // Check if game is over
      if (gameInstance.isGameOver()) {
        const winner = gameInstance.getWinner();
        if (winner === 'Max') {
          gameStatus = 'Black wins! 🎉';
        } else if (winner === 'Min') {
          gameStatus = 'White wins! 🎉';
        } else {
          gameStatus = 'Draw!';
        }
      } else {
        // Update status for next player
        const boardState = JSON.parse(gameInstance.getBoardState());
        const nextPlayer = boardState.current_player === 'Max' ? 'Black' : 'White';
        gameStatus = `${nextPlayer}'s turn`;
      }
      
    } catch (error) {
      console.error('Error making move:', error);
      gameStatus = 'Error - try again';
    }
  }
  
  function resetGame() {
    if (gameInstance) {
      gameInstance.reset();
      updateBoardFromWasm();
      currentPlayer = 'Black';
      gameStatus = 'Black\'s turn';
    }
  }
  
  function undoMove() {
    if (gameInstance && gameInstance.undoMove()) {
      updateBoardFromWasm();
      const boardState = JSON.parse(gameInstance.getBoardState());
      const player = boardState.current_player === 'Max' ? 'Black' : 'White';
      gameStatus = `${player}'s turn`;
    }
  }
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <div class="text-center mb-6">
      <h1 class="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-2">
        Player vs Player
      </h1>
      <p class="text-2xl font-bold" style="color: {currentPlayer === 'Black' ? '#FF66B2' : '#3399FF'};">
        {gameStatus}
      </p>
    </div>
    
    <GomokuBoard {board} onCellClick={handleCellClick} />
    
    <div class="flex justify-center gap-4 mt-8 flex-wrap">
      <Button variant="ghost" size="md" onclick={() => goto('/game')}>
        {$_('game.menu.back')}
      </Button>
      <Button variant="secondary" size="md" onclick={undoMove}>
        Undo
      </Button>
      <Button variant="secondary" size="md" onclick={resetGame}>
        New Game
      </Button>
    </div>
  </div>
</div>
