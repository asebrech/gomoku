<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    size?: number;
    board?: Array<Array<'black' | 'white' | null>>;
    onCellClick?: (row: number, col: number) => void;
  }
  
  let { 
    board = Array(19).fill(null).map(() => Array(19).fill(null)),
    size = board.length || 19, 
    onCellClick 
  }: Props = $props();
  
  // Reactive size based on board
  const actualSize = $derived(board.length || size);
  const cellSize = 30; // px
  const boardSize = $derived(cellSize * (actualSize - 1));
  const padding = 40;
  const totalSize = $derived(boardSize + (padding * 2));
  
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
</script>

<div class="flex items-center justify-center w-full p-8">
  <div class="rounded-lg p-8" style="background: {$currentTheme.background}E6; border: 3px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};">
    <svg 
      width={totalSize} 
      height={totalSize}
      class="cursor-pointer"
      onclick={handleClick}
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
        fill="{$currentTheme.background}CC"
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
              fill={stone === 'black' ? 'url(#blackGradient)' : 'url(#whiteGradient)'}
              filter="url(#stoneShadow)"
              class="animate-fade-in"
            />
          {/if}
        {/each}
      {/each}
      
      <!-- Gradients for stones -->
      <defs>
        <radialGradient id="blackGradient">
          <stop offset="30%" stop-color="#555" />
          <stop offset="100%" stop-color="#000" />
        </radialGradient>
        <radialGradient id="whiteGradient">
          <stop offset="30%" stop-color="#fff" />
          <stop offset="100%" stop-color="#ccc" />
        </radialGradient>
        <filter id="stoneShadow">
          <feDropShadow dx="2" dy="3" stdDeviation="2" flood-opacity="0.5"/>
        </filter>
      </defs>
    </svg>
  </div>
</div>
