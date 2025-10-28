<script lang="ts">
  import type { Snippet } from 'svelte';
  import { currentTheme } from '$lib/theme/themeStore';
  
  interface Props {
    variant?: 'primary' | 'secondary' | 'ghost';
    size?: 'sm' | 'md' | 'lg';
    onclick?: () => void;
    disabled?: boolean;
    fullWidth?: boolean;
    children: Snippet;
  }
  
  let { 
    variant = 'primary', 
    size = 'md',
    onclick,
    disabled = false,
    fullWidth = false,
    children 
  }: Props = $props();
  
  const baseClasses = 'font-semibold rounded-lg transition-all duration-200 disabled:opacity-50 disabled:cursor-not-allowed';
  
  const sizeClasses = {
    sm: 'px-4 py-2 text-sm',
    md: 'px-6 py-3 text-base',
    lg: 'px-8 py-4 text-lg'
  };
  
  // Dynamic styles based on variant
  const getVariantStyles = (variant: string) => {
    switch (variant) {
      case 'primary':
        return {
          background: $currentTheme.gradientPrimary,
          color: '#FFFFFF',
          boxShadow: $currentTheme.glowPrimary,
          border: 'none'
        };
      case 'secondary':
        return {
          background: `${$currentTheme.surface}1A`, // 10% opacity
          color: '#FFFFFF',
          border: `1px solid ${$currentTheme.primary}33`, // 20% opacity
          backdropFilter: 'blur(8px)'
        };
      case 'ghost':
        return {
          background: 'transparent',
          color: '#FFFFFF',
          border: 'none'
        };
      default:
        return {};
    }
  };
  
  const getHoverStyles = (variant: string) => {
    switch (variant) {
      case 'primary':
        return {
          transform: 'translateY(-2px)',
          boxShadow: $currentTheme.glowAccent
        };
      case 'secondary':
        return {
          background: `${$currentTheme.surface}33` // 20% opacity
        };
      case 'ghost':
        return {
          background: `${$currentTheme.surface}1A` // 10% opacity
        };
      default:
        return {};
    }
  };
  
  let isHovered = $state(false);
  
  const buttonStyles = $derived({
    ...getVariantStyles(variant),
    ...(isHovered && !disabled ? getHoverStyles(variant) : {})
  });
  
  const styleString = $derived(
    Object.entries(buttonStyles)
      .map(([key, value]) => `${key.replace(/([A-Z])/g, '-$1').toLowerCase()}: ${value}`)
      .join('; ')
  );
</script>

<button 
  class="{baseClasses} {sizeClasses[size]} {fullWidth ? 'w-full' : ''}"
  style={styleString}
  {onclick}
  {disabled}
  onmouseenter={() => isHovered = true}
  onmouseleave={() => isHovered = false}
>
  {@render children()}
</button>
