<script lang="ts">
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
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
