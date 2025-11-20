<script lang="ts">
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { initI18n } from '$lib/i18n/i18n';
	import { isLoading, _ } from 'svelte-i18n';
	import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
	import ThemeSwitcher from '$lib/components/ThemeSwitcher.svelte';
	import BackgroundMusic from '$lib/components/BackgroundMusic.svelte';
	import dolphinVideo from '$lib/assets/backgrounds/dolphin/dolphin.webm';
	import logo from '$lib/assets/logo/gomoku-logo.png';
	import '../app.css';

	let { children } = $props();
	let ready = $state(false);
	let wasmReady = $state(false);

	onMount(async () => {
		if (browser) {
			try {
				// Initialize i18n
				await initI18n();
				
				// Initialize WASM module and thread pool
				const wasmModule = await import('$lib/wasm/pkg/gomoku');
				await wasmModule.default();
				
				// Initialize thread pool for parallel AI search
				// Use fewer threads for WASM to reduce coordination overhead
				const numThreads = Math.min(navigator.hardwareConcurrency || 4, 4); // Cap at 4 for WASM
				await wasmModule.initThreadPool(1);
				
				wasmReady = true;
			} catch (error) {
				console.error('Failed to initialize WASM:', error);
			}
		}
		ready = true;
	});
</script>

<svelte:head>
	<link rel="icon" type="image/x-icon" href="/favicon.ico" />
	<link rel="icon" type="image/png" href="/favicon.png" />
</svelte:head>

{#if !ready || $isLoading}
	<div class="flex items-center justify-center min-h-screen bg-gradient-to-br from-gray-900 via-purple-900 to-indigo-900">
		<div class="text-white text-xl">Loading...</div>
	</div>
{:else}
	<div class="fixed inset-0 flex flex-col overflow-hidden">
		<!-- Persistent dolphin background -->
		<video 
			autoplay 
			muted 
			loop 
			playsinline
			class="fixed inset-0 w-full h-full object-cover -z-10"
		>
			<source src={dolphinVideo} type="video/webm" />
		</video>
		
		<!-- Gradient overlay -->
		<div class="fixed inset-0 bg-gradient-to-br from-black/30 to-indigo-900/40 -z-10"></div>
		
		<!-- Persistent navbar -->
		<nav class="flex-shrink-0 z-50 backdrop-blur-md border-b"
		     style="background-color: rgba(17, 24, 39, 0.7); border-color: rgba(139, 92, 246, 0.3);">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div class="flex items-center justify-between h-16">
					<div class="flex items-center gap-6">
						<a href="/" class="flex items-center hover:opacity-80 transition-opacity duration-200">
							<img src={logo} alt="Gomoku" class="h-14 w-auto drop-shadow-lg" />
						</a>
						
						<!-- Play and History buttons aligned to left -->
						<div class="flex items-center gap-2">
							<a 
								href="/game" 
								class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-white/10 transition-all duration-200 text-white font-medium"
								title={$_('home.play')}
							>
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
								</svg>
								<span class="hidden sm:inline">{$_('home.play')}</span>
							</a>
							<a 
								href="/history" 
								class="flex items-center gap-2 px-4 py-2 rounded-lg hover:bg-white/10 transition-all duration-200 text-white font-medium"
								title={$_('history.title')}
								onclick={(e) => {
									// If we're already on history page and in replay mode, exit replay
									if (window.location.pathname === '/history') {
										e.preventDefault();
										window.dispatchEvent(new CustomEvent('exitReplayMode'));
									}
								}}
							>
								<svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
									<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
								</svg>
								<span class="hidden sm:inline">{$_('history.title')}</span>
							</a>
						</div>
					</div>
					
					<div class="flex items-center gap-4">
						<ThemeSwitcher />
						<LanguageSwitcher />
					</div>
				</div>
			</div>
		</nav>
		
		<!-- Main content area - flexbox takes remaining space -->
		<main class="flex-1 overflow-hidden">
			{@render children?.()}
		</main>
		
		<!-- Bottom Left: Settings Button -->
		<div class="fixed bottom-6 left-6 z-40">
			<a 
				href="/settings" 
				class="w-12 h-12 rounded-full backdrop-blur-md border flex items-center justify-center hover:scale-110 transition-all duration-200 shadow-lg"
				style="background-color: rgba(17, 24, 39, 0.7); border-color: rgba(139, 92, 246, 0.3);"
				title={$_('home.settings')}
			>
				<svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
				</svg>
			</a>
		</div>
		
		<!-- Background Music Player (positioned in bottom right) -->
		<BackgroundMusic />
	</div>
{/if}
