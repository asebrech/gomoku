<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Game from '$lib/components/Game.svelte';
  import { gameSettings } from '$lib/stores/gameSettings';
  
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
      player1Type="ai"
      player2Type="ai"
      player1Name="AI 1"
      player2Name="AI 2"
      aiDepth={$gameSettings.aiDepth}
      autoStart={!resumeMatchId}
      moveDelay={500}
      {resumeMatchId}
      onBack={() => goto('/game')}
    />
  </div>
</div>
