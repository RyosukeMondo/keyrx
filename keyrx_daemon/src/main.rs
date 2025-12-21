mod platform;

#[cfg(feature = "web")]
mod web;

fn main() {
    println!("KeyRx Daemon - OS-level keyboard remapping");

    #[cfg(feature = "web")]
    {
        println!("Web server feature enabled");
    }

    #[cfg(feature = "linux")]
    {
        println!("Linux platform support enabled");
    }

    #[cfg(feature = "windows")]
    {
        println!("Windows platform support enabled");
    }
}
