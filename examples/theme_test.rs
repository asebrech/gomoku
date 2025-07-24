use std::fs;

fn main() {
    println!("Testing theme file loading...");
    
    // Test loading dark theme
    match fs::read_to_string("assets/themes/dark_theme.json") {
        Ok(content) => {
            println!("✅ Dark theme file loaded successfully");
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => println!("✅ Dark theme JSON is valid"),
                Err(e) => println!("❌ Dark theme JSON is invalid: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to load dark theme: {}", e),
    }
    
    // Test loading synthwave theme
    match fs::read_to_string("assets/themes/synthwave_theme.json") {
        Ok(content) => {
            println!("✅ Synthwave theme file loaded successfully");
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => println!("✅ Synthwave theme JSON is valid"),
                Err(e) => println!("❌ Synthwave theme JSON is invalid: {}", e),
            }
        },
        Err(e) => println!("❌ Failed to load synthwave theme: {}", e),
    }
}
