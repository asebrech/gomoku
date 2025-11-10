<script lang="ts">
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';
	import { initI18n } from '$lib/i18n/i18n';
	import { isLoading } from 'svelte-i18n';
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
				await wasmModule.initThreadPool(numThreads);
				
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
	<div class="relative min-h-screen">
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
		<div class="fixed inset-0 bg-gradient-to-br from-black/60 to-indigo-900/70 -z-10"></div>
		
		<!-- Persistent navbar -->
		<nav class="fixed top-0 left-0 right-0 z-50 backdrop-blur-md border-b"
		     style="background-color: rgba(17, 24, 39, 0.7); border-color: rgba(139, 92, 246, 0.2);">
			<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
				<div class="flex items-center justify-between h-16">
					<div class="flex items-center gap-3">
						<a href="/" class="flex items-center hover:opacity-80 transition-opacity duration-200">
							<img src={logo} alt="Gomoku" class="h-10 w-auto drop-shadow-lg" />
						</a>
					</div>
					
					<div class="flex items-center gap-4">
						<ThemeSwitcher />
						<LanguageSwitcher />
					</div>
				</div>
			</div>
		</nav>
		
		<!-- Main content with padding for navbar -->
		<main class="pt-16 min-h-screen">
			{@render children?.()}
		</main>
		
		<!-- Background Music Player -->
		<BackgroundMusic />
	</div>
{/if}
