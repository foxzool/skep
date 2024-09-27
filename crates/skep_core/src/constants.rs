// // Format for platform files
// pub const PLATFORM_FORMAT: &str = "{platform}.{domain}";
//
// use bevy_ecs::component::Component;
// use bevy_reflect::Reflect;
// use strum_macros::{Display, EnumString};
//
// #[derive(EnumString, Hash, Eq, Debug, Display, PartialEq, Clone, Copy, Reflect, Component)]
// #[strum(serialize_all = "snake_case")]
// #[strum(ascii_case_insensitive)]
// pub enum Platform {
//     AirQuality,
//     AlarmControlPanel,
//     AssistSatellite,
//     BinarySensor,
//     Button,
//     Calendar,
//     Camera,
//     Climate,
//     Conversation,
//     Cover,
//     Date,
//     DateTime,
//     DeviceTracker,
//     Event,
//     Fan,
//     GeoLocation,
//     Humidifier,
//     Image,
//     ImageProcessing,
//     LawnMower,
//     Light,
//     Lock,
//     MediaPlayer,
//     Notify,
//     Number,
//     Remote,
//     Scene,
//     Select,
//     Sensor,
//     Siren,
//     Stt,
//     Switch,
//     Text,
//     Time,
//     Todo,
//     Tts,
//     Vacuum,
//     Valve,
//     Update,
//     WakeWord,
//     WaterHeater,
//     Weather,
// }
//
// #[cfg(test)]
// mod test {
//     use crate::constants::Platform;
//     use std::str::FromStr;
//
//     #[test]
//     fn test_platform() {
//         let platform = Platform::Sensor;
//         assert_eq!(platform.to_string(), "sensor");
//         assert_eq!(Platform::from_str("sensor").unwrap(), Platform::Sensor);
//         assert_eq!(Platform::from_str("Sensor").unwrap(), Platform::Sensor);
//     }
// }
