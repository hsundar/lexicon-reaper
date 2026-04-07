use ratatui::style::Color;

use crate::generation::stage::Biome;

pub struct BiomeTheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,
    pub enemy_art: Color,
    pub bg_hint: Color,
    pub ground: Color,
    pub prop1: Color,
    pub prop2: Color,
    pub sky: Color,
}

pub fn theme_for_biome(biome: Biome) -> BiomeTheme {
    match biome {
        Biome::Forest => BiomeTheme {
            primary: Color::Rgb(100, 200, 100),
            secondary: Color::Rgb(60, 140, 60),
            accent: Color::Rgb(180, 255, 120),
            border: Color::Rgb(80, 160, 80),
            enemy_art: Color::Rgb(120, 220, 90),
            bg_hint: Color::Rgb(15, 30, 15),
            ground: Color::Rgb(60, 100, 40),
            prop1: Color::Rgb(140, 90, 50),   // wood/mushroom
            prop2: Color::Rgb(80, 180, 60),    // leaves/vines
            sky: Color::Rgb(20, 40, 25),
        },
        Biome::Crypt => BiomeTheme {
            primary: Color::Rgb(160, 160, 200),
            secondary: Color::Rgb(100, 100, 140),
            accent: Color::Rgb(200, 200, 255),
            border: Color::Rgb(120, 120, 170),
            enemy_art: Color::Rgb(180, 180, 220),
            bg_hint: Color::Rgb(20, 20, 35),
            ground: Color::Rgb(70, 70, 90),
            prop1: Color::Rgb(200, 180, 100),  // candles/gold
            prop2: Color::Rgb(140, 140, 160),  // stone/bones
            sky: Color::Rgb(25, 25, 40),
        },
        Biome::Volcano => BiomeTheme {
            primary: Color::Rgb(255, 120, 50),
            secondary: Color::Rgb(180, 80, 30),
            accent: Color::Rgb(255, 200, 80),
            border: Color::Rgb(200, 100, 40),
            enemy_art: Color::Rgb(255, 140, 60),
            bg_hint: Color::Rgb(35, 15, 10),
            ground: Color::Rgb(100, 50, 30),
            prop1: Color::Rgb(255, 100, 30),   // fire/lava
            prop2: Color::Rgb(120, 80, 60),    // rocks
            sky: Color::Rgb(50, 20, 10),
        },
        Biome::Abyss => BiomeTheme {
            primary: Color::Rgb(180, 80, 220),
            secondary: Color::Rgb(120, 50, 160),
            accent: Color::Rgb(220, 140, 255),
            border: Color::Rgb(150, 60, 190),
            enemy_art: Color::Rgb(200, 100, 240),
            bg_hint: Color::Rgb(25, 10, 35),
            ground: Color::Rgb(50, 30, 70),
            prop1: Color::Rgb(100, 200, 255),  // crystals
            prop2: Color::Rgb(180, 80, 220),   // portals
            sky: Color::Rgb(20, 10, 35),
        },
        Biome::Void => BiomeTheme {
            primary: Color::Rgb(200, 200, 200),
            secondary: Color::Rgb(140, 140, 140),
            accent: Color::Rgb(255, 255, 255),
            border: Color::Rgb(170, 170, 170),
            enemy_art: Color::Rgb(220, 220, 220),
            bg_hint: Color::Rgb(10, 10, 10),
            ground: Color::Rgb(40, 40, 50),
            prop1: Color::Rgb(180, 180, 200),  // debris
            prop2: Color::Rgb(100, 100, 120),  // glitch
            sky: Color::Rgb(8, 8, 12),
        },
    }
}
