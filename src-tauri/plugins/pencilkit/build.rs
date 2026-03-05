const COMMANDS: &[&str] = &[
    "is_available",
    "show",
    "hide",
    "clear",
    "get_drawing",
    "set_drawing",
    "get_image",
    "set_tool",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .build();
}
