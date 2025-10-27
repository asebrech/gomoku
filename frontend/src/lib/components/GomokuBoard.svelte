<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    size?: number;
    board?: Array<Array<'black' | 'white' | null>>;
    onCellClick?: (row: number, col: number) => void;
    currentPlayer?: 'black' | 'white';
  }
  
  let { 
    board = Array(19).fill(null).map(() => Array(19).fill(null)),
    size = board.length || 19,
    currentPlayer = 'black',
    onCellClick 
  }: Props = $props();
  
  // Reactive size based on board
  const actualSize = $derived(board.length || size);
  const cellSize = 30; // px
  const boardSize = $derived(cellSize * (actualSize - 1));
  const padding = 40;
  const totalSize = $derived(boardSize + (padding * 2));
  
  // Hover state
  let hoverRow = $state<number | null>(null);
  let hoverCol = $state<number | null>(null);
  
  function handleClick(event: MouseEvent) {
    const svg = event.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const x = event.clientX - rect.left - padding;
    const y = event.clientY - rect.top - padding;
    
    const col = Math.round(x / cellSize);
    const row = Math.round(y / cellSize);
    
    if (row >= 0 && row < actualSize && col >= 0 && col < actualSize) {
      console.log(`Clicked: row ${row}, col ${col}`);
      onCellClick?.(row, col);
    }
  }
  
  function handleMouseMove(event: MouseEvent) {
    const svg = event.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    const x = event.clientX - rect.left - padding;
    const y = event.clientY - rect.top - padding;
    
    const col = Math.round(x / cellSize);
    const row = Math.round(y / cellSize);
    
    if (row >= 0 && row < actualSize && col >= 0 && col < actualSize) {
      // Only show hover if the cell is empty
      if (!board[row]?.[col]) {
        hoverRow = row;
        hoverCol = col;
      } else {
        hoverRow = null;
        hoverCol = null;
      }
    } else {
      hoverRow = null;
      hoverCol = null;
    }
  }
  
  function handleMouseLeave() {
    hoverRow = null;
    hoverCol = null;
  }
</script>

<div class="flex items-center justify-center w-full p-8">
  <div class="rounded-lg p-8" style="background: {$currentTheme.background}99; border: 3px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};">
    <svg 
      width={totalSize} 
      height={totalSize}
      class="cursor-pointer"
      onclick={handleClick}
      onmousemove={handleMouseMove}
      onmouseleave={handleMouseLeave}
      onkeydown={(e) => e.key === 'Enter' && handleClick(e)}
      role="button"
      tabindex="0"
    >
      <!-- Board background -->
      <rect 
        x={padding} 
        y={padding} 
        width={boardSize} 
        height={boardSize}
        fill="transparent"
        stroke="none"
      />
      
      <!-- Grid lines -->
      {#each Array(actualSize) as _, i}
        <!-- Horizontal lines -->
        <line
          x1={padding}
          y1={padding + i * cellSize}
          x2={padding + boardSize}
          y2={padding + i * cellSize}
          stroke="{$currentTheme.primary}C0"
          stroke-width="2"
        />
        <!-- Vertical lines -->
        <line
          x1={padding + i * cellSize}
          y1={padding}
          x2={padding + i * cellSize}
          y2={padding + boardSize}
          stroke="{$currentTheme.primary}C0"
          stroke-width="2"
        />
      {/each}
      
      <!-- Stones -->
      {#each board as rowData, row}
        {#each rowData as stone, col}
          {#if stone}
            <circle
              cx={padding + col * cellSize}
              cy={padding + row * cellSize}
              r="13"
              fill={stone === 'black' ? `url(#player1Gradient)` : `url(#player2Gradient)`}
              filter="url(#stoneShadow)"
              class="animate-fade-in"
            />
          {/if}
        {/each}
      {/each}
      
      <!-- Hover ghost stone -->
      {#if hoverRow !== null && hoverCol !== null}
        <circle
          cx={padding + hoverCol * cellSize}
          cy={padding + hoverRow * cellSize}
          r="13"
          fill={currentPlayer === 'black' ? `url(#player1GhostGradient)` : `url(#player2GhostGradient)`}
          opacity="0.5"
          class="pointer-events-none"
        />
      {/if}
      
      <!-- Gradients for stones -->
      <defs>
        <!-- Player 1 (Black) stone gradient -->
        <radialGradient id="player1Gradient">
          <stop offset="30%" stop-color="{$currentTheme.stonePlayer1}" stop-opacity="0.9" />
          <stop offset="100%" stop-color="{$currentTheme.stonePlayer1}" stop-opacity="1" />
        </radialGradient>
        
        <!-- Player 2 (White) stone gradient -->
        <radialGradient id="player2Gradient">
          <stop offset="30%" stop-color="{$currentTheme.stonePlayer2}" stop-opacity="0.9" />
          <stop offset="100%" stop-color="{$currentTheme.stonePlayer2}" stop-opacity="1" />
        </radialGradient>
        
        <!-- Player 1 ghost stone gradient -->
        <radialGradient id="player1GhostGradient">
          <stop offset="30%" stop-color="{$currentTheme.stonePlayer1}" stop-opacity="0.5" />
          <stop offset="100%" stop-color="{$currentTheme.stonePlayer1}" stop-opacity="0.7" />
        </radialGradient>
        
        <!-- Player 2 ghost stone gradient -->
        <radialGradient id="player2GhostGradient">
          <stop offset="30%" stop-color="{$currentTheme.stonePlayer2}" stop-opacity="0.5" />
          <stop offset="100%" stop-color="{$currentTheme.stonePlayer2}" stop-opacity="0.7" />
        </radialGradient>
        
        <filter id="stoneShadow">
          <feDropShadow dx="2" dy="3" stdDeviation="2" flood-opacity="0.5"/>
        </filter>
      </defs>
    </svg>
  </div>
</div>
