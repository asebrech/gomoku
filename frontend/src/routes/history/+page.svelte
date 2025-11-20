<script lang="ts">
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { matchHistory, type MatchData } from '$lib/stores/matchHistory';
  import { currentTheme } from '$lib/theme/themeStore';
  import Button from '$lib/components/Button.svelte';
  import MatchCard from '$lib/components/MatchCard.svelte';
  import ReplayViewer from '$lib/components/ReplayViewer.svelte';
  import { writable } from 'svelte/store';
  
  const MATCHES_PER_PAGE = 9;
  
  let currentPage = $state(0);
  let filterStatus = $state<'all' | 'ongoing' | 'finished'>('all');
  let showDeleteConfirm = $state(false);
  let matchToDelete = $state<string | null>(null);
  let replayMatch = $state<MatchData | null>(null); // For replay mode
  
  // Create a store to track replay state that can be accessed globally
  export const isInReplayMode = writable(false);
  
  // Debug: Log when component mounts
  onMount(() => {
    // Force refresh from localStorage
    matchHistory.refresh();
    
    // Listen for custom event to exit replay mode
    const handleExitReplay = () => {
      exitReplay();
    };
    window.addEventListener('exitReplayMode', handleExitReplay);
    
    return () => {
      window.removeEventListener('exitReplayMode', handleExitReplay);
    };
  });
  
  // Filter matches based on status
  const filteredMatches = $derived(() => {
    if (filterStatus === 'all') return $matchHistory;
    return $matchHistory.filter(match => match.status === filterStatus);
  });
  
  // Paginate matches
  const totalPages = $derived(Math.ceil(filteredMatches().length / MATCHES_PER_PAGE));
  const paginatedMatches = $derived(() => {
    const start = currentPage * MATCHES_PER_PAGE;
    const end = start + MATCHES_PER_PAGE;
    const matches = filteredMatches().slice(start, end);
    return matches;
  });
  
  // Reset to first page when filter changes
  $effect(() => {
    filterStatus; // Track filter changes
    currentPage = 0;
  });
  
  function handleReplay(match: MatchData) {
    replayMatch = match;
    isInReplayMode.set(true);
  }
  
  function handleResume(match: MatchData) {
    // Store the match ID in sessionStorage so the game can load it
    sessionStorage.setItem('resumeMatchId', match.id);
    
    // Navigate to the appropriate game mode route
    let route = '/game';
    if (match.player1Type === 'human' && match.player2Type === 'ai') {
      route = '/game/player-vs-ai';
    } else if (match.player1Type === 'ai' && match.player2Type === 'human') {
      route = '/game/player-vs-ai';
    } else if (match.player1Type === 'human' && match.player2Type === 'human') {
      route = '/game/player-vs-player';
    } else if (match.player1Type === 'ai' && match.player2Type === 'ai') {
      route = '/game/ai-vs-ai';
    }
    
    goto(route);
  }
  
  function exitReplay() {
    replayMatch = null;
    isInReplayMode.set(false);
  }
  
  function confirmDelete(matchId: string) {
    matchToDelete = matchId;
    showDeleteConfirm = true;
  }
  
  function handleDelete() {
    if (matchToDelete) {
      matchHistory.deleteMatch(matchToDelete);
      matchToDelete = null;
      showDeleteConfirm = false;
      
      // Adjust current page if we deleted the last item on the page
      if (paginatedMatches().length === 0 && currentPage > 0) {
        currentPage--;
      }
    }
  }
  
  function cancelDelete() {
    matchToDelete = null;
    showDeleteConfirm = false;
  }
  
  function handleClearAll() {
    if (confirm($_('history.clearAllConfirm'))) {
      matchHistory.clearAll();
      currentPage = 0;
    }
  }
  
  function goToPage(page: number) {
    currentPage = page;
  }
  
  function nextPage() {
    if (currentPage < totalPages - 1) {
      currentPage++;
    }
  }
  
  function prevPage() {
    if (currentPage > 0) {
      currentPage--;
    }
  }
</script>

{#if replayMatch}
  <!-- Replay Mode -->
  <ReplayViewer match={replayMatch} onBack={exitReplay} />
{:else}
  <!-- History List Mode -->
<div class="h-full w-full flex flex-col overflow-hidden">
  <div class="flex-1 overflow-y-auto">
    <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-6 sm:py-8">
      <!-- Header -->
      <div class="mb-6 sm:mb-8">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-4">
          <h1 
            class="text-3xl sm:text-4xl font-bold text-white"
          >
            {$_('history.title')}
          </h1>
          <div class="flex gap-2">
            <Button variant="secondary" size="sm" onclick={() => goto('/')}>
              <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
              </svg>
              {$_('history.back')}
            </Button>
            {#if $matchHistory.length > 0}
              <Button variant="ghost" size="sm" onclick={handleClearAll}>
                <svg class="w-4 h-4 inline-block mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
                {$_('history.clearAll')}
              </Button>
            {/if}
          </div>
        </div>
        
        <!-- Filter Tabs -->
        <div class="flex gap-2 flex-wrap">
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all duration-200"
            style="
              background: {filterStatus === 'all' ? $currentTheme.primary : `${$currentTheme.surface}20`};
              color: #FFFFFF;
              border: 1px solid {filterStatus === 'all' ? $currentTheme.primary : `${$currentTheme.primary}40`};
            "
            onclick={() => filterStatus = 'all'}
          >
            {$_('history.filter.all')} ({$matchHistory.length})
          </button>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all duration-200"
            style="
              background: {filterStatus === 'ongoing' ? $currentTheme.primary : `${$currentTheme.surface}20`};
              color: #FFFFFF;
              border: 1px solid {filterStatus === 'ongoing' ? $currentTheme.primary : `${$currentTheme.primary}40`};
            "
            onclick={() => filterStatus = 'ongoing'}
          >
            {$_('history.filter.ongoing')} ({$matchHistory.filter(m => m.status === 'ongoing').length})
          </button>
          <button
            class="px-4 py-2 rounded-lg font-medium transition-all duration-200"
            style="
              background: {filterStatus === 'finished' ? $currentTheme.primary : `${$currentTheme.surface}20`};
              color: #FFFFFF;
              border: 1px solid {filterStatus === 'finished' ? $currentTheme.primary : `${$currentTheme.primary}40`};
            "
            onclick={() => filterStatus = 'finished'}
          >
            {$_('history.filter.finished')} ({$matchHistory.filter(m => m.status === 'finished').length})
          </button>
        </div>
      </div>
      
      <!-- Match Grid -->
      {#if paginatedMatches().length > 0}
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
          {#each paginatedMatches() as match (match.id)}
            <MatchCard
              {match}
              onReplay={() => handleReplay(match)}
              onResume={() => handleResume(match)}
              onDelete={() => confirmDelete(match.id)}
            />
          {/each}
        </div>
        
        <!-- Pagination -->
        {#if totalPages > 1}
          <div class="flex items-center justify-center gap-2 flex-wrap">
            <Button
              variant="secondary"
              size="sm"
              onclick={prevPage}
              disabled={currentPage === 0}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 19l-7-7 7-7" />
              </svg>
            </Button>
            
            <div class="flex gap-1">
              {#each Array(totalPages) as _, i}
                {#if totalPages <= 7 || i === 0 || i === totalPages - 1 || (i >= currentPage - 1 && i <= currentPage + 1)}
                  <button
                    class="w-10 h-10 rounded-lg font-medium transition-all duration-200"
                    style="
                      background: {i === currentPage ? $currentTheme.primary : `${$currentTheme.surface}20`};
                      color: #FFFFFF;
                      border: 1px solid {i === currentPage ? $currentTheme.primary : `${$currentTheme.primary}40`};
                    "
                    onclick={() => goToPage(i)}
                  >
                    {i + 1}
                  </button>
                {:else if i === currentPage - 2 || i === currentPage + 2}
                  <span class="w-10 h-10 flex items-center justify-center text-white">
                    ...
                  </span>
                {/if}
              {/each}
            </div>
            
            <Button
              variant="secondary"
              size="sm"
              onclick={nextPage}
              disabled={currentPage === totalPages - 1}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
            </Button>
          </div>
        {/if}
      {:else}
        <!-- Empty State -->
        <div class="flex flex-col items-center justify-center py-16 px-4">
          <svg 
            class="w-24 h-24 mb-4 opacity-30 text-white" 
            fill="none" 
            stroke="currentColor" 
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <h2 
            class="text-2xl font-bold mb-2 text-white"
          >
            {$_('history.empty.title')}
          </h2>
          <p 
            class="text-center mb-6 opacity-70 text-white"
          >
            {#if filterStatus === 'all'}
              {$_('history.empty.description')}
            {:else}
              {$_('history.empty.descriptionFiltered', { values: { filter: filterStatus } })}
            {/if}
          </p>
          <Button variant="primary" size="md" onclick={() => goto('/game')}>
            {$_('history.empty.startPlaying')}
          </Button>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm}
  <div 
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    style="background: rgba(0, 0, 0, 0.7);"
    onclick={cancelDelete}
  >
    <div 
      class="rounded-lg p-6 max-w-sm w-full backdrop-blur-lg border animate-scale-in"
      style="
        background: {$currentTheme.background}E6;
        border-color: {$currentTheme.primary}60;
        box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3);
      "
      onclick={(e) => e.stopPropagation()}
    >
      <h3 
        class="text-xl font-bold mb-4 text-white"
      >
        {$_('history.deleteConfirm.title')}
      </h3>
      <p 
        class="mb-6 opacity-80 text-white"
      >
        {$_('history.deleteConfirm.message')}
      </p>
      <div class="flex gap-3">
        <Button variant="secondary" size="md" fullWidth={true} onclick={cancelDelete}>
          {$_('history.deleteConfirm.cancel')}
        </Button>
        <Button variant="primary" size="md" fullWidth={true} onclick={handleDelete}>
          {$_('history.deleteConfirm.delete')}
        </Button>
      </div>
    </div>
  </div>
{/if}
{/if}

<style>
  @keyframes scale-in {
    from {
      transform: scale(0.9);
      opacity: 0;
    }
    to {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .animate-scale-in {
    animation: scale-in 0.2s ease-out;
  }
</style>
