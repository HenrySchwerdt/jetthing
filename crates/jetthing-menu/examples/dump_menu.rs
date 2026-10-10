//! Prints a presented menu level: `cargo run -p jetthing-menu --example dump_menu [MenuId]`.

use jetthing_menu::{
    ActionContext, Binding, BuiltInDynamicItems, Keymap, Presented, ShortcutSource, main_menu,
    present,
};

struct AllEnabled;

impl ActionContext for AllEnabled {
    fn is_enabled(&self, _binding: &Binding) -> bool {
        true
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wanted = std::env::args().nth(1).unwrap_or_else(|| "FileMenu".into());
    let keymap = Keymap::platform_default()?;
    let shortcuts: &dyn ShortcutSource = &keymap;
    let menus = main_menu(&BuiltInDynamicItems)?;
    let Some(menu) = menus.iter().find(|menu| menu.id == wanted) else {
        return Err(format!("no main-menu entry {wanted}").into());
    };
    println!("{}", menu.text);
    for row in present(&menu.items, &AllEnabled, shortcuts) {
        match row {
            Presented::Separator(text) => println!("  ----- {}", text.unwrap_or_default()),
            Presented::Action {
                text,
                shortcut,
                enabled,
                ..
            } => println!(
                "  {}{text}  {}",
                if enabled { "" } else { "(disabled) " },
                shortcut.unwrap_or_default()
            ),
            Presented::Submenu { text, enabled, .. } => {
                println!("  {}{text} ▸", if enabled { "" } else { "(disabled) " })
            }
        }
    }
    Ok(())
}
