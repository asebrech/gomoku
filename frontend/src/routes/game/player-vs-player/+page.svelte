<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
  
  let resumeMatchId = $state<string | undefined>(undefined);
  
  onMount(() => {
    // Check if we're resuming a match
    const storedResumeId = sessionStorage.getItem('resumeMatchId');
    if (storedResumeId) {
      resumeMatchId = storedResumeId;
    }
  });
</script>

<div class="h-full w-full">
  <div class="h-full w-full px-4">
    <Game 
      player1Type="human"
      player2Type="human"
      player1Name="Player 1"
      player2Name="Player 2"
      autoStart={!resumeMatchId}
      {resumeMatchId}
      onBack={() => goto('/game')}
    />
  </div>
</div>
