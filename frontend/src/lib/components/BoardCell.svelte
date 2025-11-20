<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    row: number;
    col: number;
    stone?: 'black' | 'white' | null;
    onclick?: () => void;
  }
  
  let { row, col, stone = null, onclick }: Props = $props();
  
  // Star points for 19x19 board (at intersections 3, 9, 15)
  const isStarPoint = (row === 3 || row === 9 || row === 15) && 
                      (col === 3 || col === 9 || col === 15);
</script>

<div class="relative w-full" style="padding-bottom: 100%;">
  <!-- Intersection point (clickable) -->
  <button
    class="absolute inset-0 flex items-center justify-center group cursor-pointer z-20
           transition-all duration-150 rounded-full"
    onclick={onclick}
    disabled={stone !== null}
    aria-label="Intersection {row},{col}"
  >
    <!-- Star point -->
    {#if isStarPoint && !stone}
      <div 
        class="absolute w-2.5 h-2.5 rounded-full"
        style="background-color: {$currentTheme.textPrimary};"
      ></div>
    {/if}
    
    <!-- Hover indicator -->
    {#if !stone}
      <div 
        class="absolute w-8 h-8 rounded-full opacity-0 group-hover:opacity-60 
               transition-opacity duration-200"
        style="background: {$currentTheme.gradientAccent};"
      ></div>
    {/if}
    
    <!-- Stone -->
    {#if stone}
      <div 
        class="absolute w-9 h-9 rounded-full animate-fade-in"
        style="background: {stone === 'black' 
          ? 'radial-gradient(circle at 40% 40%, #555, #000)' 
          : 'radial-gradient(circle at 40% 40%, #fff, #ccc)'};
          box-shadow: {stone === 'black' 
            ? '0 4px 12px rgba(0, 0, 0, 0.9), inset -3px -3px 8px rgba(0, 0, 0, 0.6)' 
            : '0 4px 12px rgba(0, 0, 0, 0.5), inset -3px -3px 8px rgba(0, 0, 0, 0.2)'};"
      ></div>
    {/if}
  </button>
  
  <!-- Grid lines (non-interactive background) -->
  <div class="absolute inset-0 pointer-events-none flex items-center justify-center">
    <!-- Horizontal line -->
    <div 
      class="absolute h-[2px] w-full left-0"
      style="background-color: {$currentTheme.primary}A0;"
    ></div>
    
    <!-- Vertical line -->
    <div 
      class="absolute w-[2px] h-full top-0"
      style="background-color: {$currentTheme.primary}A0;"
    ></div>
  </div>
</div>
