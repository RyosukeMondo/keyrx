#[cfg(feature = "linux")]
pub mod linux;

#[cfg(feature = "windows")]
pub mod windows;

#[cfg(feature = "linux")]
pub use linux::LinuxPlatform;

#[cfg(feature = "windows")]
pub use windows::WindowsPlatform;

pub enum Platform {
    #[cfg(feature = "linux")]
    Linux(LinuxPlatform),
    #[cfg(feature = "windows")]
    Windows(WindowsPlatform),
    #[cfg(not(any(feature = "linux", feature = "windows")))]
    Unsupported,
}

impl Platform {
    pub fn new() -> Self {
        #[cfg(feature = "linux")]
        {
            Platform::Linux(LinuxPlatform::new())
        }
        #[cfg(all(feature = "windows", not(feature = "linux")))]
        {
            Platform::Windows(WindowsPlatform::new())
        }
        #[cfg(not(any(feature = "linux", feature = "windows")))]
        {
            Platform::Unsupported
        }
    }

    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(feature = "linux")]
            Platform::Linux(p) => p.init(),
            #[cfg(feature = "windows")]
            Platform::Windows(p) => p.init(),
            #[cfg(not(any(feature = "linux", feature = "windows")))]
            Platform::Unsupported => Ok(()),
        }
    }

    pub fn process_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            #[cfg(feature = "linux")]
            Platform::Linux(p) => p.process_events(),
            #[cfg(feature = "windows")]
            Platform::Windows(p) => p.process_events(),
            #[cfg(not(any(feature = "linux", feature = "windows")))]
            Platform::Unsupported => Ok(()),
        }
    }
}

impl Default for Platform {
    fn default() -> Self {
        Self::new()
    }
}
