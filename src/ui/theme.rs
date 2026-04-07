use ratatui::style::Color;

use crate::generation::stage::Biome;

pub struct BiomeTheme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub border: Color,
    pub enemy_art: Color,
    pub bg_hint: Color,
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
        },
        Biome::Crypt => BiomeTheme {
            primary: Color::Rgb(160, 160, 200),
            secondary: Color::Rgb(100, 100, 140),
            accent: Color::Rgb(200, 200, 255),
            border: Color::Rgb(120, 120, 170),
            enemy_art: Color::Rgb(180, 180, 220),
            bg_hint: Color::Rgb(20, 20, 35),
        },
        Biome::Volcano => BiomeTheme {
            primary: Color::Rgb(255, 120, 50),
            secondary: Color::Rgb(180, 80, 30),
            accent: Color::Rgb(255, 200, 80),
            border: Color::Rgb(200, 100, 40),
            enemy_art: Color::Rgb(255, 140, 60),
            bg_hint: Color::Rgb(35, 15, 10),
        },
        Biome::Abyss => BiomeTheme {
            primary: Color::Rgb(180, 80, 220),
            secondary: Color::Rgb(120, 50, 160),
            accent: Color::Rgb(220, 140, 255),
            border: Color::Rgb(150, 60, 190),
            enemy_art: Color::Rgb(200, 100, 240),
            bg_hint: Color::Rgb(25, 10, 35),
        },
        Biome::Void => BiomeTheme {
            primary: Color::Rgb(200, 200, 200),
            secondary: Color::Rgb(140, 140, 140),
            accent: Color::Rgb(255, 255, 255),
            border: Color::Rgb(170, 170, 170),
            enemy_art: Color::Rgb(220, 220, 220),
            bg_hint: Color::Rgb(10, 10, 10),
        },
    }
}
