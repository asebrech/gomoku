<script lang="ts">
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <Game 
      player1Type="human"
      player2Type="ai"
      player1Name="You"
      player2Name="AI"
      aiDepth={$gameSettings.aiDepth}
      autoStart={true}
      onBack={() => goto('/game')}
    />
  </div>
</div>
