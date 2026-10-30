use anyhow::Result;
use esp_idf_hal::temp_sensor::{TempSensor, TempSensorConfig, TempSensorDriver};

pub struct ChipInfo {
    temp_sensor: TempSensorDriver<'static>,
}

impl ChipInfo {
    pub fn new(sensor_peripheral: TempSensor<'static>) -> Result<Self> {
        let config = TempSensorConfig::default();
        let mut temp_sensor = TempSensorDriver::new(&config, sensor_peripheral)?;

        temp_sensor.enable()?;
        Ok(Self { temp_sensor })
    }

    pub fn chip_info_log(&self) -> Result<()> {
        let temp = self.temp_sensor.get_celsius()?;
        log::info!("Temperatura: {:.1}°C", temp);
        Ok(())
    }
}
