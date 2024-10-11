use bevy_reflect::{
    erased_serde::__private::serde::{Deserialize, Serialize},
    Reflect,
};
use strum_macros::{Display, EnumString};

pub const DOMAIN: &str = "sensor";

pub const CONF_STATE_CLASS: &str = "state_class";

pub const ATTR_LAST_RESET: &str = "last_reset";
pub const ATTR_STATE_CLASS: &str = "state_class";
pub const ATTR_OPTIONS: &str = "options";

pub const ENTITY_ID_FORMAT: &str = "sensor.{}";

#[derive(Debug, EnumString, Display, PartialEq, Clone, Eq, Reflect, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
#[strum(ascii_case_insensitive)]
pub enum SensorDeviceClass {
    // Non-numerical device classes
    Date, /* Date. Unit of measurement: None. ISO8601 format: https://en.wikipedia.org/wiki/ISO_8601 */
    Enum, /* Enumeration. Provides a fixed list of options the state of the sensor can be in.
           * Unit of measurement: None */
    Timestamp, /* Timestamp. Unit of measurement: None. ISO8601 format: https://en.wikipedia.org/wiki/ISO_8601 */

    // Numerical device classes, these should be aligned with NumberDeviceClass
    ApparentPower, // Apparent power. Unit of measurement: VA
    Aqi,           // Air Quality Index. Unit of measurement: None
    AtmosphericPressure, /* Atmospheric pressure. Unit of measurement: UnitOfPressure
                    * units */
    Battery, // Percentage of battery that is left. Unit of measurement: %
    Co,      /* Carbon Monoxide gas concentration. Unit of measurement: ppm
              * (parts per million) */
    Co2, // Carbon Dioxide gas concentration. Unit of measurement: ppm (parts per million)
    Conductivity, // Conductivity. Unit of measurement: S/cm, mS/cm, µS/cm
    Current, // Current. Unit of measurement: A, mA
    DataRate, // Data rate. Unit of measurement: UnitOfDataRate
    DataSize, // Data size. Unit of measurement: UnitOfInformation
    Distance, // Generic distance. Unit of measurement: LENGTH_* units
    Duration, // Fixed duration. Unit of measurement: d, h, min, s, ms
    Energy, // Energy. Unit of measurement: J, kJ, MJ, GJ, Wh, kWh, MWh, cal, kcal, Mcal, Gcal
    EnergyStorage, // Stored energy. Unit of measurement: Wh, kWh, MWh, MJ, GJ
    Frequency, // Frequency. Unit of measurement: Hz, kHz, MHz, GHz
    Gas, // Gas. Unit of measurement: m³, ft³, CCF
    Humidity, // Relative humidity. Unit of measurement: %
    Illuminance, // Illuminance. Unit of measurement: lx
    Irradiance, // Irradiance. Unit of measurement: W/m², BTU/(h⋅ft²)
    Moisture, // Moisture. Unit of measurement: %
    Monetary, // Amount of money. Unit of measurement: ISO4217 currency code
    NitrogenDioxide, // Amount of NO2. Unit of measurement: µg/m³
    NitrogenMonoxide, // Amount of NO. Unit of measurement: µg/m³
    NitrousOxide, // Amount of N2O. Unit of measurement: µg/m³
    Ozone, // Amount of O3. Unit of measurement: µg/m³
    Ph,  // Potential hydrogen (acidity/alkalinity). Unit of measurement: Unitless
    Pm1, // Particulate matter <= 1 μm. Unit of measurement: µg/m³
    Pm10, // Particulate matter <= 10 μm. Unit of measurement: µg/m³
    Pm25, // Particulate matter <= 2.5 μm. Unit of measurement: µg/m³
    PowerFactor, // Power factor. Unit of measurement: %, None
    Power, // Power. Unit of measurement: W, kW
    Precipitation, // Accumulated precipitation. Unit of measurement: UnitOfPrecipitationDepth
    PrecipitationIntensity, // Precipitation intensity. Unit of measurement: UnitOfVolumetricFlux
    Pressure, // Pressure. Unit of measurement: mbar, cbar, bar, Pa, hPa, kPa, inHg, psi
    ReactivePower, // Reactive power. Unit of measurement: var
    SignalStrength, // Signal strength. Unit of measurement: dB, dBm
    SoundPressure, // Sound pressure. Unit of measurement: dB, dBA
    Speed, // Generic speed. Unit of measurement: SPEED_* units or UnitOfVolumetricFlux
    SulphurDioxide, // Amount of SO2. Unit of measurement: µg/m³
    Temperature, // Temperature. Unit of measurement: °C, °F, K
    VolatileOrganicCompounds, // Amount of VOC. Unit of measurement: µg/m³
    VolatileOrganicCompoundsParts, // Ratio of VOC. Unit of measurement: ppm, ppb
    Voltage, // Voltage. Unit of measurement: V, mV
    Volume, // Generic volume. Unit of measurement: VOLUME_* units
    VolumeStorage, // Generic stored volume. Unit of measurement: VOLUME_* units
    VolumeFlowRate, // Generic flow rate. Unit of measurement: UnitOfVolumeFlowRate
    Water, // Water. Unit of measurement: m³, L, ft³, CCF, gal
    Weight, /* Generic weight, represents a measurement of an object's mass. Unit of measurement:
          * MASS_* units */
    WindSpeed, // Wind speed. Unit of measurement: SPEED_* units
}

impl SensorDeviceClass {
    pub fn unit_of_measurement(&self) -> Option<String> {
        let s = match self {
            SensorDeviceClass::Date => None,
            SensorDeviceClass::Enum => None,
            SensorDeviceClass::Timestamp => None,
            SensorDeviceClass::ApparentPower => Some("VA"),
            SensorDeviceClass::Aqi => None,
            SensorDeviceClass::AtmosphericPressure => Some("UnitOfPressure"),
            SensorDeviceClass::Battery => Some("%"),
            SensorDeviceClass::Co => Some("ppm"),
            SensorDeviceClass::Co2 => Some("ppm"),
            SensorDeviceClass::Conductivity => Some("S/cm, mS/cm, µS/cm"),
            SensorDeviceClass::Current => Some("A, mA"),
            SensorDeviceClass::DataRate => Some("UnitOfDataRate"),
            SensorDeviceClass::DataSize => Some("UnitOfInformation"),
            SensorDeviceClass::Distance => Some("LENGTH_* units"),
            SensorDeviceClass::Duration => Some("d, h, min, s, ms"),
            SensorDeviceClass::Energy => Some("J, kJ, MJ, GJ, Wh, kWh, MWh, cal, kcal, Mcal, Gcal"),
            SensorDeviceClass::EnergyStorage => Some("Wh, kWh, MWh, MJ, GJ"),
            SensorDeviceClass::Frequency => Some("Hz, kHz, MHz, GHz"),
            SensorDeviceClass::Gas => Some("m³, ft³, CCF"),
            SensorDeviceClass::Humidity => Some("%"),
            SensorDeviceClass::Illuminance => Some("lx"),
            SensorDeviceClass::Irradiance => Some("W/m², BTU/(h⋅ft²)"),
            SensorDeviceClass::Moisture => Some("%"),
            SensorDeviceClass::Monetary => Some("ISO4217 currency code"),
            SensorDeviceClass::NitrogenDioxide => Some("µg/m³"),
            SensorDeviceClass::NitrogenMonoxide => Some("µg/m³"),
            SensorDeviceClass::NitrousOxide => Some("µg/m³"),
            SensorDeviceClass::Ozone => Some("µg/m³"),
            SensorDeviceClass::Ph => Some("Unitless"),
            SensorDeviceClass::Pm1 => Some("µg/m³"),
            SensorDeviceClass::Pm10 => Some("µg/m³"),
            SensorDeviceClass::Pm25 => Some("µg/m³"),
            SensorDeviceClass::PowerFactor => Some("%, None"),
            SensorDeviceClass::Power => Some("W, kW"),
            SensorDeviceClass::Precipitation => Some("UnitOfPrecipitationDepth"),
            SensorDeviceClass::PrecipitationIntensity => Some("UnitOfVolumetricFlux"),
            SensorDeviceClass::Pressure => Some("mbar, cbar, bar, Pa, hPa, kPa, inHg, psi"),
            SensorDeviceClass::ReactivePower => Some("var"),
            SensorDeviceClass::SignalStrength => Some("dB, dBm"),
            SensorDeviceClass::SoundPressure => Some("dB, dBA"),
            SensorDeviceClass::Speed => Some("SPEED_* units or UnitOfVolumetricFlux"),
            SensorDeviceClass::SulphurDioxide => Some("µg/m³"),
            SensorDeviceClass::Temperature => Some("°C, °F, K"),
            SensorDeviceClass::VolatileOrganicCompounds => Some("µg/m³"),
            SensorDeviceClass::VolatileOrganicCompoundsParts => Some("ppm, ppb"),
            SensorDeviceClass::Voltage => Some("V, mV"),
            SensorDeviceClass::Volume => Some("VOLUME_* units"),
            SensorDeviceClass::VolumeStorage => Some("VOLUME_* units"),
            SensorDeviceClass::VolumeFlowRate => Some("UnitOfVolumeFlowRate"),
            SensorDeviceClass::Water => Some("m³, L, ft³, CCF, gal"),
            SensorDeviceClass::Weight => Some("MASS_* units"),
            SensorDeviceClass::WindSpeed => Some("SPEED_* units"),
        };

        s.map(|v| v.to_string())
    }
}

#[cfg(test)]
mod test {
    use crate::SensorDeviceClass;
    use std::str::FromStr;

    #[test]
    fn test_sensor_class() {
        let sensor = SensorDeviceClass::Temperature;
        assert_eq!(sensor.to_string(), "temperature");
        assert_eq!(
            SensorDeviceClass::from_str("temperature").unwrap(),
            SensorDeviceClass::Temperature
        );
        assert_eq!(
            SensorDeviceClass::from_str("signal_strength").unwrap(),
            SensorDeviceClass::SignalStrength
        );
    }
}
