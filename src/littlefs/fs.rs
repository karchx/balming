use std::ffi::CString;

use anyhow::{Context, Result};
use esp_idf_svc::sys::{esp, esp_littlefs_format};
use esp_idf_svc::{fs::littlefs::Littlefs, io::vfs::MountedLittlefs};

const PARTITION: &str = "storage";
const BASE: &str = "/littlefs";

pub struct Fs {
    _mounted: MountedLittlefs<Littlefs<()>>,
    pub base: &'static str,
}

impl Fs {
    pub fn mount() -> Result<Self> {
        let mounted = Self::try_mount().or_else(|e| {
            log::warn!("LittleFS mount failed ({e:?}), formatting '{PARTITION}'...");
            let label = CString::new(PARTITION).context("Invalid partition label")?;
            esp!(unsafe { esp_littlefs_format(label.as_ptr()) })
                .context("esp_littlefs_format failed")?;
            Self::try_mount().context("mount after format failed")
        })?;

        Ok(Self {
            _mounted: mounted,
            base: BASE,
        })
    }

    fn try_mount() -> Result<MountedLittlefs<Littlefs<()>>> {
        log::info!("Trying to mount LittleFS on partition '{}'", PARTITION);

        let littlefs = unsafe { Littlefs::new_partition(PARTITION) }
            .context("Littlefs::new_partition failed - check config partition table")?;

        let mounted =
            MountedLittlefs::mount(littlefs, BASE).context("MountedLittlefs::mount failed")?;

        if !std::path::Path::new(BASE).exists() {
            anyhow::bail!("Mount reported success but {} does not exist in VFS", BASE);
        }

        log::info!("LittleFS mounted successfully at {}", BASE);

        Ok(mounted)
    }
}
