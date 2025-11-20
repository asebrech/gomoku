<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { untrack } from 'svelte';
  import GomokuBoard from './GomokuBoard.svelte';
  import GameStats from './GameStats.svelte';
  import Scoreboard from './Scoreboard.svelte';
  import WinnerModal from './WinnerModal.svelte';
  import Button from './Button.svelte';
  import Toggle from './Toggle.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  import { currentTheme } from '$lib/theme/themeStore';
  import { playStoneSound, playWinSound, playLoseSound } from '$lib/utils/soundEffects';
  import { matchHistory, generateMatchId, type MatchData } from '$lib/stores/matchHistory';
  
  interface Props {
    player1Type: 'human' | 'ai';
    player2Type: 'human' | 'ai';
    player1Name?: string;
    player2Name?: string;
    aiDepth?: number;
    autoStart?: boolean;
    moveDelay?: number;
    showSpeedControl?: boolean;
    resumeMatchId?: string; // Match ID to resume
    onBack?: () => void;
  }
  
  let {
    player1Type,
    player2Type,
    player1Name = player1Type === 'ai' ? $_('game.players.ai1') : $_('game.players.player1'),
    player2Name = player2Type === 'ai' ? $_('game.players.ai2') : $_('game.players.player2'),
    aiDepth = $gameSettings.aiDepth,
    autoStart = false,
    moveDelay = 500,
    showSpeedControl = false,
    resumeMatchId = undefined,
    onBack
  }: Props = $props();
  
  let board = $state(Array(19).fill(null).map(() => Array(19).fill(null)));
  let gameInstance: any = $state(null);
  let gameStatus = $state($_('game.status.initializing'));
  let isPlaying = $state(false);
  let isPaused = $state(false);
  let isGameOver = $state(false);
  let currentPlayer = $state(1); // 1 or 2
  let currentPlayerName = $state(player1Name);
  let winnerPlayer = $state<number | null>(null); // Track winner (1 or 2)
  let showWinnerModal = $state(false); // Control winner modal visibility
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
  
  // Forced capture positions (when there's a breakable five)
  let forcedCapturePositions = $state<Array<{row: number, col: number}>>([]);
  
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
  
  // Match tracking for history
  let currentMatchId = $state<string | null>(null);
  let gameStartTime = $state<number | null>(null);
  let matchAddedToStore = $state(false); // Track if match has been added to store
  let autoSaveInterval: NodeJS.Timeout | null = null;
  
  const isFullAI = player1Type === 'ai' && player2Type === 'ai';
  const hasHuman = player1Type === 'human' || player2Type === 'human';
  
  // Determine if a human can currently play (for showing hover preview)
  const canHumanPlay = $derived(() => {
    if (isGameOver || needsAIContinue || isAIThinking) return false;
    const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
    // Show preview if:
    // 1. It's a human's turn
    // 2. Game hasn't started yet (before first move)
    // 3. OR game is playing and waiting for human move
    if (currentPlayerType !== 'human') return false;
    return !isPlaying || waitingForHumanMove;
  });
  
  // Calculate turn number (a turn is one black move + one white move)
  const turnNumber = $derived(Math.ceil(totalMoves / 2));
  
  // Generate game mode display name
  const gameModeDisplay = $derived(() => {
    if (player1Type === 'ai' && player2Type === 'ai') return $_('history.match.gameMode.aiVsAi');
    if (player1Type === 'human' && player2Type === 'human') return $_('history.match.gameMode.playerVsPlayer');
    return $_('history.match.gameMode.playerVsAi');
  });
  
  // Get move history from WASM
  function getMoveHistory(): Array<{ row: number; col: number }> {
    if (!gameInstance) return [];
    
    try {
      const history = gameInstance.get_move_history();
      if (!history) return [];
      
      const moves: Array<{ row: number; col: number }> = [];
      for (let i = 0; i < history.length; i++) {
        const move = history[i];
        if (move && typeof move.row === 'number' && typeof move.col === 'number') {
          moves.push({ row: move.row, col: move.col });
        }
      }
      return moves;
    } catch (error) {
      console.error('Error getting move history:', error);
      return [];
    }
  }
  
  // Resume a match from history
  async function resumeMatch(match: MatchData) {
    if (!gameInstance) {
      console.error('Cannot resume: game instance not initialized');
      return;
    }
    
    // Replay all moves from history
    let lastMove = null;
    for (const move of match.moveHistory) {
      if (gameInstance.is_move_legal_coords(move.row, move.col)) {
        gameInstance.make_move_coords(move.row, move.col);
        totalMoves++;
        lastMove = move; // Track the last move
      } else {
        console.error('Invalid move in history:', move);
      }
    }
    
    // Update board from WASM state
    updateBoardFromWasm();
    
    // Set the last move position for highlighting
    if (lastMove) {
      lastMovePosition = { row: lastMove.row, col: lastMove.col };
    }
    
    // Restore match metadata
    currentMatchId = match.id;
    gameStartTime = match.date.getTime();
    player1Captures = match.player1Captures;
    player2Captures = match.player2Captures;
    matchAddedToStore = true; // Match already exists in store
    
    // Update game state - use the same logic as startGame
    shouldStop = false;
    isPlaying = true;
    isPaused = false;
    isGameOver = false;
    gameStatus = 'Game resumed';
    
    // Start auto-save
    startAutoSave();
    
    // Start the game loop which will handle both human and AI turns
    playGameLoop();
  }
  
  // Save or update match in history
  function saveMatchToHistory() {
    if (!gameInstance) {
      return;
    }
    
    if (!gameStartTime) {
      return;
    }
    
    const now = Date.now();
    const duration = Math.floor((now - gameStartTime) / 1000); // seconds
    
    const matchData: MatchData = {
      id: currentMatchId || generateMatchId(),
      date: new Date(gameStartTime),
      player1Name: player1Name,
      player2Name: player2Name,
      player1Type: player1Type,
      player2Type: player2Type,
      boardSize: $gameSettings.boardSize,
      winCondition: $gameSettings.winCondition,
      aiDepth: (player1Type === 'ai' || player2Type === 'ai') ? aiDepth : undefined,
      moveHistory: getMoveHistory(),
      winner: isGameOver ? winnerPlayer : null,
      status: isGameOver ? 'finished' : 'ongoing',
      totalMoves: totalMoves,
      player1Captures: player1Captures,
      player2Captures: player2Captures,
      duration: duration
    };
    
    if (matchAddedToStore) {
      // Update existing match in store
      matchHistory.updateMatch(matchData.id, matchData);
    } else {
      // Add new match to store
      currentMatchId = matchData.id;
      matchHistory.addMatch(matchData);
      matchAddedToStore = true;
    }
  }
  
  // Start auto-save interval (every 30 seconds)
  function startAutoSave() {
    if (autoSaveInterval) {
      clearInterval(autoSaveInterval);
    }
    
    autoSaveInterval = setInterval(() => {
      if (gameStartTime && totalMoves > 0) {
        saveMatchToHistory();
      }
    }, 30000); // 30 seconds
  }
  
  // Stop auto-save interval
  function stopAutoSave() {
    if (autoSaveInterval) {
      clearInterval(autoSaveInterval);
      autoSaveInterval = null;
    }
  }
  
  onMount(async () => {
    try {
      // Import WASM module (already initialized in root layout)
      const wasmModule = await import('$lib/wasm/pkg/gomoku');
      
      // Create game instance with settings from store
      gameInstance = new wasmModule.WasmGameState($gameSettings.boardSize, $gameSettings.winCondition);
      
      // Initialize board with correct size
      board = Array($gameSettings.boardSize).fill(null).map(() => Array($gameSettings.boardSize).fill(null));
      updateBoardFromWasm();
      isGameOver = false;
      gameStatus = 'Ready to start';
      
      // Check if we need to resume a match
      if (resumeMatchId) {
        const matchToResume = $matchHistory.find(m => m.id === resumeMatchId);
        if (matchToResume && matchToResume.status === 'ongoing') {
          await resumeMatch(matchToResume);
          // Clear the resume flag
          sessionStorage.removeItem('resumeMatchId');
        }
      } else if (autoStart && isFullAI) {
        // Auto-start only for AI vs AI mode when autoStart is true
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
    
    // Clear auto-save interval
    if (autoSaveInterval) {
      clearInterval(autoSaveInterval);
      autoSaveInterval = null;
    }
    
    // Save one last time before destroying (if game has started)
    if (currentMatchId && gameStartTime) {
      saveMatchToHistory();
    }
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
      
      // Update double-three positions (function has its own guards)
      updateDoubleThreePositions();
      
      // Update forced capture positions (function has its own guards)
      updateForcedCapturePositions();
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
    
    // Determine if current player is human
    const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
    const isCurrentPlayerHuman = currentPlayerType === 'human';
    
    // Always show for human players, only show for AI/opponent if toggle is on
    if (!isCurrentPlayerHuman && !showDoubleThree) {
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
  
  function updateForcedCapturePositions() {
    // Guard against calling when game instance doesn't exist or game is over
    if (!gameInstance || isGameOver) {
      forcedCapturePositions = [];
      return;
    }
    
    try {
      const legalMoves = gameInstance.get_legal_moves();
      
      if (!legalMoves) {
        forcedCapturePositions = [];
        return;
      }
      
      // Get total number of empty positions on the board
      let emptyCount = 0;
      for (let row = 0; row < board.length; row++) {
        for (let col = 0; col < board[row].length; col++) {
          if (board[row][col] === null) emptyCount++;
        }
      }
      
      // Always show dots for legal moves
      forcedCapturePositions = [];
      for (let i = 0; i < legalMoves.length; i++) {
        const pos = legalMoves[i];
        if (pos && typeof pos.row === 'number' && typeof pos.col === 'number') {
          forcedCapturePositions.push({ row: pos.row, col: pos.col });
        }
      }
      
      // If moves are restricted, log it
      if (legalMoves.length < emptyCount) {
        console.log('[DEBUG] Moves restricted! Showing', forcedCapturePositions.length, 'legal positions out of', emptyCount, 'empty positions');
      }
    } catch (error) {
      console.error('Error getting forced capture positions:', error);
      forcedCapturePositions = [];
    }
  }
  
  async function calculateAIHint() {
    // Only calculate hints for human players during their turn
    const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
    if (!gameInstance || isGameOver || !showAIHint || currentPlayerType !== 'human') {
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
      
      // Play stone placement sound
      playStoneSound();
      
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
          showWinnerModal = true; // Show the winner modal
          
          // Play win/lose sounds based on game mode
          if (player1Type === 'human' && player2Type === 'ai') {
            // Player vs AI: Human is player 1
            if (winnerPlayer === 1) {
              playWinSound();
            } else {
              playLoseSound();
            }
          } else if (player1Type === 'ai' && player2Type === 'human') {
            // Player vs AI: Human is player 2
            if (winnerPlayer === 2) {
              playWinSound();
            } else {
              playLoseSound();
            }
          } else if (player1Type === 'human' && player2Type === 'human') {
            // Player vs Player: Just play win sound for whoever wins
            playWinSound();
          }
          // AI vs AI: no sounds (both are AI)
        } else {
          winnerPlayer = null;
          gameStatus = 'Game Over - Draw';
        }
        isPlaying = false;
        
        // Save match on game completion
        stopAutoSave();
        saveMatchToHistory();
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
    
    // Initialize match tracking if this is a new game
    if (!gameStartTime) {
      gameStartTime = Date.now();
      currentMatchId = generateMatchId();
      startAutoSave();
    }
    
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
      
      // Stop auto-save and save final state if game was in progress
      stopAutoSave();
      if (gameStartTime && totalMoves > 0) {
        saveMatchToHistory();
      }
      
      // Now reset the game state
      shouldStop = false;
      gameInstance.reset();
      isGameOver = false;
      isPaused = false;
      updateBoardFromWasm();
      waitingForHumanMove = false;
      needsAIContinue = false;
      winnerPlayer = null;
      showWinnerModal = false;
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
      
      // Reset match tracking for new game
      currentMatchId = null;
      gameStartTime = null;
      matchAddedToStore = false;
      
      // Auto-restart for AI vs AI only
      if (autoStart && isFullAI) {
        setTimeout(() => startGame(), 500);
      }
    }
  }
  
  async function undoMove() {
    if (gameInstance && totalMoves > 0) {
      // First, stop any running game loop to prevent race conditions
      const wasPlaying = isPlaying;
      shouldStop = true;
      isPlaying = false;
      waitingForHumanMove = false;
      needsAIContinue = false;
      
      // Wait for the game loop to stop
      if (wasPlaying) {
        await new Promise(resolve => setTimeout(resolve, 150));
      }
      
      // Now perform the undo
      gameInstance.undo_last_move();
      totalMoves--;
      updateBoardFromWasm();
      gameStatus = 'Move undone';
      
      // Clear AI hint and last move position after undo
      aiHintPosition = null;
      lastMovePosition = null;
      
      // Reset shouldStop flag
      shouldStop = false;
      
      // Check whose turn it is now after undo (currentPlayer is already updated by updateBoardFromWasm)
      const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
      
      // If it's now an AI's turn and we have at least one human player
      if (currentPlayerType === 'ai' && hasHuman) {
        needsAIContinue = true;
        waitingForHumanMove = false;
        gameStatus = 'Click Continue to let AI play';
      } else if (currentPlayerType === 'human') {
        // It's a human's turn
        needsAIContinue = false;
        
        // If there was a game running, restart it so the human can play
        if (wasPlaying && hasHuman) {
          waitingForHumanMove = true;
          isPlaying = true;
          gameStatus = 'Your turn';
          playGameLoop();
        } else {
          waitingForHumanMove = false;
          gameStatus = 'Your turn - Click Start to continue';
        }
      } else {
        // AI vs AI or other edge case
        needsAIContinue = false;
        waitingForHumanMove = false;
        
        // If the game was playing, set it to paused so the "Resume" button appears
        if (wasPlaying && isFullAI) {
          isPaused = true;
          gameStatus = 'Game paused';
        } else {
          gameStatus = 'Move undone - Click Start to continue';
        }
      }
    }
  }
  
  function continueAfterUndo() {
    if (needsAIContinue) {
      needsAIContinue = false;
      // Restart the game loop to handle AI turn
      shouldStop = false;
      isPlaying = true;
      gameStatus = 'AI is thinking...';
      playGameLoop();
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
    // Update double-three positions when toggle changes, board updates, or current player changes
    // Always show for human players, only show for opponent if toggle is on
    // Use untrack to prevent infinite loops when updating doubleThreePositions
    if (gameInstance && !isGameOver) {
      // Track currentPlayer so we update when turn changes
      const _ = currentPlayer;
      untrack(() => {
        updateDoubleThreePositions();
      });
    } else {
      // Clear positions when game is over
      untrack(() => {
        doubleThreePositions = [];
      });
    }
  });
  
  $effect(() => {
    // Calculate AI hint when toggle is on and it's human's turn
    const currentPlayerType = currentPlayer === 1 ? player1Type : player2Type;
    if (gameInstance && !isGameOver && showAIHint && currentPlayerType === 'human' && hasHuman) {
      calculateAIHint();
    } else if (!showAIHint) {
      // Clear hint when toggle is off
      untrack(() => {
        aiHintPosition = null;
      });
    }
  });
</script>

<div class="h-full w-full flex flex-col items-center overflow-hidden">
  <!-- Desktop Layout: Board and Stats Side by Side -->
  <div class="hidden md:flex justify-center items-center gap-6 flex-1 min-h-0 px-4 py-4 w-full max-w-screen-xl">
    <!-- Board Column with Scoreboard -->
    <div class="flex flex-col items-center justify-center gap-3 flex-1 min-w-0 max-w-[600px]">
      <!-- Board Container with aspect ratio constraint -->
      <div class="w-full flex-shrink-0 flex items-center justify-center" style="aspect-ratio: 1/1; max-width: min(100%, calc(100vh - 16rem)); max-height: calc(100vh - 16rem);">
        <div class="w-full h-full">
          <GomokuBoard 
            {board} 
            onCellClick={handleCellClick}
            currentPlayer={currentPlayer === 1 ? 'black' : 'white'}
            {doubleThreePositions}
            {forcedCapturePositions}
            {aiHintPosition}
            {lastMovePosition}
            canHumanPlay={canHumanPlay()}
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
          totalMoves={turnNumber}
        />
      </div>
    </div>
    
    <!-- Stats Panel -->
    <div class="flex-shrink-0 overflow-y-auto">
      <GameStats
        gameMode={gameModeDisplay()}
        boardSize={$gameSettings.boardSize}
        aiDepth={player1Type === 'ai' || player2Type === 'ai' ? aiDepth : undefined}
        currentPlayer={currentPlayerName}
        winnerPlayer={winnerPlayer}
        player1Name={player1Name}
        player2Name={player2Name}
        totalMoves={turnNumber}
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
  <div class="md:hidden flex flex-col items-center w-full h-full px-2 py-4 gap-3 overflow-y-auto">
    <!-- Board -->
    <div class="w-full flex-shrink-0" style="max-width: min(95vw, calc(100vh - 20rem)); aspect-ratio: 1/1;">
      <GomokuBoard 
        {board} 
        onCellClick={handleCellClick}
        currentPlayer={currentPlayer === 1 ? 'black' : 'white'}
        {doubleThreePositions}
        {forcedCapturePositions}
        {aiHintPosition}
        {lastMovePosition}
        canHumanPlay={canHumanPlay()}
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
              {cleanWinnerName} {$_('game.stats.wins')}
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
          totalMoves={turnNumber}
        />
      </div>

      <!-- Mobile Controls -->
      <div class="flex flex-wrap gap-2 justify-center">
        <!-- Game Controls - Start button only for AI vs AI -->
        {#if !isPlaying && !isPaused && isFullAI}
          <Button variant="primary" size="sm" onclick={startGame} disabled={!gameInstance || isGameOver}>
            {$_('game.stats.startMatch')}
          </Button>
        {/if}
        
        <!-- AI vs AI Controls -->
        {#if isFullAI}
          {#if isPlaying}
            <Button variant="primary" size="sm" onclick={pauseGame}>
              {$_('game.stats.pause')}
            </Button>
          {/if}
          
          {#if isPaused && !isGameOver}
            <Button variant="primary" size="sm" onclick={resumeGame}>
              {$_('game.stats.resume')}
            </Button>
          {/if}
        {/if}
        
        <!-- Undo button -->
        <Button variant="primary" size="sm" onclick={undoMove} disabled={totalMoves === 0 || isGameOver}>
          {$_('game.undo')}
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

<!-- Winner Modal -->
{#if showWinnerModal && winnerPlayer !== null}
  <WinnerModal
    {winnerPlayer}
    {player1Name}
    {player2Name}
    onClose={() => showWinnerModal = false}
    onNewGame={resetGame}
  />
{/if}
