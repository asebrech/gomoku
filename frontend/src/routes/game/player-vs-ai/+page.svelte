<script lang="ts">
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
</script>

<div class="min-h-[calc(100vh-4rem)] py-8">
  <div class="max-w-6xl mx-auto px-4">
    <div class="text-center mb-6">
      <h1 class="text-4xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 via-pink-500 to-purple-600 mb-2">
        Player vs AI
      </h1>
    </div>
    
    <Game 
      player1Type="human"
      player2Type="ai"
      player1Name="You (Black)"
      player2Name="AI (White)"
      autoStart={true}
      onBack={() => goto('/game')}
    />
  </div>
</div>
