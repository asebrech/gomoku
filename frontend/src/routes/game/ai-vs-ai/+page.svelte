<script lang="ts">
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <Game 
      player1Type="ai"
      player2Type="ai"
      player1Name="AI 1"
      player2Name="AI 2"
      aiDepth={$gameSettings.aiDepth}
      autoStart={true}
      moveDelay={500}
      onBack={() => goto('/game')}
    />
  </div>
</div>
