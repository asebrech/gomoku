# Gomoku Theme System

## Overview

The Gomoku game now features a comprehensive theme system that allows for easy customization of the UI colors and styling. The system supports multiple themes including Dark mode and Synthwave mode, with the ability to dynamically switch between themes and load custom themes from JSON files.

## Features

-   **Dynamic Theme Switching**: Switch between themes at runtime using the "Toggle Theme" button in the main menu
-   **JSON-based Configuration**: Themes are defined in JSON files for easy customization
-   **Component-based Theming**: UI elements are automatically updated when themes change
-   **Extensible System**: Easy to add new themes and color schemes

## Built-in Themes

### Dark Theme

-   Clean, professional dark color scheme
-   Easy on the eyes for extended gaming sessions
-   Uses muted grays and greens

### Synthwave Theme

-   Retro 80s cyberpunk aesthetic
-   Bright magentas, cyans, and purples
-   High contrast for visual appeal

## Theme File Format

Themes are defined in JSON files located in `assets/themes/`. Each theme file contains:

```json
{
	"name": "Theme Name",
	"colors": {
		"primary": { "r": 1.0, "g": 0.0, "b": 1.0, "a": 1.0 },
		"secondary": { "r": 0.0, "g": 1.0, "b": 1.0, "a": 1.0 },
		"accent": { "r": 1.0, "g": 0.2, "b": 0.8, "a": 1.0 },
		"background": { "r": 0.05, "g": 0.0, "b": 0.15, "a": 1.0 },
		"surface": { "r": 0.1, "g": 0.0, "b": 0.2, "a": 1.0 },
		"text_primary": { "r": 0.0, "g": 1.0, "b": 1.0, "a": 1.0 },
		"text_secondary": { "r": 1.0, "g": 0.0, "b": 1.0, "a": 1.0 },
		"button_normal": { "r": 0.2, "g": 0.0, "b": 0.4, "a": 1.0 },
		"button_hovered": { "r": 0.4, "g": 0.0, "b": 0.6, "a": 1.0 },
		"button_pressed": { "r": 1.0, "g": 0.2, "b": 0.8, "a": 1.0 },
		"title_background": { "r": 0.1, "g": 0.0, "b": 0.2, "a": 0.8 },
		"content_background": { "r": 0.2, "g": 0.0, "b": 0.4, "a": 0.6 },
		"footer_background": { "r": 0.3, "g": 0.0, "b": 0.6, "a": 0.7 }
	}
}
```

### Color Properties

-   **primary**: Main brand color
-   **secondary**: Secondary brand color
-   **accent**: Highlight and selection color
-   **background**: Main background color
-   **surface**: Secondary background for panels
-   **text_primary**: Main text color
-   **text_secondary**: Secondary text color
-   **button_normal**: Default button background
-   **button_hovered**: Button background when hovered
-   **button_pressed**: Button background when pressed/selected
-   **title_background**: Background for title sections
-   **content_background**: Background for main content areas
-   **footer_background**: Background for footer sections

Color values are in RGBA format with values from 0.0 to 1.0.

## Creating Custom Themes

To create a custom theme:

1. Create a new JSON file in `assets/themes/`
2. Use the format shown above
3. Customize the color values to your preference
4. The theme will be automatically loaded when the game starts

## How It Works

The theme system uses several key components:

### ThemeManager Resource

-   Manages the current theme and available themes
-   Handles theme switching and loading from files
-   Provides access to theme colors throughout the application

### ThemeElement Component

-   Applied to UI elements that should respond to theme changes
-   Specifies what type of themed element it is (button, text, background, etc.)

### Theme Update System

-   Automatically updates UI elements when the theme changes
-   Runs every frame but only applies changes when the theme actually changes

## Usage in Code

### Applying Themes to New UI Elements

When creating new UI elements, use the theme system like this:

```rust
// Get the current theme
let theme = &theme_manager.current_theme;

// Create a button with theme colors
parent.spawn((
    Button,
    BackgroundColor(theme.colors.button_normal.clone().into()),
    ThemeElement { element_type: ThemeElementType::ButtonNormal },
))
.with_children(|parent| {
    parent.spawn((
        Text::new("Button Text"),
        TextColor(theme.colors.text_primary.clone().into()),
        ThemeElement { element_type: ThemeElementType::TextPrimary },
    ));
});
```

### Adding New Theme Element Types

To add new theme element types:

1. Add the new type to `ThemeElementType` enum
2. Add corresponding colors to the theme JSON files
3. Update the theme update system to handle the new type

## Future Enhancements

-   User-selectable themes in settings menu
-   Theme preview system
-   Gradient and animation support
-   Per-component theme overrides
-   Theme import/export functionality
