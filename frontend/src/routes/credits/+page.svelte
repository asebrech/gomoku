<script lang="ts">
  import { goto } from '$app/navigation';
  import Button from '$lib/components/Button.svelte';
  import { currentTheme } from '$lib/theme/themeStore';
  
  const authors = [
    {
      name: 'NotJustJoe',
      github: 'NotJustJoe',
      url: 'https://github.com/NotJustJoe',
      role: 'Developer'
    },
    {
      name: 'asebrech',
      github: 'asebrech',
      url: 'https://github.com/asebrech',
      role: 'Developer'
    }
  ];
  
  const specialThanks = {
    name: 'LaBoiteDeCarton',
    github: 'LaBoiteDeCarton',
    url: 'https://github.com/LaBoiteDeCarton',
    role: 'AI Tester & Gomoku Expert'
  };
  
  const repository = {
    name: 'gomoku',
    url: 'https://github.com/asebrech/gomoku'
  };
</script>

<div class="min-h-[calc(100vh-4rem)] py-12 px-4">
  <div class="max-w-4xl mx-auto">
    <!-- Title -->
    <div class="text-center mb-12">
      <h1 class="text-6xl font-black text-transparent bg-clip-text mb-4"
          style="background-image: {$currentTheme.gradientPrimary}; -webkit-background-clip: text; background-clip: text;">
        Credits
      </h1>
      <p class="text-xl" style="color: rgba(255, 255, 255, 0.8);">
        Made with passion and lots of coffee ☕
      </p>
    </div>
    
    <!-- Repository Info -->
    <div class="mb-12 text-center">
      <a 
        href={repository.url}
        target="_blank"
        rel="noopener noreferrer"
        class="inline-block px-8 py-4 rounded-xl transition-all duration-300 hover:scale-105"
        style="
          background: {$currentTheme.background}CC;
          border: 2px solid {$currentTheme.primary};
          box-shadow: {$currentTheme.glowPrimary};
        "
      >
        <div class="text-sm mb-1" style="color: rgba(255, 255, 255, 0.6);">View on GitHub</div>
        <div class="text-2xl font-bold" style="color: {$currentTheme.primary};">
          {repository.name}
        </div>
      </a>
    </div>
    
    <!-- Authors Section -->
    <div class="mb-12">
      <h2 
        class="text-3xl font-bold text-center mb-8"
        style="color: {$currentTheme.primary}; text-shadow: {$currentTheme.glowPrimary};"
      >
        Developers
      </h2>
      
      <div class="flex flex-wrap justify-center gap-6 max-w-4xl mx-auto">
        {#each authors as author}
          <a
            href={author.url}
            target="_blank"
            rel="noopener noreferrer"
            class="group block p-8 rounded-xl transition-all duration-300 hover:scale-105 hover:-translate-y-2"
            style="
              background: {$currentTheme.background}DD;
              border: 3px solid {$currentTheme.primary};
              box-shadow: 0 8px 32px {$currentTheme.primary}40;
              flex: 0 0 auto;
              width: 280px;
            "
          >
            <div class="text-center">
              <div style="width: 80px; height: 80px; margin: 0 auto 1rem; overflow: hidden; border-radius: 50%; border: 2px solid {$currentTheme.primary}; box-shadow: {$currentTheme.glowPrimary};">
                <img 
                  src="https://avatars.githubusercontent.com/{author.github}?size=160"
                  alt="{author.name}"
                  style="width: 100%; height: 100%; object-fit: cover; display: block;"
                />
              </div>
              <h3 class="text-2xl font-bold mb-2" style="color: {$currentTheme.primary};">
                {author.name}
              </h3>
              <p class="text-base mb-3" style="color: rgba(255, 255, 255, 0.7);">
                {author.role}
              </p>
              <p class="text-sm group-hover:underline" style="color: {$currentTheme.primary};">
                @{author.github}
              </p>
            </div>
          </a>
        {/each}
      </div>
    </div>
    
    <!-- Special Thanks Section -->
    <div class="mb-12">
      <h2 
        class="text-3xl font-bold text-center mb-8"
        style="color: {$currentTheme.accent}; text-shadow: 0 0 20px {$currentTheme.accent}80;"
      >
        Special Thanks
      </h2>
      
      <div class="max-w-md mx-auto">
        <a
          href={specialThanks.url}
          target="_blank"
          rel="noopener noreferrer"
          class="group block p-8 rounded-xl transition-all duration-300 hover:scale-105 hover:-translate-y-2"
          style="
            background: {$currentTheme.background}DD;
            border: 3px solid {$currentTheme.accent};
            box-shadow: 0 8px 32px {$currentTheme.accent}40;
          "
        >
          <div class="text-center">
            <div style="width: 80px; height: 80px; margin: 0 auto 1rem; overflow: hidden; border-radius: 50%; border: 2px solid {$currentTheme.accent}; box-shadow: {$currentTheme.glowAccent};">
              <img 
                src="https://avatars.githubusercontent.com/{specialThanks.github}?size=160"
                alt="{specialThanks.name}"
                style="width: 100%; height: 100%; object-fit: cover; display: block;"
              />
            </div>
            <h3 class="text-2xl font-bold mb-2" style="color: {$currentTheme.accent};">
              {specialThanks.name}
            </h3>
            <p class="text-base mb-3" style="color: rgba(255, 255, 255, 0.7);">
              {specialThanks.role}
            </p>
            <p class="text-sm italic mb-4" style="color: rgba(255, 255, 255, 0.6);">
              "For rigorously testing our AI and proving it can actually play Gomoku! 🎯"
            </p>
            <p class="text-sm group-hover:underline" style="color: {$currentTheme.accent};">
              @{specialThanks.github}
            </p>
          </div>
        </a>
      </div>
    </div>
    
    <!-- Tech Stack Section -->
    <div class="mb-12">
      <h2 
        class="text-2xl font-bold text-center mb-6"
        style="color: {$currentTheme.primary};"
      >
        Built With
      </h2>
      
      <div class="flex flex-wrap justify-center gap-4">
        <div class="px-6 py-3 rounded-lg transition-all duration-300 hover:scale-105" style="background: {$currentTheme.background}99; border: 2px solid {$currentTheme.secondary};">
          <span style="color: {$currentTheme.secondary};">🦀 Rust</span>
        </div>
        <div class="px-6 py-3 rounded-lg transition-all duration-300 hover:scale-105" style="background: {$currentTheme.background}99; border: 2px solid {$currentTheme.secondary};">
          <span style="color: {$currentTheme.secondary};">⚡ SvelteKit</span>
        </div>
        <div class="px-6 py-3 rounded-lg transition-all duration-300 hover:scale-105" style="background: {$currentTheme.background}99; border: 2px solid {$currentTheme.secondary};">
          <span style="color: {$currentTheme.secondary};">🎨 TypeScript</span>
        </div>
        <div class="px-6 py-3 rounded-lg transition-all duration-300 hover:scale-105" style="background: {$currentTheme.background}99; border: 2px solid {$currentTheme.secondary};">
          <span style="color: {$currentTheme.secondary};">🌐 WebAssembly</span>
        </div>
      </div>
    </div>
    
    <!-- Footer -->
    <div class="text-center text-sm mb-8" style="color: rgba(255, 255, 255, 0.5);">
      <p>© 2025 Gomoku Project</p>
      <p class="mt-2">Made with ❤️ for the love of strategy games</p>
    </div>
    
    <!-- Back Button -->
    <div class="flex justify-center">
      <Button variant="primary" size="lg" onclick={() => goto('/')}>
        Back to Menu
      </Button>
    </div>
  </div>
</div>
