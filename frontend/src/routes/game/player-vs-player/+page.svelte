<script lang="ts">
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
</script>

<div class="h-full w-full">
  <div class="h-full w-full px-4">
    <Game 
      player1Type="human"
      player2Type="human"
      player1Name="Player 1"
      player2Name="Player 2"
      autoStart={true}
      onBack={() => goto('/game')}
    />
  </div>
</div>
