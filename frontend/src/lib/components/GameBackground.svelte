<script lang="ts">
  import { currentTheme } from '$lib/theme/themeStore';
  import { onMount, onDestroy } from 'svelte';
  
  interface FloatingStone {
    x: number;
    y: number;
    z: number; // For depth/parallax
    size: number;
    targetSize: number; // Final size to grow to
    growthRate: number; // How fast it grows
    speedX: number;
    speedY: number;
    rotation: number;
    rotationSpeed: number;
    color: string;
    opacity: number;
  }
  
  interface GridParticle {
    x: number;
    y: number;
    size: number;
    opacity: number;
    pulsePhase: number;
    pulseSpeed: number;
  }
  
  interface Wave {
    offset: number;
    speed: number;
    amplitude: number;
    frequency: number;
  }
  
  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null;
  let animationId: number;
  let stones: FloatingStone[] = [];
  let gridParticles: GridParticle[] = [];
  let waves: Wave[] = [];
  let time = 0;
  let spawnTimer = 0;
  const SPAWN_INTERVAL = 180; // Spawn a stone every 180 frames (~3 seconds at 60fps)
  const MAX_STONES = 25; // Don't go crazy with stones
  let nextStoneIsPlayer1 = true; // Alternate between player colors
  
  // Parse color from various formats to RGB
  function parseColor(color: string): { r: number; g: number; b: number } {
    // Handle hex colors
    if (color.startsWith('#')) {
      const hex = color.slice(1);
      if (hex.length === 6) {
        return {
          r: parseInt(hex.slice(0, 2), 16),
          g: parseInt(hex.slice(2, 4), 16),
          b: parseInt(hex.slice(4, 6), 16)
        };
      }
    }
    // Default fallback
    return { r: 255, g: 0, b: 255 };
  }
  
  function initStones(width: number, height: number) {
    stones = [];
    // Start with more stones for initial splatter effect
    const stoneCount = 30; // 1.5x more stones for bigger explosion
    
    const centerX = width / 2;
    const centerY = height / 2;
    
    for (let i = 0; i < stoneCount; i++) {
      spawnStone(width, height, true); // Pass true for initial burst
    }
  }
  
  function spawnStone(width: number, height: number, isInitial: boolean = false) {
    if (stones.length >= MAX_STONES) return;
    
    const centerX = width / 2;
    const centerY = height / 2;
    const isPlayer1 = nextStoneIsPlayer1;
    nextStoneIsPlayer1 = !nextStoneIsPlayer1; // Toggle for next spawn
    
    // Spawn stones from center with outward velocity
    const angle = Math.random() * Math.PI * 2;
    const spawnDistance = Math.random() * 80 + 20; // Spawn close to center
    const startX = centerX + Math.cos(angle) * spawnDistance;
    const startY = centerY + Math.sin(angle) * spawnDistance;
    
    // Outward velocity from center - much faster for initial burst
    const speed = isInitial 
      ? Math.random() * 8.0 + 5.0  // EXPLOSIVE initial burst (5.0-13.0) KABOOM!
      : Math.random() * 0.5 + 0.3;  // Normal slow speed (0.3-0.8)
    const speedX = Math.cos(angle) * speed;
    const speedY = Math.sin(angle) * speed;
    
    const targetSize = Math.random() * 50 + 40;
    stones.push({
      x: startX,
      y: startY,
      z: Math.random() * 0.6 + 0.4,
      size: 1, // Start tiny
      targetSize: targetSize,
      growthRate: targetSize * 0.015, // Grow gradually to target size
      speedX: speedX,
      speedY: speedY,
      rotation: Math.random() * Math.PI * 2,
      rotationSpeed: (Math.random() - 0.5) * 0.02,
      color: isPlayer1 ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2,
      opacity: Math.random() * 0.5 + 0.3
    });
  }
  
  function initGridParticles(width: number, height: number) {
    gridParticles = [];
    const spacing = 100;
    
    // Only add particles in the upper 60% (sky area)
    for (let x = 0; x < width; x += spacing) {
      for (let y = 0; y < height * 0.6; y += spacing) {
        if (Math.random() > 0.7) { // Stars/particles in the sky
          gridParticles.push({
            x: x + (Math.random() - 0.5) * spacing * 0.5,
            y: y + (Math.random() - 0.5) * spacing * 0.5,
            size: Math.random() * 2 + 0.5,
            opacity: 0,
            pulsePhase: Math.random() * Math.PI * 2,
            pulseSpeed: Math.random() * 0.02 + 0.005
          });
        }
      }
    }
  }
  
  function initWaves() {
    waves = [
      { offset: 0, speed: 0.0008, amplitude: 15, frequency: 0.008 },
      { offset: Math.PI, speed: 0.001, amplitude: 10, frequency: 0.01 }
    ]; // Reduced to 2 waves for cleaner look
  }
  
  function drawStone(stone: FloatingStone) {
    if (!ctx) return;
    
    const scaledSize = stone.size * stone.z;
    const alpha = stone.opacity * stone.z;
    
    ctx.save();
    ctx.translate(stone.x, stone.y);
    ctx.rotate(stone.rotation);
    
    // Stronger outer glow for prominence
    ctx.shadowBlur = 30 * stone.z;
    ctx.shadowColor = stone.color;
    
    // Create gradient for 3D effect with more contrast
    const gradient = ctx.createRadialGradient(
      -scaledSize * 0.3,
      -scaledSize * 0.3,
      0,
      0,
      0,
      scaledSize
    );
    
    const colorRGB = parseColor(stone.color);
    gradient.addColorStop(0, `rgba(255, 255, 255, ${alpha})`); // Brighter highlight
    gradient.addColorStop(0.2, `rgba(${colorRGB.r}, ${colorRGB.g}, ${colorRGB.b}, ${alpha})`);
    gradient.addColorStop(0.7, `rgba(${colorRGB.r * 0.7}, ${colorRGB.g * 0.7}, ${colorRGB.b * 0.7}, ${alpha})`);
    gradient.addColorStop(1, `rgba(${colorRGB.r * 0.2}, ${colorRGB.g * 0.2}, ${colorRGB.b * 0.2}, ${alpha * 0.6})`);
    
    // Draw stone with border for definition
    ctx.beginPath();
    ctx.arc(0, 0, scaledSize, 0, Math.PI * 2);
    ctx.fillStyle = gradient;
    ctx.fill();
    
    // Add border for better distinction
    ctx.strokeStyle = `rgba(${colorRGB.r * 0.5}, ${colorRGB.g * 0.5}, ${colorRGB.b * 0.5}, ${alpha * 0.8})`;
    ctx.lineWidth = 2;
    ctx.stroke();
    
    ctx.restore();
  }
  
  function drawGridParticle(particle: GridParticle) {
    if (!ctx) return;
    
    const accentRGB = parseColor($currentTheme.accent);
    ctx.beginPath();
    ctx.arc(particle.x, particle.y, particle.size, 0, Math.PI * 2);
    ctx.fillStyle = `rgba(${accentRGB.r}, ${accentRGB.g}, ${accentRGB.b}, ${particle.opacity})`;
    ctx.fill();
    
    // Glow effect
    if (particle.opacity > 0.5) {
      ctx.shadowBlur = 10;
      ctx.shadowColor = $currentTheme.accent;
      ctx.fill();
      ctx.shadowBlur = 0;
    }
  }
  
  function drawWaves(width: number, height: number) {
    if (!ctx) return;
    
    ctx.save();
    ctx.globalAlpha = 0.08; // More subtle waves
    
    waves.forEach((wave, index) => {
      ctx!.beginPath();
      ctx!.moveTo(0, height);
      
      for (let x = 0; x <= width; x += 5) {
        const y = height * 0.5 + 
                  Math.sin(x * wave.frequency + wave.offset) * wave.amplitude +
                  Math.sin(time * 0.001 + x * 0.005) * 10;
        ctx!.lineTo(x, y);
      }
      
      ctx!.lineTo(width, height);
      ctx!.closePath();
      
      const gradient = ctx!.createLinearGradient(0, height * 0.3, 0, height);
      const primaryRGB = parseColor($currentTheme.primary);
      const secondaryRGB = parseColor($currentTheme.secondary);
      
      if (index % 2 === 0) {
        gradient.addColorStop(0, `rgba(${primaryRGB.r}, ${primaryRGB.g}, ${primaryRGB.b}, 0.3)`);
        gradient.addColorStop(1, `rgba(${primaryRGB.r}, ${primaryRGB.g}, ${primaryRGB.b}, 0)`);
      } else {
        gradient.addColorStop(0, `rgba(${secondaryRGB.r}, ${secondaryRGB.g}, ${secondaryRGB.b}, 0.3)`);
        gradient.addColorStop(1, `rgba(${secondaryRGB.r}, ${secondaryRGB.g}, ${secondaryRGB.b}, 0)`);
      }
      
      ctx!.fillStyle = gradient;
      ctx!.fill();
    });
    
    ctx.restore();
  }
  
  function drawGridLines(width: number, height: number) {
    if (!ctx) return;
    
    ctx.save();
    
    // Draw perspective grid at bottom (classic synthwave style)
    const horizonY = height * 0.65; // Horizon line
    const gridSpacing = 40;
    const gridDepth = 15; // Number of horizontal lines
    
    ctx.strokeStyle = `rgba(${parseColor($currentTheme.primary).r}, ${parseColor($currentTheme.primary).g}, ${parseColor($currentTheme.primary).b}, 0.15)`;
    ctx.lineWidth = 1.5;
    
    // Draw perspective horizontal lines
    for (let i = 0; i < gridDepth; i++) {
      const progress = i / gridDepth;
      const y = horizonY + progress * (height - horizonY);
      const perspective = 1 - progress * 0.7; // Perspective scaling
      
      ctx.globalAlpha = 0.3 - progress * 0.2; // Fade with distance
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }
    
    // Draw perspective vertical lines
    const verticalLines = 12;
    for (let i = 0; i <= verticalLines; i++) {
      const x = (width / verticalLines) * i;
      const vanishingX = width / 2;
      
      ctx.globalAlpha = 0.25;
      ctx.beginPath();
      ctx.moveTo(x, horizonY);
      
      // Calculate vanishing point convergence
      const targetX = vanishingX + (x - vanishingX) * 0.3;
      ctx.lineTo(targetX, height);
      ctx.stroke();
    }
    
    ctx.restore();
  }
  
  function drawSynthwaveSun(width: number, height: number) {
    if (!ctx) return;
    
    ctx.save();
    const sunX = width / 2;
    const sunY = height * 0.35;
    const sunRadius = Math.min(width, height) * 0.15;
    
    // Sun gradient
    const sunGradient = ctx.createRadialGradient(sunX, sunY, 0, sunX, sunY, sunRadius);
    const primaryRGB = parseColor($currentTheme.primary);
    const secondaryRGB = parseColor($currentTheme.secondary);
    
    sunGradient.addColorStop(0, `rgba(${secondaryRGB.r}, ${secondaryRGB.g}, ${secondaryRGB.b}, 0.3)`);
    sunGradient.addColorStop(0.5, `rgba(${primaryRGB.r}, ${primaryRGB.g}, ${primaryRGB.b}, 0.15)`);
    sunGradient.addColorStop(1, 'rgba(0, 0, 0, 0)');
    
    ctx.fillStyle = sunGradient;
    ctx.beginPath();
    ctx.arc(sunX, sunY, sunRadius, 0, Math.PI * 2);
    ctx.fill();
    
    // Sun horizontal lines (synthwave style)
    ctx.strokeStyle = `rgba(${primaryRGB.r}, ${primaryRGB.g}, ${primaryRGB.b}, 0.4)`;
    ctx.lineWidth = 2;
    const lineCount = 8;
    for (let i = 0; i < lineCount; i++) {
      const y = sunY - sunRadius + (sunRadius * 2 / lineCount) * i;
      const lineWidth = Math.sqrt(sunRadius * sunRadius - Math.pow(y - sunY, 2)) * 2;
      
      ctx.beginPath();
      ctx.moveTo(sunX - lineWidth / 2, y);
      ctx.lineTo(sunX + lineWidth / 2, y);
      ctx.stroke();
    }
    
    ctx.restore();
  }
  
  function updateStones(width: number, height: number) {
    const centerX = width / 2;
    const centerY = height / 2;
    const maxDistance = Math.sqrt(width * width + height * height) / 2 + 200;
    
    // Update positions
    stones.forEach(stone => {
      stone.x += stone.speedX * stone.z;
      stone.y += stone.speedY * stone.z;
      stone.rotation += stone.rotationSpeed;
      
      // Grow stone gradually to target size
      if (stone.size < stone.targetSize) {
        stone.size = Math.min(stone.size + stone.growthRate, stone.targetSize);
      }
    });
    
    // Remove stones that went off-screen
    stones = stones.filter(stone => {
      const dx = stone.x - centerX;
      const dy = stone.y - centerY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      return distance <= maxDistance;
    });
    
    // Spawn new stones periodically
    spawnTimer++;
    if (spawnTimer >= SPAWN_INTERVAL) {
      spawnStone(width, height);
      spawnTimer = 0;
    }
  }
  
  function updateGridParticles() {
    gridParticles.forEach(particle => {
      particle.pulsePhase += particle.pulseSpeed;
      particle.opacity = (Math.sin(particle.pulsePhase) + 1) * 0.3;
    });
  }
  
  function updateWaves() {
    waves.forEach(wave => {
      wave.offset += wave.speed;
    });
  }
  
  function animate() {
    if (!ctx || !canvas) return;
    
    const width = canvas.width;
    const height = canvas.height;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    // Draw synthwave background gradient (darker at top, lighter at horizon)
    const bgGradient = ctx.createLinearGradient(0, 0, 0, height);
    const bgRGB = parseColor($currentTheme.background);
    const primaryRGB = parseColor($currentTheme.primary);
    
    bgGradient.addColorStop(0, $currentTheme.background);
    bgGradient.addColorStop(0.5, `rgba(${bgRGB.r}, ${bgRGB.g}, ${bgRGB.b * 1.3}, 1)`);
    bgGradient.addColorStop(0.65, `rgba(${primaryRGB.r * 0.15}, ${primaryRGB.g * 0.15}, ${primaryRGB.b * 0.2}, 1)`);
    bgGradient.addColorStop(1, `rgba(${primaryRGB.r * 0.2}, ${primaryRGB.g * 0.1}, ${primaryRGB.b * 0.3}, 1)`);
    ctx.fillStyle = bgGradient;
    ctx.fillRect(0, 0, width, height);
    
    // Draw perspective grid
    drawGridLines(width, height);
    
    // Draw subtle waves at horizon
    drawWaves(width, height);
    
    // Update and draw stones (back to front based on z)
    updateStones(width, height);
    stones
      .sort((a, b) => a.z - b.z)
      .forEach(stone => drawStone(stone));
    
    // Update and draw grid particles (stars in the sky)
    updateGridParticles();
    gridParticles.forEach(particle => drawGridParticle(particle));
    
    // Update waves
    updateWaves();
    
    time++;
    animationId = requestAnimationFrame(animate);
  }
  
  function resize() {
    if (!canvas) return;
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    initStones(canvas.width, canvas.height);
    initGridParticles(canvas.width, canvas.height);
  }
  
  onMount(() => {
    if (!canvas) return;
    
    ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    resize();
    initWaves();
    animate();
    
    window.addEventListener('resize', resize);
  });
  
  onDestroy(() => {
    if (animationId) {
      cancelAnimationFrame(animationId);
    }
    window.removeEventListener('resize', resize);
  });
  
  // React to theme changes
  $effect(() => {
    if (stones.length > 0 && $currentTheme) {
      stones.forEach(stone => {
        const isPlayer1 = Math.random() > 0.5;
        stone.color = isPlayer1 ? $currentTheme.stonePlayer1 : $currentTheme.stonePlayer2;
      });
    }
  });
</script>

<canvas 
  bind:this={canvas}
  class="fixed inset-0 w-full h-full pointer-events-none"
  style="z-index: 0;"
></canvas>

<style>
  canvas {
    image-rendering: auto;
    image-rendering: crisp-edges;
    image-rendering: pixelated;
  }
</style>
