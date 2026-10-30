use anyhow::{Context, Result};
use esp_idf_svc::sys::{esp, esp_littlefs_format};
use esp_idf_svc::{fs::littlefs::Littlefs, io::vfs::MountedLittlefs};
use std::fs;

const PARTITION: &str = "ffat";
const BASE: &str = "/littlefs";

pub struct Fs {
    _mounted: MountedLittlefs<Littlefs<()>>,
    pub base: &'static str,
}

impl Fs {
    pub fn mount() -> Result<Self> {
        let mounted = Self::try_mount().or_else(|e| {
            log::warn!("LittleFS mount failed ({e:?}), formatting '{PARTITION}'...");
            esp!(unsafe { esp_littlefs_format(PARTITION.as_ptr() as *const _) })
                .context("esp_littlefs_format failed")?;
            Self::try_mount().context("mount after format failed")
        })?;

        Ok(Self {
            _mounted: mounted,
            base: BASE,
        })
    }

    fn try_mount() -> Result<MountedLittlefs<Littlefs<()>>> {
        let littlefs =
            unsafe { Littlefs::new_partition(PARTITION) }.context("partition not found")?;

        MountedLittlefs::mount(littlefs, BASE).map_err(Into::into)
    }
}
