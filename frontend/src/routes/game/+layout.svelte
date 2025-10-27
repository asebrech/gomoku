<script lang="ts">
	import ingameVideo from '$lib/assets/backgrounds/ingame-background/in-game.webm';
	
	let { children } = $props();
</script>

<div class="relative min-h-[calc(100vh-4rem)]">
	<!-- In-game background video -->
	<video 
		autoplay 
		muted 
		loop 
		playsinline
		class="fixed inset-0 w-full h-full object-cover -z-10 top-16"
	>
		<source src={ingameVideo} type="video/webm" />
	</video>
	
	<!-- Game content -->
	<div class="relative z-10">
		{@render children?.()}
	</div>
</div>
