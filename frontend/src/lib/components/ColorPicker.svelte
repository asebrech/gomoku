<script lang="ts">
  interface Props {
    label: string;
    value: string;
    onchange?: (value: string) => void;
    description?: string;
  }
  
  let { label, value = $bindable('#000000'), onchange, description }: Props = $props();
  
  let inputValue = $state(value);
  
  // Update input when prop changes
  $effect(() => {
    inputValue = value;
  });
  
  function handleColorChange(e: Event) {
    const target = e.target as HTMLInputElement;
    inputValue = target.value;
    value = target.value;
    onchange?.(target.value);
  }
  
  function handleTextChange(e: Event) {
    const target = e.target as HTMLInputElement;
    let newValue = target.value;
    
    // Add # if not present
    if (!newValue.startsWith('#')) {
      newValue = '#' + newValue;
    }
    
    // Validate hex color
    const hexRegex = /^#[0-9A-Fa-f]{6}$/;
    if (hexRegex.test(newValue)) {
      inputValue = newValue;
      value = newValue;
      onchange?.(newValue);
    } else {
      // Revert to previous valid value
      target.value = inputValue;
    }
  }
</script>

<div class="color-picker-wrapper">
  <div class="flex items-center justify-between mb-2">
    <span class="text-white/90 font-medium text-sm">
      {label}
    </span>
    {#if description}
      <span class="text-white/50 text-xs italic">{description}</span>
    {/if}
  </div>
  
  <div class="color-picker-container">
    <div class="color-preview-wrapper">
      <input
        type="color"
        value={inputValue}
        oninput={handleColorChange}
        class="color-input"
        aria-label={`${label} color picker`}
      />
      <div 
        class="color-preview" 
        style="background-color: {inputValue}"
      ></div>
    </div>
    
    <input
      type="text"
      value={inputValue}
      oninput={handleTextChange}
      placeholder="#000000"
      class="hex-input"
      maxlength="7"
      aria-label={`${label} hex value`}
    />
  </div>
</div>

<style>
  .color-picker-wrapper {
    width: 100%;
  }
  
  .color-picker-container {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }
  
  .color-preview-wrapper {
    position: relative;
    width: 3rem;
    height: 2.5rem;
  }
  
  .color-input {
    position: absolute;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
  }
  
  .color-preview {
    width: 100%;
    height: 100%;
    border-radius: 0.5rem;
    border: 2px solid rgba(255, 255, 255, 0.3);
    cursor: pointer;
    transition: all 0.2s ease;
  }
  
  .color-preview:hover {
    border-color: rgba(255, 255, 255, 0.6);
    transform: scale(1.05);
  }
  
  .hex-input {
    flex: 1;
    padding: 0.625rem 1rem;
    background: rgba(255, 255, 255, 0.1);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 0.5rem;
    color: white;
    font-family: 'Courier New', monospace;
    font-size: 0.875rem;
    text-transform: uppercase;
    transition: all 0.2s ease;
  }
  
  .hex-input:focus {
    outline: none;
    background: rgba(255, 255, 255, 0.15);
    border-color: rgba(255, 255, 255, 0.4);
  }
  
  .hex-input::placeholder {
    color: rgba(255, 255, 255, 0.3);
  }
</style>
