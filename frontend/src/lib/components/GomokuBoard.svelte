<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    size?: number;
    board?: Array<Array<'black' | 'white' | null>>;
    onCellClick?: (row: number, col: number) => void;
    currentPlayer?: 'black' | 'white';
    doubleThreePositions?: Array<{row: number, col: number}>;
    aiHintPosition?: {row: number, col: number} | null;
    lastMovePosition?: {row: number, col: number} | null;
    canHumanPlay?: boolean;
  }
  
  let { 
    board = Array(19).fill(null).map(() => Array(19).fill(null)),
    size = board.length || 19,
    currentPlayer = 'black',
    onCellClick,
    doubleThreePositions = [],
    aiHintPosition = null,
    lastMovePosition = null,
    canHumanPlay = true
  }: Props = $props();
  
  // Window width for responsive calculations
  let windowWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 768);
  
  // Update window width on resize
  if (typeof window !== 'undefined') {
    const updateWidth = () => {
      windowWidth = window.innerWidth;
    };
    window.addEventListener('resize', updateWidth);
    updateWidth();
  }
  
  // Hover state
  let hoverRow = $state<number | null>(null);
  let hoverCol = $state<number | null>(null);
  
  // Reactive size calculations
  const actualSize = $derived(board.length || size);
  const isMobile = $derived(windowWidth < 768);
  
  // Simple, fixed sizes that work well on both mobile and desktop
  const cellSize = $derived(isMobile ? 18 : 22);
  const boardSize = $derived(cellSize * (actualSize - 1));
  const padding = $derived(isMobile ? 20 : 30);
  const totalSize = $derived(boardSize + (padding * 2));
  const stoneRadius = $derived(isMobile ? 9 : 11); // Increased stone size for better visibility
  
  function handleClick(event: MouseEvent) {
    const svg = event.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    
    // Calculate the scale factor between the actual SVG size and displayed size
    const scaleX = totalSize / rect.width;
    const scaleY = totalSize / rect.height;
    
    // Convert screen coordinates to SVG coordinates
    const svgX = (event.clientX - rect.left) * scaleX;
    const svgY = (event.clientY - rect.top) * scaleY;
    
    // Convert to board coordinates
    const x = svgX - padding;
    const y = svgY - padding;
    
    const col = Math.round(x / cellSize);
    const row = Math.round(y / cellSize);
    
    if (row >= 0 && row < actualSize && col >= 0 && col < actualSize) {
      onCellClick?.(row, col);
    }
  }
  
  function handleMouseMove(event: MouseEvent) {
    const svg = event.currentTarget as SVGSVGElement;
    const rect = svg.getBoundingClientRect();
    
    // Calculate the scale factor between the actual SVG size and displayed size
    const scaleX = totalSize / rect.width;
    const scaleY = totalSize / rect.height;
    
    // Convert screen coordinates to SVG coordinates
    const svgX = (event.clientX - rect.left) * scaleX;
    const svgY = (event.clientY - rect.top) * scaleY;
    
    // Convert to board coordinates
    const x = svgX - padding;
    const y = svgY - padding;
    
    const col = Math.round(x / cellSize);
    const row = Math.round(y / cellSize);
    
    if (row >= 0 && row < actualSize && col >= 0 && col < actualSize) {
      // Only show hover if the cell is empty, not a double-three position, and human can play
      const isDoubleThree = doubleThreePositions.some(pos => pos.row === row && pos.col === col);
      if (!board[row]?.[col] && !isDoubleThree && canHumanPlay) {
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
  
  function isDoubleThreePosition(row: number, col: number): boolean {
    return doubleThreePositions.some(pos => pos.row === row && pos.col === col);
  }
  
  function handleMouseLeave() {
    hoverRow = null;
    hoverCol = null;
  }
</script>

<div class="w-full h-full flex items-center justify-center">
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <svg 
    viewBox="0 0 {totalSize} {totalSize}"
    class="cursor-pointer block rounded-lg"
    style="width: 100%; height: 100%; max-width: 100%; max-height: 100%; aspect-ratio: 1/1; background: {$currentTheme.background}99; border: 3px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};"
    onclick={handleClick}
    onmousemove={handleMouseMove}
    onmouseleave={handleMouseLeave}
    role="button"
    tabindex="0"
    preserveAspectRatio="xMidYMid meet"
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
            <g>
              <circle
                cx={padding + col * cellSize}
                cy={padding + row * cellSize}
                r={stoneRadius}
                fill={stone === 'black' ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2}
                class="animate-fade-in"
              />
              <!-- Highlight ring for last move -->
              {#if lastMovePosition && lastMovePosition.row === row && lastMovePosition.col === col}
                <circle
                  cx={padding + col * cellSize}
                  cy={padding + row * cellSize}
                  r={stoneRadius + 3}
                  fill="none"
                  stroke="{$currentTheme.accent}"
                  stroke-width="2.5"
                  opacity="0.9"
                  class="animate-pulse"
                />
              {/if}
            </g>
          {/if}
        {/each}
      {/each}
      
      <!-- Double-three markers (red crosses) -->
      {#each doubleThreePositions as pos}
        {#if !board[pos.row]?.[pos.col]}
          <g class="animate-fade-in">
            <!-- Red X mark -->
            <line
              x1={padding + pos.col * cellSize - (isMobile ? 4 : 6)}
              y1={padding + pos.row * cellSize - (isMobile ? 4 : 6)}
              x2={padding + pos.col * cellSize + (isMobile ? 4 : 6)}
              y2={padding + pos.row * cellSize + (isMobile ? 4 : 6)}
              stroke="#FF0000"
              stroke-width="2"
              stroke-linecap="round"
            />
            <line
              x1={padding + pos.col * cellSize + (isMobile ? 4 : 6)}
              y1={padding + pos.row * cellSize - (isMobile ? 4 : 6)}
              x2={padding + pos.col * cellSize - (isMobile ? 4 : 6)}
              y2={padding + pos.row * cellSize + (isMobile ? 4 : 6)}
              stroke="#FF0000"
              stroke-width="2"
              stroke-linecap="round"
            />
            <!-- Optional: Add a subtle glow effect -->
            <circle
              cx={padding + pos.col * cellSize}
              cy={padding + pos.row * cellSize}
              r={isMobile ? "6" : "8"}
              fill="none"
              stroke="#FF0000"
              stroke-width="1"
              opacity="0.3"
            />
          </g>
        {/if}
      {/each}
      
      <!-- Hover ghost stone -->
      {#if hoverRow !== null && hoverCol !== null}
        <circle
          cx={padding + hoverCol * cellSize}
          cy={padding + hoverRow * cellSize}
          r={stoneRadius}
          fill={currentPlayer === 'black' ? `url(#player1GhostGradient)` : `url(#player2GhostGradient)`}
          opacity="0.5"
          class="pointer-events-none"
        />
      {/if}
      
      <!-- AI Hint indicator -->
      {#if aiHintPosition && !board[aiHintPosition.row]?.[aiHintPosition.col]}
        <g class="pointer-events-none animate-pulse">
          <!-- Pulsing ring around suggested move -->
          <circle
            cx={padding + aiHintPosition.col * cellSize}
            cy={padding + aiHintPosition.row * cellSize}
            r={isMobile ? "9" : "12"}
            fill="none"
            stroke="#FFD700"
            stroke-width="2"
            opacity="0.8"
          />
          <!-- Inner dot -->
          <circle
            cx={padding + aiHintPosition.col * cellSize}
            cy={padding + aiHintPosition.row * cellSize}
            r={isMobile ? "3" : "4"}
            fill="#FFD700"
            opacity="0.6"
          />
        </g>
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
