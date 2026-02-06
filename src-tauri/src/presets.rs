use crate::config::RainColor;
use serde::{Deserialize, Serialize};

/// A rain preset with a name and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: String,
    pub config: PresetConfig,
}

/// Subset of RainConfig used in presets (excludes enabled state)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub intensity: f32,
    pub speed: f32,
    pub angle: f32,
    pub drop_length: f32,
    pub drop_width: f32,
    pub color: RainColor,
    pub opacity: f32,
    pub splash_enabled: bool,
    pub splash_intensity: f32,
}

/// Get built-in presets
pub fn get_builtin_presets() -> Vec<Preset> {
    vec![
        Preset {
            name: "Light Drizzle".to_string(),
            description: "Gentle, calming light rain".to_string(),
            config: PresetConfig {
                intensity: 0.3,
                speed: 0.8,
                angle: 0.0,
                drop_length: 15.0,
                drop_width: 1.5,
                color: RainColor {
                    r: 200,
                    g: 220,
                    b: 255,
                    a: 120,
                },
                opacity: 0.5,
                splash_enabled: true,
                splash_intensity: 0.3,
            },
        },
        Preset {
            name: "Steady Rain".to_string(),
            description: "Moderate steady rainfall".to_string(),
            config: PresetConfig {
                intensity: 0.5,
                speed: 1.0,
                angle: 0.0,
                drop_length: 20.0,
                drop_width: 2.0,
                color: RainColor {
                    r: 174,
                    g: 194,
                    b: 224,
                    a: 180,
                },
                opacity: 0.7,
                splash_enabled: true,
                splash_intensity: 0.5,
            },
        },
        Preset {
            name: "Heavy Downpour".to_string(),
            description: "Intense heavy rain".to_string(),
            config: PresetConfig {
                intensity: 0.9,
                speed: 1.5,
                angle: -5.0,
                drop_length: 30.0,
                drop_width: 2.5,
                color: RainColor {
                    r: 150,
                    g: 170,
                    b: 200,
                    a: 200,
                },
                opacity: 0.85,
                splash_enabled: true,
                splash_intensity: 0.8,
            },
        },
        Preset {
            name: "Windy Storm".to_string(),
            description: "Rain blown by strong wind".to_string(),
            config: PresetConfig {
                intensity: 0.7,
                speed: 2.0,
                angle: -30.0,
                drop_length: 25.0,
                drop_width: 2.0,
                color: RainColor {
                    r: 140,
                    g: 160,
                    b: 190,
                    a: 180,
                },
                opacity: 0.75,
                splash_enabled: true,
                splash_intensity: 0.6,
            },
        },
        Preset {
            name: "Misty".to_string(),
            description: "Fine misty rain, almost fog-like".to_string(),
            config: PresetConfig {
                intensity: 0.6,
                speed: 0.5,
                angle: 0.0,
                drop_length: 8.0,
                drop_width: 1.0,
                color: RainColor {
                    r: 220,
                    g: 230,
                    b: 245,
                    a: 100,
                },
                opacity: 0.4,
                splash_enabled: false,
                splash_intensity: 0.0,
            },
        },
        Preset {
            name: "Cyberpunk".to_string(),
            description: "Neon-tinted futuristic rain".to_string(),
            config: PresetConfig {
                intensity: 0.5,
                speed: 1.2,
                angle: -10.0,
                drop_length: 35.0,
                drop_width: 1.5,
                color: RainColor {
                    r: 0,
                    g: 255,
                    b: 200,
                    a: 150,
                },
                opacity: 0.6,
                splash_enabled: true,
                splash_intensity: 0.4,
            },
        },
        Preset {
            name: "Warm Sunset".to_string(),
            description: "Golden hour rain".to_string(),
            config: PresetConfig {
                intensity: 0.4,
                speed: 0.9,
                angle: 5.0,
                drop_length: 18.0,
                drop_width: 2.0,
                color: RainColor {
                    r: 255,
                    g: 200,
                    b: 150,
                    a: 140,
                },
                opacity: 0.55,
                splash_enabled: true,
                splash_intensity: 0.35,
            },
        },
        Preset {
            name: "Blood Rain".to_string(),
            description: "Ominous red-tinted rain".to_string(),
            config: PresetConfig {
                intensity: 0.6,
                speed: 1.1,
                angle: -8.0,
                drop_length: 22.0,
                drop_width: 2.5,
                color: RainColor {
                    r: 180,
                    g: 50,
                    b: 50,
                    a: 160,
                },
                opacity: 0.65,
                splash_enabled: true,
                splash_intensity: 0.5,
            },
        },
    ]
}
