<script lang="ts">
	import GameBackground from '$lib/components/GameBackground.svelte';
	
	let { children } = $props();
</script>

<div class="h-full w-full relative">
	<!-- Dynamic synthwave background -->
	<GameBackground />
	
	<!-- Game content -->
	<div class="relative z-10 h-full w-full">
		{@render children?.()}
	</div>
</div>
