/// CSS color palette

pub struct Colors;

impl Colors {
    // Slate
    pub const SLATE_50: (u8, u8, u8) = (248, 250, 252);
    pub const SLATE_100: (u8, u8, u8) = (241, 245, 249);
    pub const SLATE_200: (u8, u8, u8) = (226, 232, 240);
    pub const SLATE_300: (u8, u8, u8) = (203, 213, 225);
    pub const SLATE_400: (u8, u8, u8) = (148, 163, 184);
    pub const SLATE_500: (u8, u8, u8) = (100, 116, 139);
    pub const SLATE_600: (u8, u8, u8) = (71, 85, 105);
    pub const SLATE_700: (u8, u8, u8) = (51, 65, 85);
    pub const SLATE_800: (u8, u8, u8) = (30, 41, 59);
    pub const SLATE_900: (u8, u8, u8) = (15, 23, 42);
    pub const SLATE_950: (u8, u8, u8) = (2, 6, 23);

    // Gray
    pub const GRAY_50: (u8, u8, u8) = (249, 250, 251);
    pub const GRAY_100: (u8, u8, u8) = (243, 244, 246);
    pub const GRAY_200: (u8, u8, u8) = (229, 231, 235);
    pub const GRAY_300: (u8, u8, u8) = (209, 213, 219);
    pub const GRAY_400: (u8, u8, u8) = (156, 163, 175);
    pub const GRAY_500: (u8, u8, u8) = (107, 114, 128);
    pub const GRAY_600: (u8, u8, u8) = (75, 85, 99);
    pub const GRAY_700: (u8, u8, u8) = (55, 65, 81);
    pub const GRAY_800: (u8, u8, u8) = (31, 41, 55);
    pub const GRAY_900: (u8, u8, u8) = (17, 24, 39);
    pub const GRAY_950: (u8, u8, u8) = (3, 7, 18);

    // Zinc
    pub const ZINC_50: (u8, u8, u8) = (250, 250, 250);
    pub const ZINC_100: (u8, u8, u8) = (244, 244, 245);
    pub const ZINC_200: (u8, u8, u8) = (228, 228, 231);
    pub const ZINC_300: (u8, u8, u8) = (212, 212, 216);
    pub const ZINC_400: (u8, u8, u8) = (161, 161, 170);
    pub const ZINC_500: (u8, u8, u8) = (113, 113, 122);
    pub const ZINC_600: (u8, u8, u8) = (82, 82, 91);
    pub const ZINC_700: (u8, u8, u8) = (63, 63, 70);
    pub const ZINC_800: (u8, u8, u8) = (39, 39, 42);
    pub const ZINC_900: (u8, u8, u8) = (24, 24, 27);
    pub const ZINC_950: (u8, u8, u8) = (9, 9, 11);

    // Neutral
    pub const NEUTRAL_50: (u8, u8, u8) = (250, 250, 250);
    pub const NEUTRAL_100: (u8, u8, u8) = (245, 245, 245);
    pub const NEUTRAL_200: (u8, u8, u8) = (229, 229, 229);
    pub const NEUTRAL_300: (u8, u8, u8) = (212, 212, 212);
    pub const NEUTRAL_400: (u8, u8, u8) = (163, 163, 163);
    pub const NEUTRAL_500: (u8, u8, u8) = (115, 115, 115);
    pub const NEUTRAL_600: (u8, u8, u8) = (82, 82, 82);
    pub const NEUTRAL_700: (u8, u8, u8) = (64, 64, 64);
    pub const NEUTRAL_800: (u8, u8, u8) = (38, 38, 38);
    pub const NEUTRAL_900: (u8, u8, u8) = (23, 23, 23);
    pub const NEUTRAL_950: (u8, u8, u8) = (10, 10, 10);

    // Stone
    pub const STONE_50: (u8, u8, u8) = (250, 250, 249);
    pub const STONE_100: (u8, u8, u8) = (245, 245, 244);
    pub const STONE_200: (u8, u8, u8) = (231, 229, 228);
    pub const STONE_300: (u8, u8, u8) = (214, 211, 209);
    pub const STONE_400: (u8, u8, u8) = (168, 162, 158);
    pub const STONE_500: (u8, u8, u8) = (120, 113, 108);
    pub const STONE_600: (u8, u8, u8) = (87, 83, 78);
    pub const STONE_700: (u8, u8, u8) = (68, 64, 60);
    pub const STONE_800: (u8, u8, u8) = (41, 37, 36);
    pub const STONE_900: (u8, u8, u8) = (28, 25, 23);
    pub const STONE_950: (u8, u8, u8) = (12, 10, 9);

    // Red
    pub const RED_50: (u8, u8, u8) = (254, 242, 242);
    pub const RED_100: (u8, u8, u8) = (254, 226, 226);
    pub const RED_200: (u8, u8, u8) = (254, 202, 202);
    pub const RED_300: (u8, u8, u8) = (252, 165, 165);
    pub const RED_400: (u8, u8, u8) = (248, 113, 113);
    pub const RED_500: (u8, u8, u8) = (239, 68, 68);
    pub const RED_600: (u8, u8, u8) = (220, 38, 38);
    pub const RED_700: (u8, u8, u8) = (185, 28, 28);
    pub const RED_800: (u8, u8, u8) = (153, 27, 27);
    pub const RED_900: (u8, u8, u8) = (127, 29, 29);
    pub const RED_950: (u8, u8, u8) = (69, 10, 10);

    // Orange
    pub const ORANGE_50: (u8, u8, u8) = (255, 247, 237);
    pub const ORANGE_100: (u8, u8, u8) = (255, 237, 213);
    pub const ORANGE_200: (u8, u8, u8) = (254, 215, 170);
    pub const ORANGE_300: (u8, u8, u8) = (253, 186, 116);
    pub const ORANGE_400: (u8, u8, u8) = (251, 146, 60);
    pub const ORANGE_500: (u8, u8, u8) = (249, 115, 22);
    pub const ORANGE_600: (u8, u8, u8) = (234, 88, 12);
    pub const ORANGE_700: (u8, u8, u8) = (194, 65, 12);
    pub const ORANGE_800: (u8, u8, u8) = (154, 52, 18);
    pub const ORANGE_900: (u8, u8, u8) = (124, 45, 18);
    pub const ORANGE_950: (u8, u8, u8) = (67, 20, 7);

    // Amber
    pub const AMBER_50: (u8, u8, u8) = (255, 251, 235);
    pub const AMBER_100: (u8, u8, u8) = (254, 243, 199);
    pub const AMBER_200: (u8, u8, u8) = (253, 230, 138);
    pub const AMBER_300: (u8, u8, u8) = (252, 211, 77);
    pub const AMBER_400: (u8, u8, u8) = (251, 191, 36);
    pub const AMBER_500: (u8, u8, u8) = (245, 158, 11);
    pub const AMBER_600: (u8, u8, u8) = (217, 119, 6);
    pub const AMBER_700: (u8, u8, u8) = (180, 83, 9);
    pub const AMBER_800: (u8, u8, u8) = (146, 64, 14);
    pub const AMBER_900: (u8, u8, u8) = (120, 53, 15);
    pub const AMBER_950: (u8, u8, u8) = (69, 26, 3);

    // Yellow
    pub const YELLOW_50: (u8, u8, u8) = (254, 252, 232);
    pub const YELLOW_100: (u8, u8, u8) = (254, 249, 195);
    pub const YELLOW_200: (u8, u8, u8) = (254, 240, 138);
    pub const YELLOW_300: (u8, u8, u8) = (253, 224, 71);
    pub const YELLOW_400: (u8, u8, u8) = (250, 204, 21);
    pub const YELLOW_500: (u8, u8, u8) = (234, 179, 8);
    pub const YELLOW_600: (u8, u8, u8) = (202, 138, 4);
    pub const YELLOW_700: (u8, u8, u8) = (161, 98, 7);
    pub const YELLOW_800: (u8, u8, u8) = (133, 77, 14);
    pub const YELLOW_900: (u8, u8, u8) = (113, 63, 18);
    pub const YELLOW_950: (u8, u8, u8) = (66, 32, 6);

    // Lime
    pub const LIME_50: (u8, u8, u8) = (247, 254, 231);
    pub const LIME_100: (u8, u8, u8) = (236, 252, 203);
    pub const LIME_200: (u8, u8, u8) = (217, 249, 157);
    pub const LIME_300: (u8, u8, u8) = (190, 242, 100);
    pub const LIME_400: (u8, u8, u8) = (163, 230, 53);
    pub const LIME_500: (u8, u8, u8) = (132, 204, 22);
    pub const LIME_600: (u8, u8, u8) = (101, 163, 13);
    pub const LIME_700: (u8, u8, u8) = (77, 124, 15);
    pub const LIME_800: (u8, u8, u8) = (63, 98, 18);
    pub const LIME_900: (u8, u8, u8) = (54, 83, 20);
    pub const LIME_950: (u8, u8, u8) = (26, 46, 5);

    // Green
    pub const GREEN_50: (u8, u8, u8) = (240, 253, 244);
    pub const GREEN_100: (u8, u8, u8) = (220, 252, 231);
    pub const GREEN_200: (u8, u8, u8) = (187, 247, 208);
    pub const GREEN_300: (u8, u8, u8) = (134, 239, 172);
    pub const GREEN_400: (u8, u8, u8) = (74, 222, 128);
    pub const GREEN_500: (u8, u8, u8) = (34, 197, 94);
    pub const GREEN_600: (u8, u8, u8) = (22, 163, 74);
    pub const GREEN_700: (u8, u8, u8) = (21, 128, 61);
    pub const GREEN_800: (u8, u8, u8) = (22, 101, 52);
    pub const GREEN_900: (u8, u8, u8) = (20, 83, 45);
    pub const GREEN_950: (u8, u8, u8) = (5, 46, 22);

    // Emerald
    pub const EMERALD_50: (u8, u8, u8) = (236, 253, 245);
    pub const EMERALD_100: (u8, u8, u8) = (209, 250, 229);
    pub const EMERALD_200: (u8, u8, u8) = (167, 243, 208);
    pub const EMERALD_300: (u8, u8, u8) = (110, 231, 183);
    pub const EMERALD_400: (u8, u8, u8) = (52, 211, 153);
    pub const EMERALD_500: (u8, u8, u8) = (16, 185, 129);
    pub const EMERALD_600: (u8, u8, u8) = (5, 150, 105);
    pub const EMERALD_700: (u8, u8, u8) = (4, 120, 87);
    pub const EMERALD_800: (u8, u8, u8) = (6, 95, 70);
    pub const EMERALD_900: (u8, u8, u8) = (6, 78, 59);
    pub const EMERALD_950: (u8, u8, u8) = (2, 44, 34);

    // Teal
    pub const TEAL_50: (u8, u8, u8) = (240, 253, 250);
    pub const TEAL_100: (u8, u8, u8) = (204, 251, 241);
    pub const TEAL_200: (u8, u8, u8) = (153, 246, 228);
    pub const TEAL_300: (u8, u8, u8) = (94, 234, 212);
    pub const TEAL_400: (u8, u8, u8) = (45, 212, 191);
    pub const TEAL_500: (u8, u8, u8) = (20, 184, 166);
    pub const TEAL_600: (u8, u8, u8) = (13, 148, 136);
    pub const TEAL_700: (u8, u8, u8) = (15, 118, 110);
    pub const TEAL_800: (u8, u8, u8) = (17, 94, 89);
    pub const TEAL_900: (u8, u8, u8) = (19, 78, 74);
    pub const TEAL_950: (u8, u8, u8) = (4, 47, 46);

    // Cyan
    pub const CYAN_50: (u8, u8, u8) = (236, 254, 255);
    pub const CYAN_100: (u8, u8, u8) = (207, 250, 254);
    pub const CYAN_200: (u8, u8, u8) = (165, 243, 252);
    pub const CYAN_300: (u8, u8, u8) = (103, 232, 249);
    pub const CYAN_400: (u8, u8, u8) = (34, 211, 238);
    pub const CYAN_500: (u8, u8, u8) = (6, 182, 212);
    pub const CYAN_600: (u8, u8, u8) = (8, 145, 178);
    pub const CYAN_700: (u8, u8, u8) = (14, 116, 144);
    pub const CYAN_800: (u8, u8, u8) = (21, 94, 117);
    pub const CYAN_900: (u8, u8, u8) = (22, 78, 99);
    pub const CYAN_950: (u8, u8, u8) = (8, 51, 68);

    // Sky
    pub const SKY_50: (u8, u8, u8) = (240, 249, 255);
    pub const SKY_100: (u8, u8, u8) = (224, 242, 254);
    pub const SKY_200: (u8, u8, u8) = (186, 230, 253);
    pub const SKY_300: (u8, u8, u8) = (125, 211, 252);
    pub const SKY_400: (u8, u8, u8) = (56, 189, 248);
    pub const SKY_500: (u8, u8, u8) = (14, 165, 233);
    pub const SKY_600: (u8, u8, u8) = (2, 132, 199);
    pub const SKY_700: (u8, u8, u8) = (3, 105, 161);
    pub const SKY_800: (u8, u8, u8) = (7, 89, 133);
    pub const SKY_900: (u8, u8, u8) = (12, 74, 110);
    pub const SKY_950: (u8, u8, u8) = (8, 47, 73);

    // Blue
    pub const BLUE_50: (u8, u8, u8) = (239, 246, 255);
    pub const BLUE_100: (u8, u8, u8) = (219, 234, 254);
    pub const BLUE_200: (u8, u8, u8) = (191, 219, 254);
    pub const BLUE_300: (u8, u8, u8) = (147, 197, 253);
    pub const BLUE_400: (u8, u8, u8) = (96, 165, 250);
    pub const BLUE_500: (u8, u8, u8) = (59, 130, 246);
    pub const BLUE_600: (u8, u8, u8) = (37, 99, 235);
    pub const BLUE_700: (u8, u8, u8) = (29, 78, 216);
    pub const BLUE_800: (u8, u8, u8) = (30, 64, 175);
    pub const BLUE_900: (u8, u8, u8) = (30, 58, 138);
    pub const BLUE_950: (u8, u8, u8) = (23, 37, 84);

    // Indigo
    pub const INDIGO_50: (u8, u8, u8) = (238, 242, 255);
    pub const INDIGO_100: (u8, u8, u8) = (224, 231, 255);
    pub const INDIGO_200: (u8, u8, u8) = (199, 210, 254);
    pub const INDIGO_300: (u8, u8, u8) = (165, 180, 252);
    pub const INDIGO_400: (u8, u8, u8) = (129, 140, 248);
    pub const INDIGO_500: (u8, u8, u8) = (99, 102, 241);
    pub const INDIGO_600: (u8, u8, u8) = (79, 70, 229);
    pub const INDIGO_700: (u8, u8, u8) = (67, 56, 202);
    pub const INDIGO_800: (u8, u8, u8) = (55, 48, 163);
    pub const INDIGO_900: (u8, u8, u8) = (49, 46, 129);
    pub const INDIGO_950: (u8, u8, u8) = (30, 27, 75);

    // Violet
    pub const VIOLET_50: (u8, u8, u8) = (245, 243, 255);
    pub const VIOLET_100: (u8, u8, u8) = (237, 233, 254);
    pub const VIOLET_200: (u8, u8, u8) = (221, 214, 254);
    pub const VIOLET_300: (u8, u8, u8) = (196, 181, 253);
    pub const VIOLET_400: (u8, u8, u8) = (167, 139, 250);
    pub const VIOLET_500: (u8, u8, u8) = (139, 92, 246);
    pub const VIOLET_600: (u8, u8, u8) = (124, 58, 237);
    pub const VIOLET_700: (u8, u8, u8) = (109, 40, 217);
    pub const VIOLET_800: (u8, u8, u8) = (91, 33, 182);
    pub const VIOLET_900: (u8, u8, u8) = (76, 29, 149);
    pub const VIOLET_950: (u8, u8, u8) = (46, 16, 101);

    // Purple
    pub const PURPLE_50: (u8, u8, u8) = (250, 245, 255);
    pub const PURPLE_100: (u8, u8, u8) = (243, 232, 255);
    pub const PURPLE_200: (u8, u8, u8) = (233, 213, 255);
    pub const PURPLE_300: (u8, u8, u8) = (216, 180, 254);
    pub const PURPLE_400: (u8, u8, u8) = (192, 132, 252);
    pub const PURPLE_500: (u8, u8, u8) = (168, 85, 247);
    pub const PURPLE_600: (u8, u8, u8) = (147, 51, 234);
    pub const PURPLE_700: (u8, u8, u8) = (126, 34, 206);
    pub const PURPLE_800: (u8, u8, u8) = (107, 33, 168);
    pub const PURPLE_900: (u8, u8, u8) = (88, 28, 135);
    pub const PURPLE_950: (u8, u8, u8) = (59, 7, 100);

    // Fuchsia
    pub const FUCHSIA_50: (u8, u8, u8) = (253, 244, 255);
    pub const FUCHSIA_100: (u8, u8, u8) = (250, 232, 255);
    pub const FUCHSIA_200: (u8, u8, u8) = (245, 208, 254);
    pub const FUCHSIA_300: (u8, u8, u8) = (240, 171, 252);
    pub const FUCHSIA_400: (u8, u8, u8) = (232, 121, 249);
    pub const FUCHSIA_500: (u8, u8, u8) = (217, 70, 239);
    pub const FUCHSIA_600: (u8, u8, u8) = (192, 38, 211);
    pub const FUCHSIA_700: (u8, u8, u8) = (162, 28, 175);
    pub const FUCHSIA_800: (u8, u8, u8) = (134, 25, 143);
    pub const FUCHSIA_900: (u8, u8, u8) = (112, 26, 117);
    pub const FUCHSIA_950: (u8, u8, u8) = (74, 4, 78);

    // Pink
    pub const PINK_50: (u8, u8, u8) = (253, 242, 248);
    pub const PINK_100: (u8, u8, u8) = (252, 231, 243);
    pub const PINK_200: (u8, u8, u8) = (251, 207, 232);
    pub const PINK_300: (u8, u8, u8) = (249, 168, 212);
    pub const PINK_400: (u8, u8, u8) = (244, 114, 182);
    pub const PINK_500: (u8, u8, u8) = (236, 72, 153);
    pub const PINK_600: (u8, u8, u8) = (219, 39, 119);
    pub const PINK_700: (u8, u8, u8) = (190, 24, 93);
    pub const PINK_800: (u8, u8, u8) = (157, 23, 77);
    pub const PINK_900: (u8, u8, u8) = (131, 24, 67);
    pub const PINK_950: (u8, u8, u8) = (80, 7, 36);

    // Rose
    pub const ROSE_50: (u8, u8, u8) = (255, 241, 242);
    pub const ROSE_100: (u8, u8, u8) = (255, 228, 230);
    pub const ROSE_200: (u8, u8, u8) = (254, 205, 211);
    pub const ROSE_300: (u8, u8, u8) = (253, 164, 175);
    pub const ROSE_400: (u8, u8, u8) = (251, 113, 133);
    pub const ROSE_500: (u8, u8, u8) = (244, 63, 94);
    pub const ROSE_600: (u8, u8, u8) = (225, 29, 72);
    pub const ROSE_700: (u8, u8, u8) = (190, 18, 60);
    pub const ROSE_800: (u8, u8, u8) = (159, 18, 57);
    pub const ROSE_900: (u8, u8, u8) = (136, 19, 55);
    pub const ROSE_950: (u8, u8, u8) = (76, 5, 25);
}

/// Helper to get color by name and shade
pub fn get_color(color: &str, shade: &str) -> Option<(u8, u8, u8)> {
    match (color, shade) {
        // Slate
        ("slate", "50") => Some(Colors::SLATE_50),
        ("slate", "100") => Some(Colors::SLATE_100),
        ("slate", "200") => Some(Colors::SLATE_200),
        ("slate", "300") => Some(Colors::SLATE_300),
        ("slate", "400") => Some(Colors::SLATE_400),
        ("slate", "500") => Some(Colors::SLATE_500),
        ("slate", "600") => Some(Colors::SLATE_600),
        ("slate", "700") => Some(Colors::SLATE_700),
        ("slate", "800") => Some(Colors::SLATE_800),
        ("slate", "900") => Some(Colors::SLATE_900),
        ("slate", "950") => Some(Colors::SLATE_950),
        
        // Gray
        ("gray", "50") => Some(Colors::GRAY_50),
        ("gray", "100") => Some(Colors::GRAY_100),
        ("gray", "200") => Some(Colors::GRAY_200),
        ("gray", "300") => Some(Colors::GRAY_300),
        ("gray", "400") => Some(Colors::GRAY_400),
        ("gray", "500") => Some(Colors::GRAY_500),
        ("gray", "600") => Some(Colors::GRAY_600),
        ("gray", "700") => Some(Colors::GRAY_700),
        ("gray", "800") => Some(Colors::GRAY_800),
        ("gray", "900") => Some(Colors::GRAY_900),
        ("gray", "950") => Some(Colors::GRAY_950),
        
        // Zinc  
        ("zinc", "50") => Some(Colors::ZINC_50),
        ("zinc", "100") => Some(Colors::ZINC_100),
        ("zinc", "200") => Some(Colors::ZINC_200),
        ("zinc", "300") => Some(Colors::ZINC_300),
        ("zinc", "400") => Some(Colors::ZINC_400),
        ("zinc", "500") => Some(Colors::ZINC_500),
        ("zinc", "600") => Some(Colors::ZINC_600),
        ("zinc", "700") => Some(Colors::ZINC_700),
        ("zinc", "800") => Some(Colors::ZINC_800),
        ("zinc", "900") => Some(Colors::ZINC_900),
        ("zinc", "950") => Some(Colors::ZINC_950),
        
        // Neutral
        ("neutral", "50") => Some(Colors::NEUTRAL_50),
        ("neutral", "100") => Some(Colors::NEUTRAL_100),
        ("neutral", "200") => Some(Colors::NEUTRAL_200),
        ("neutral", "300") => Some(Colors::NEUTRAL_300),
        ("neutral", "400") => Some(Colors::NEUTRAL_400),
        ("neutral", "500") => Some(Colors::NEUTRAL_500),
        ("neutral", "600") => Some(Colors::NEUTRAL_600),
        ("neutral", "700") => Some(Colors::NEUTRAL_700),
        ("neutral", "800") => Some(Colors::NEUTRAL_800),
        ("neutral", "900") => Some(Colors::NEUTRAL_900),
        ("neutral", "950") => Some(Colors::NEUTRAL_950),
        
        // Stone
        ("stone", "50") => Some(Colors::STONE_50),
        ("stone", "100") => Some(Colors::STONE_100),
        ("stone", "200") => Some(Colors::STONE_200),
        ("stone", "300") => Some(Colors::STONE_300),
        ("stone", "400") => Some(Colors::STONE_400),
        ("stone", "500") => Some(Colors::STONE_500),
        ("stone", "600") => Some(Colors::STONE_600),
        ("stone", "700") => Some(Colors::STONE_700),
        ("stone", "800") => Some(Colors::STONE_800),
        ("stone", "900") => Some(Colors::STONE_900),
        ("stone", "950") => Some(Colors::STONE_950),
        
        // Red
        ("red", "50") => Some(Colors::RED_50),
        ("red", "100") => Some(Colors::RED_100),
        ("red", "200") => Some(Colors::RED_200),
        ("red", "300") => Some(Colors::RED_300),
        ("red", "400") => Some(Colors::RED_400),
        ("red", "500") => Some(Colors::RED_500),
        ("red", "600") => Some(Colors::RED_600),
        ("red", "700") => Some(Colors::RED_700),
        ("red", "800") => Some(Colors::RED_800),
        ("red", "900") => Some(Colors::RED_900),
        ("red", "950") => Some(Colors::RED_950),
        
        // Orange
        ("orange", "50") => Some(Colors::ORANGE_50),
        ("orange", "100") => Some(Colors::ORANGE_100),
        ("orange", "200") => Some(Colors::ORANGE_200),
        ("orange", "300") => Some(Colors::ORANGE_300),
        ("orange", "400") => Some(Colors::ORANGE_400),
        ("orange", "500") => Some(Colors::ORANGE_500),
        ("orange", "600") => Some(Colors::ORANGE_600),
        ("orange", "700") => Some(Colors::ORANGE_700),
        ("orange", "800") => Some(Colors::ORANGE_800),
        ("orange", "900") => Some(Colors::ORANGE_900),
        ("orange", "950") => Some(Colors::ORANGE_950),
        
        // Amber
        ("amber", "50") => Some(Colors::AMBER_50),
        ("amber", "100") => Some(Colors::AMBER_100),
        ("amber", "200") => Some(Colors::AMBER_200),
        ("amber", "300") => Some(Colors::AMBER_300),
        ("amber", "400") => Some(Colors::AMBER_400),
        ("amber", "500") => Some(Colors::AMBER_500),
        ("amber", "600") => Some(Colors::AMBER_600),
        ("amber", "700") => Some(Colors::AMBER_700),
        ("amber", "800") => Some(Colors::AMBER_800),
        ("amber", "900") => Some(Colors::AMBER_900),
        ("amber", "950") => Some(Colors::AMBER_950),
        
        // Yellow
        ("yellow", "50") => Some(Colors::YELLOW_50),
        ("yellow", "100") => Some(Colors::YELLOW_100),
        ("yellow", "200") => Some(Colors::YELLOW_200),
        ("yellow", "300") => Some(Colors::YELLOW_300),
        ("yellow", "400") => Some(Colors::YELLOW_400),
        ("yellow", "500") => Some(Colors::YELLOW_500),
        ("yellow", "600") => Some(Colors::YELLOW_600),
        ("yellow", "700") => Some(Colors::YELLOW_700),
        ("yellow", "800") => Some(Colors::YELLOW_800),
        ("yellow", "900") => Some(Colors::YELLOW_900),
        ("yellow", "950") => Some(Colors::YELLOW_950),
        
        // Lime
        ("lime", "50") => Some(Colors::LIME_50),
        ("lime", "100") => Some(Colors::LIME_100),
        ("lime", "200") => Some(Colors::LIME_200),
        ("lime", "300") => Some(Colors::LIME_300),
        ("lime", "400") => Some(Colors::LIME_400),
        ("lime", "500") => Some(Colors::LIME_500),
        ("lime", "600") => Some(Colors::LIME_600),
        ("lime", "700") => Some(Colors::LIME_700),
        ("lime", "800") => Some(Colors::LIME_800),
        ("lime", "900") => Some(Colors::LIME_900),
        ("lime", "950") => Some(Colors::LIME_950),
        
        // Green
        ("green", "50") => Some(Colors::GREEN_50),
        ("green", "100") => Some(Colors::GREEN_100),
        ("green", "200") => Some(Colors::GREEN_200),
        ("green", "300") => Some(Colors::GREEN_300),
        ("green", "400") => Some(Colors::GREEN_400),
        ("green", "500") => Some(Colors::GREEN_500),
        ("green", "600") => Some(Colors::GREEN_600),
        ("green", "700") => Some(Colors::GREEN_700),
        ("green", "800") => Some(Colors::GREEN_800),
        ("green", "900") => Some(Colors::GREEN_900),
        ("green", "950") => Some(Colors::GREEN_950),
        
        // Emerald
        ("emerald", "50") => Some(Colors::EMERALD_50),
        ("emerald", "100") => Some(Colors::EMERALD_100),
        ("emerald", "200") => Some(Colors::EMERALD_200),
        ("emerald", "300") => Some(Colors::EMERALD_300),
        ("emerald", "400") => Some(Colors::EMERALD_400),
        ("emerald", "500") => Some(Colors::EMERALD_500),
        ("emerald", "600") => Some(Colors::EMERALD_600),
        ("emerald", "700") => Some(Colors::EMERALD_700),
        ("emerald", "800") => Some(Colors::EMERALD_800),
        ("emerald", "900") => Some(Colors::EMERALD_900),
        ("emerald", "950") => Some(Colors::EMERALD_950),
        
        // Teal
        ("teal", "50") => Some(Colors::TEAL_50),
        ("teal", "100") => Some(Colors::TEAL_100),
        ("teal", "200") => Some(Colors::TEAL_200),
        ("teal", "300") => Some(Colors::TEAL_300),
        ("teal", "400") => Some(Colors::TEAL_400),
        ("teal", "500") => Some(Colors::TEAL_500),
        ("teal", "600") => Some(Colors::TEAL_600),
        ("teal", "700") => Some(Colors::TEAL_700),
        ("teal", "800") => Some(Colors::TEAL_800),
        ("teal", "900") => Some(Colors::TEAL_900),
        ("teal", "950") => Some(Colors::TEAL_950),
        
        // Cyan
        ("cyan", "50") => Some(Colors::CYAN_50),
        ("cyan", "100") => Some(Colors::CYAN_100),
        ("cyan", "200") => Some(Colors::CYAN_200),
        ("cyan", "300") => Some(Colors::CYAN_300),
        ("cyan", "400") => Some(Colors::CYAN_400),
        ("cyan", "500") => Some(Colors::CYAN_500),
        ("cyan", "600") => Some(Colors::CYAN_600),
        ("cyan", "700") => Some(Colors::CYAN_700),
        ("cyan", "800") => Some(Colors::CYAN_800),
        ("cyan", "900") => Some(Colors::CYAN_900),
        ("cyan", "950") => Some(Colors::CYAN_950),
        
        // Sky
        ("sky", "50") => Some(Colors::SKY_50),
        ("sky", "100") => Some(Colors::SKY_100),
        ("sky", "200") => Some(Colors::SKY_200),
        ("sky", "300") => Some(Colors::SKY_300),
        ("sky", "400") => Some(Colors::SKY_400),
        ("sky", "500") => Some(Colors::SKY_500),
        ("sky", "600") => Some(Colors::SKY_600),
        ("sky", "700") => Some(Colors::SKY_700),
        ("sky", "800") => Some(Colors::SKY_800),
        ("sky", "900") => Some(Colors::SKY_900),
        ("sky", "950") => Some(Colors::SKY_950),
        
        // Blue
        ("blue", "50") => Some(Colors::BLUE_50),
        ("blue", "100") => Some(Colors::BLUE_100),
        ("blue", "200") => Some(Colors::BLUE_200),
        ("blue", "300") => Some(Colors::BLUE_300),
        ("blue", "400") => Some(Colors::BLUE_400),
        ("blue", "500") => Some(Colors::BLUE_500),
        ("blue", "600") => Some(Colors::BLUE_600),
        ("blue", "700") => Some(Colors::BLUE_700),
        ("blue", "800") => Some(Colors::BLUE_800),
        ("blue", "900") => Some(Colors::BLUE_900),
        ("blue", "950") => Some(Colors::BLUE_950),
        
        // Indigo
        ("indigo", "50") => Some(Colors::INDIGO_50),
        ("indigo", "100") => Some(Colors::INDIGO_100),
        ("indigo", "200") => Some(Colors::INDIGO_200),
        ("indigo", "300") => Some(Colors::INDIGO_300),
        ("indigo", "400") => Some(Colors::INDIGO_400),
        ("indigo", "500") => Some(Colors::INDIGO_500),
        ("indigo", "600") => Some(Colors::INDIGO_600),
        ("indigo", "700") => Some(Colors::INDIGO_700),
        ("indigo", "800") => Some(Colors::INDIGO_800),
        ("indigo", "900") => Some(Colors::INDIGO_900),
        ("indigo", "950") => Some(Colors::INDIGO_950),
        
        // Violet
        ("violet", "50") => Some(Colors::VIOLET_50),
        ("violet", "100") => Some(Colors::VIOLET_100),
        ("violet", "200") => Some(Colors::VIOLET_200),
        ("violet", "300") => Some(Colors::VIOLET_300),
        ("violet", "400") => Some(Colors::VIOLET_400),
        ("violet", "500") => Some(Colors::VIOLET_500),
        ("violet", "600") => Some(Colors::VIOLET_600),
        ("violet", "700") => Some(Colors::VIOLET_700),
        ("violet", "800") => Some(Colors::VIOLET_800),
        ("violet", "900") => Some(Colors::VIOLET_900),
        ("violet", "950") => Some(Colors::VIOLET_950),
        
        // Purple
        ("purple", "50") => Some(Colors::PURPLE_50),
        ("purple", "100") => Some(Colors::PURPLE_100),
        ("purple", "200") => Some(Colors::PURPLE_200),
        ("purple", "300") => Some(Colors::PURPLE_300),
        ("purple", "400") => Some(Colors::PURPLE_400),
        ("purple", "500") => Some(Colors::PURPLE_500),
        ("purple", "600") => Some(Colors::PURPLE_600),
        ("purple", "700") => Some(Colors::PURPLE_700),
        ("purple", "800") => Some(Colors::PURPLE_800),
        ("purple", "900") => Some(Colors::PURPLE_900),
        ("purple", "950") => Some(Colors::PURPLE_950),
        
        // Fuchsia
        ("fuchsia", "50") => Some(Colors::FUCHSIA_50),
        ("fuchsia", "100") => Some(Colors::FUCHSIA_100),
        ("fuchsia", "200") => Some(Colors::FUCHSIA_200),
        ("fuchsia", "300") => Some(Colors::FUCHSIA_300),
        ("fuchsia", "400") => Some(Colors::FUCHSIA_400),
        ("fuchsia", "500") => Some(Colors::FUCHSIA_500),
        ("fuchsia", "600") => Some(Colors::FUCHSIA_600),
        ("fuchsia", "700") => Some(Colors::FUCHSIA_700),
        ("fuchsia", "800") => Some(Colors::FUCHSIA_800),
        ("fuchsia", "900") => Some(Colors::FUCHSIA_900),
        ("fuchsia", "950") => Some(Colors::FUCHSIA_950),
        
        // Pink
        ("pink", "50") => Some(Colors::PINK_50),
        ("pink", "100") => Some(Colors::PINK_100),
        ("pink", "200") => Some(Colors::PINK_200),
        ("pink", "300") => Some(Colors::PINK_300),
        ("pink", "400") => Some(Colors::PINK_400),
        ("pink", "500") => Some(Colors::PINK_500),
        ("pink", "600") => Some(Colors::PINK_600),
        ("pink", "700") => Some(Colors::PINK_700),
        ("pink", "800") => Some(Colors::PINK_800),
        ("pink", "900") => Some(Colors::PINK_900),
        ("pink", "950") => Some(Colors::PINK_950),
        
        // Rose
        ("rose", "50") => Some(Colors::ROSE_50),
        ("rose", "100") => Some(Colors::ROSE_100),
        ("rose", "200") => Some(Colors::ROSE_200),
        ("rose", "300") => Some(Colors::ROSE_300),
        ("rose", "400") => Some(Colors::ROSE_400),
        ("rose", "500") => Some(Colors::ROSE_500),
        ("rose", "600") => Some(Colors::ROSE_600),
        ("rose", "700") => Some(Colors::ROSE_700),
        ("rose", "800") => Some(Colors::ROSE_800),
        ("rose", "900") => Some(Colors::ROSE_900),
        ("rose", "950") => Some(Colors::ROSE_950),
        
        _ => None,
    }
}