//! Immortal Engine v2.0 (S-Arch-P)
//!
//! Visual Code Generator for Rust Applications
//!
//! This is the main entry point for the Dioxus Desktop application.

use tracing::Level;
use tracing_subscriber::FmtSubscriber;

fn main() {
    // Initialize logging
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .pretty()
        .init();

    // Print startup banner
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║                                                           ║");
    println!("║   🔮 Immortal Engine v2.0 (S-Arch-P)                      ║");
    println!("║   Visual Code Generator for Rust Applications            ║");
    println!("║                                                           ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // Launch the Dioxus desktop application
    imortal_ui::launch();
}
