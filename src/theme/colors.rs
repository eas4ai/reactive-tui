/// CSS color palette
pub struct Colors;

impl Colors {
    // Slate color palette
    /// Lightest slate color (248, 250, 252)
    pub const SLATE_50: (u8, u8, u8) = (248, 250, 252);
    /// Very light slate color (241, 245, 249)
    pub const SLATE_100: (u8, u8, u8) = (241, 245, 249);
    /// Light slate color (226, 232, 240)
    pub const SLATE_200: (u8, u8, u8) = (226, 232, 240);
    /// Light-medium slate color (203, 213, 225)
    pub const SLATE_300: (u8, u8, u8) = (203, 213, 225);
    /// Medium slate color (148, 163, 184)
    pub const SLATE_400: (u8, u8, u8) = (148, 163, 184);
    /// Medium slate color (100, 116, 139)
    pub const SLATE_500: (u8, u8, u8) = (100, 116, 139);
    /// Medium-dark slate color (71, 85, 105)
    pub const SLATE_600: (u8, u8, u8) = (71, 85, 105);
    /// Dark slate color (51, 65, 85)
    pub const SLATE_700: (u8, u8, u8) = (51, 65, 85);
    /// Very dark slate color (30, 41, 59)
    pub const SLATE_800: (u8, u8, u8) = (30, 41, 59);
    /// Darkest slate color (15, 23, 42)
    pub const SLATE_900: (u8, u8, u8) = (15, 23, 42);
    /// Ultra dark slate color (2, 6, 23)
    pub const SLATE_950: (u8, u8, u8) = (2, 6, 23);

    // Gray color palette
    /// Lightest gray color (249, 250, 251)
    pub const GRAY_50: (u8, u8, u8) = (249, 250, 251);
    /// Very light gray color (243, 244, 246)
    pub const GRAY_100: (u8, u8, u8) = (243, 244, 246);
    /// Light gray color (229, 231, 235)
    pub const GRAY_200: (u8, u8, u8) = (229, 231, 235);
    /// Light-medium gray color (209, 213, 219)
    pub const GRAY_300: (u8, u8, u8) = (209, 213, 219);
    /// Medium gray color (156, 163, 175)
    pub const GRAY_400: (u8, u8, u8) = (156, 163, 175);
    /// Medium gray color (107, 114, 128)
    pub const GRAY_500: (u8, u8, u8) = (107, 114, 128);
    /// Medium-dark gray color (75, 85, 99)
    pub const GRAY_600: (u8, u8, u8) = (75, 85, 99);
    /// Dark gray color (55, 65, 81)
    pub const GRAY_700: (u8, u8, u8) = (55, 65, 81);
    /// Very dark gray color (31, 41, 55)
    pub const GRAY_800: (u8, u8, u8) = (31, 41, 55);
    /// Darkest gray color (17, 24, 39)
    pub const GRAY_900: (u8, u8, u8) = (17, 24, 39);
    /// Ultra dark gray color (3, 7, 18)
    pub const GRAY_950: (u8, u8, u8) = (3, 7, 18);

    // Zinc color palette
    /// Lightest zinc color (250, 250, 250)
    pub const ZINC_50: (u8, u8, u8) = (250, 250, 250);
    /// Very light zinc color (244, 244, 245)
    pub const ZINC_100: (u8, u8, u8) = (244, 244, 245);
    /// Light zinc color (228, 228, 231)
    pub const ZINC_200: (u8, u8, u8) = (228, 228, 231);
    /// Light-medium zinc color (212, 212, 216)
    pub const ZINC_300: (u8, u8, u8) = (212, 212, 216);
    /// Medium zinc color (161, 161, 170)
    pub const ZINC_400: (u8, u8, u8) = (161, 161, 170);
    /// Medium zinc color (113, 113, 122)
    pub const ZINC_500: (u8, u8, u8) = (113, 113, 122);
    /// Medium-dark zinc color (82, 82, 91)
    pub const ZINC_600: (u8, u8, u8) = (82, 82, 91);
    /// Dark zinc color (63, 63, 70)
    pub const ZINC_700: (u8, u8, u8) = (63, 63, 70);
    /// Very dark zinc color (39, 39, 42)
    pub const ZINC_800: (u8, u8, u8) = (39, 39, 42);
    /// Darkest zinc color (24, 24, 27)
    pub const ZINC_900: (u8, u8, u8) = (24, 24, 27);
    /// Ultra dark zinc color (9, 9, 11)
    pub const ZINC_950: (u8, u8, u8) = (9, 9, 11);

    // Neutral color palette
    /// Lightest neutral color (250, 250, 250)
    pub const NEUTRAL_50: (u8, u8, u8) = (250, 250, 250);
    /// Very light neutral color (245, 245, 245)
    pub const NEUTRAL_100: (u8, u8, u8) = (245, 245, 245);
    /// Light neutral color (229, 229, 229)
    pub const NEUTRAL_200: (u8, u8, u8) = (229, 229, 229);
    /// Light-medium neutral color (212, 212, 212)
    pub const NEUTRAL_300: (u8, u8, u8) = (212, 212, 212);
    /// Medium neutral color (163, 163, 163)
    pub const NEUTRAL_400: (u8, u8, u8) = (163, 163, 163);
    /// Medium neutral color (115, 115, 115)
    pub const NEUTRAL_500: (u8, u8, u8) = (115, 115, 115);
    /// Medium-dark neutral color (82, 82, 82)
    pub const NEUTRAL_600: (u8, u8, u8) = (82, 82, 82);
    /// Dark neutral color (64, 64, 64)
    pub const NEUTRAL_700: (u8, u8, u8) = (64, 64, 64);
    /// Very dark neutral color (38, 38, 38)
    pub const NEUTRAL_800: (u8, u8, u8) = (38, 38, 38);
    /// Darkest neutral color (23, 23, 23)
    pub const NEUTRAL_900: (u8, u8, u8) = (23, 23, 23);
    /// Ultra dark neutral color (10, 10, 10)
    pub const NEUTRAL_950: (u8, u8, u8) = (10, 10, 10);

    // Stone color palette
    /// Lightest stone color (250, 250, 249)
    pub const STONE_50: (u8, u8, u8) = (250, 250, 249);
    /// Very light stone color (245, 245, 244)
    pub const STONE_100: (u8, u8, u8) = (245, 245, 244);
    /// Light stone color (231, 229, 228)
    pub const STONE_200: (u8, u8, u8) = (231, 229, 228);
    /// Light-medium stone color (214, 211, 209)
    pub const STONE_300: (u8, u8, u8) = (214, 211, 209);
    /// Medium stone color (168, 162, 158)
    pub const STONE_400: (u8, u8, u8) = (168, 162, 158);
    /// Medium stone color (120, 113, 108)
    pub const STONE_500: (u8, u8, u8) = (120, 113, 108);
    /// Medium-dark stone color (87, 83, 78)
    pub const STONE_600: (u8, u8, u8) = (87, 83, 78);
    /// Dark stone color (68, 64, 60)
    pub const STONE_700: (u8, u8, u8) = (68, 64, 60);
    /// Very dark stone color (41, 37, 36)
    pub const STONE_800: (u8, u8, u8) = (41, 37, 36);
    /// Darkest stone color (28, 25, 23)
    pub const STONE_900: (u8, u8, u8) = (28, 25, 23);
    /// Ultra dark stone color (12, 10, 9)
    pub const STONE_950: (u8, u8, u8) = (12, 10, 9);

    // Red color palette
    /// Lightest red color (254, 242, 242)
    pub const RED_50: (u8, u8, u8) = (254, 242, 242);
    /// Very light red color (254, 226, 226)
    pub const RED_100: (u8, u8, u8) = (254, 226, 226);
    /// Light red color (254, 202, 202)
    pub const RED_200: (u8, u8, u8) = (254, 202, 202);
    /// Light-medium red color (252, 165, 165)
    pub const RED_300: (u8, u8, u8) = (252, 165, 165);
    /// Medium red color (248, 113, 113)
    pub const RED_400: (u8, u8, u8) = (248, 113, 113);
    /// Medium red color (239, 68, 68)
    pub const RED_500: (u8, u8, u8) = (239, 68, 68);
    /// Medium-dark red color (220, 38, 38)
    pub const RED_600: (u8, u8, u8) = (220, 38, 38);
    /// Dark red color (185, 28, 28)
    pub const RED_700: (u8, u8, u8) = (185, 28, 28);
    /// Very dark red color (153, 27, 27)
    pub const RED_800: (u8, u8, u8) = (153, 27, 27);
    /// Darkest red color (127, 29, 29)
    pub const RED_900: (u8, u8, u8) = (127, 29, 29);
    /// Ultra dark red color (69, 10, 10)
    pub const RED_950: (u8, u8, u8) = (69, 10, 10);

    // Orange color palette
    /// Lightest orange color (255, 247, 237)
    pub const ORANGE_50: (u8, u8, u8) = (255, 247, 237);
    /// Very light orange color (255, 237, 213)
    pub const ORANGE_100: (u8, u8, u8) = (255, 237, 213);
    /// Light orange color (254, 215, 170)
    pub const ORANGE_200: (u8, u8, u8) = (254, 215, 170);
    /// Light-medium orange color (253, 186, 116)
    pub const ORANGE_300: (u8, u8, u8) = (253, 186, 116);
    /// Medium orange color (251, 146, 60)
    pub const ORANGE_400: (u8, u8, u8) = (251, 146, 60);
    /// Medium orange color (249, 115, 22)
    pub const ORANGE_500: (u8, u8, u8) = (249, 115, 22);
    /// Medium-dark orange color (234, 88, 12)
    pub const ORANGE_600: (u8, u8, u8) = (234, 88, 12);
    /// Dark orange color (194, 65, 12)
    pub const ORANGE_700: (u8, u8, u8) = (194, 65, 12);
    /// Very dark orange color (154, 52, 18)
    pub const ORANGE_800: (u8, u8, u8) = (154, 52, 18);
    /// Darkest orange color (124, 45, 18)
    pub const ORANGE_900: (u8, u8, u8) = (124, 45, 18);
    /// Ultra dark orange color (67, 20, 7)
    pub const ORANGE_950: (u8, u8, u8) = (67, 20, 7);

    // Amber color palette
    /// Lightest amber color (255, 251, 235)
    pub const AMBER_50: (u8, u8, u8) = (255, 251, 235);
    /// Very light amber color (254, 243, 199)
    pub const AMBER_100: (u8, u8, u8) = (254, 243, 199);
    /// Light amber color (253, 230, 138)
    pub const AMBER_200: (u8, u8, u8) = (253, 230, 138);
    /// Light-medium amber color (252, 211, 77)
    pub const AMBER_300: (u8, u8, u8) = (252, 211, 77);
    /// Medium amber color (251, 191, 36)
    pub const AMBER_400: (u8, u8, u8) = (251, 191, 36);
    /// Medium amber color (245, 158, 11)
    pub const AMBER_500: (u8, u8, u8) = (245, 158, 11);
    /// Medium-dark amber color (217, 119, 6)
    pub const AMBER_600: (u8, u8, u8) = (217, 119, 6);
    /// Dark amber color (180, 83, 9)
    pub const AMBER_700: (u8, u8, u8) = (180, 83, 9);
    /// Very dark amber color (146, 64, 14)
    pub const AMBER_800: (u8, u8, u8) = (146, 64, 14);
    /// Darkest amber color (120, 53, 15)
    pub const AMBER_900: (u8, u8, u8) = (120, 53, 15);
    /// Ultra dark amber color (69, 26, 3)
    pub const AMBER_950: (u8, u8, u8) = (69, 26, 3);

    // Yellow color palette
    /// Lightest yellow color (254, 252, 232)
    pub const YELLOW_50: (u8, u8, u8) = (254, 252, 232);
    /// Very light yellow color (254, 249, 195)
    pub const YELLOW_100: (u8, u8, u8) = (254, 249, 195);
    /// Light yellow color (254, 240, 138)
    pub const YELLOW_200: (u8, u8, u8) = (254, 240, 138);
    /// Light-medium yellow color (253, 224, 71)
    pub const YELLOW_300: (u8, u8, u8) = (253, 224, 71);
    /// Medium yellow color (250, 204, 21)
    pub const YELLOW_400: (u8, u8, u8) = (250, 204, 21);
    /// Medium yellow color (234, 179, 8)
    pub const YELLOW_500: (u8, u8, u8) = (234, 179, 8);
    /// Medium-dark yellow color (202, 138, 4)
    pub const YELLOW_600: (u8, u8, u8) = (202, 138, 4);
    /// Dark yellow color (161, 98, 7)
    pub const YELLOW_700: (u8, u8, u8) = (161, 98, 7);
    /// Very dark yellow color (133, 77, 14)
    pub const YELLOW_800: (u8, u8, u8) = (133, 77, 14);
    /// Darkest yellow color (113, 63, 18)
    pub const YELLOW_900: (u8, u8, u8) = (113, 63, 18);
    /// Ultra dark yellow color (66, 32, 6)
    pub const YELLOW_950: (u8, u8, u8) = (66, 32, 6);

    // Lime color palette
    /// Lightest lime color (247, 254, 231)
    pub const LIME_50: (u8, u8, u8) = (247, 254, 231);
    /// Very light lime color (236, 252, 203)
    pub const LIME_100: (u8, u8, u8) = (236, 252, 203);
    /// Light lime color (217, 249, 157)
    pub const LIME_200: (u8, u8, u8) = (217, 249, 157);
    /// Light-medium lime color (190, 242, 100)
    pub const LIME_300: (u8, u8, u8) = (190, 242, 100);
    /// Medium lime color (163, 230, 53)
    pub const LIME_400: (u8, u8, u8) = (163, 230, 53);
    /// Medium lime color (132, 204, 22)
    pub const LIME_500: (u8, u8, u8) = (132, 204, 22);
    /// Medium-dark lime color (101, 163, 13)
    pub const LIME_600: (u8, u8, u8) = (101, 163, 13);
    /// Dark lime color (77, 124, 15)
    pub const LIME_700: (u8, u8, u8) = (77, 124, 15);
    /// Very dark lime color (63, 98, 18)
    pub const LIME_800: (u8, u8, u8) = (63, 98, 18);
    /// Darkest lime color (54, 83, 20)
    pub const LIME_900: (u8, u8, u8) = (54, 83, 20);
    /// Ultra dark lime color (26, 46, 5)
    pub const LIME_950: (u8, u8, u8) = (26, 46, 5);

    // Green color palette
    /// Lightest green color (240, 253, 244)
    pub const GREEN_50: (u8, u8, u8) = (240, 253, 244);
    /// Very light green color (220, 252, 231)
    pub const GREEN_100: (u8, u8, u8) = (220, 252, 231);
    /// Light green color (187, 247, 208)
    pub const GREEN_200: (u8, u8, u8) = (187, 247, 208);
    /// Light-medium green color (134, 239, 172)
    pub const GREEN_300: (u8, u8, u8) = (134, 239, 172);
    /// Medium green color (74, 222, 128)
    pub const GREEN_400: (u8, u8, u8) = (74, 222, 128);
    /// Medium green color (34, 197, 94)
    pub const GREEN_500: (u8, u8, u8) = (34, 197, 94);
    /// Medium-dark green color (22, 163, 74)
    pub const GREEN_600: (u8, u8, u8) = (22, 163, 74);
    /// Dark green color (21, 128, 61)
    pub const GREEN_700: (u8, u8, u8) = (21, 128, 61);
    /// Very dark green color (22, 101, 52)
    pub const GREEN_800: (u8, u8, u8) = (22, 101, 52);
    /// Darkest green color (20, 83, 45)
    pub const GREEN_900: (u8, u8, u8) = (20, 83, 45);
    /// Ultra dark green color (5, 46, 22)
    pub const GREEN_950: (u8, u8, u8) = (5, 46, 22);

    // Emerald color palette
    /// Lightest emerald color (236, 253, 245)
    pub const EMERALD_50: (u8, u8, u8) = (236, 253, 245);
    /// Very light emerald color (209, 250, 229)
    pub const EMERALD_100: (u8, u8, u8) = (209, 250, 229);
    /// Light emerald color (167, 243, 208)
    pub const EMERALD_200: (u8, u8, u8) = (167, 243, 208);
    /// Light-medium emerald color (110, 231, 183)
    pub const EMERALD_300: (u8, u8, u8) = (110, 231, 183);
    /// Medium emerald color (52, 211, 153)
    pub const EMERALD_400: (u8, u8, u8) = (52, 211, 153);
    /// Medium emerald color (16, 185, 129)
    pub const EMERALD_500: (u8, u8, u8) = (16, 185, 129);
    /// Medium-dark emerald color (5, 150, 105)
    pub const EMERALD_600: (u8, u8, u8) = (5, 150, 105);
    /// Dark emerald color (4, 120, 87)
    pub const EMERALD_700: (u8, u8, u8) = (4, 120, 87);
    /// Very dark emerald color (6, 95, 70)
    pub const EMERALD_800: (u8, u8, u8) = (6, 95, 70);
    /// Darkest emerald color (6, 78, 59)
    pub const EMERALD_900: (u8, u8, u8) = (6, 78, 59);
    /// Ultra dark emerald color (2, 44, 34)
    pub const EMERALD_950: (u8, u8, u8) = (2, 44, 34);

    // Teal color palette
    /// Lightest teal color (240, 253, 250)
    pub const TEAL_50: (u8, u8, u8) = (240, 253, 250);
    /// Very light teal color (204, 251, 241)
    pub const TEAL_100: (u8, u8, u8) = (204, 251, 241);
    /// Light teal color (153, 246, 228)
    pub const TEAL_200: (u8, u8, u8) = (153, 246, 228);
    /// Light-medium teal color (94, 234, 212)
    pub const TEAL_300: (u8, u8, u8) = (94, 234, 212);
    /// Medium teal color (45, 212, 191)
    pub const TEAL_400: (u8, u8, u8) = (45, 212, 191);
    /// Medium teal color (20, 184, 166)
    pub const TEAL_500: (u8, u8, u8) = (20, 184, 166);
    /// Medium-dark teal color (13, 148, 136)
    pub const TEAL_600: (u8, u8, u8) = (13, 148, 136);
    /// Dark teal color (15, 118, 110)
    pub const TEAL_700: (u8, u8, u8) = (15, 118, 110);
    /// Very dark teal color (17, 94, 89)
    pub const TEAL_800: (u8, u8, u8) = (17, 94, 89);
    /// Darkest teal color (19, 78, 74)
    pub const TEAL_900: (u8, u8, u8) = (19, 78, 74);
    /// Ultra dark teal color (4, 47, 46)
    pub const TEAL_950: (u8, u8, u8) = (4, 47, 46);

    // Cyan color palette
    /// Lightest cyan color (236, 254, 255)
    pub const CYAN_50: (u8, u8, u8) = (236, 254, 255);
    /// Very light cyan color (207, 250, 254)
    pub const CYAN_100: (u8, u8, u8) = (207, 250, 254);
    /// Light cyan color (165, 243, 252)
    pub const CYAN_200: (u8, u8, u8) = (165, 243, 252);
    /// Light-medium cyan color (103, 232, 249)
    pub const CYAN_300: (u8, u8, u8) = (103, 232, 249);
    /// Medium cyan color (34, 211, 238)
    pub const CYAN_400: (u8, u8, u8) = (34, 211, 238);
    /// Medium cyan color (6, 182, 212)
    pub const CYAN_500: (u8, u8, u8) = (6, 182, 212);
    /// Medium-dark cyan color (8, 145, 178)
    pub const CYAN_600: (u8, u8, u8) = (8, 145, 178);
    /// Dark cyan color (14, 116, 144)
    pub const CYAN_700: (u8, u8, u8) = (14, 116, 144);
    /// Very dark cyan color (21, 94, 117)
    pub const CYAN_800: (u8, u8, u8) = (21, 94, 117);
    /// Darkest cyan color (22, 78, 99)
    pub const CYAN_900: (u8, u8, u8) = (22, 78, 99);
    /// Ultra dark cyan color (8, 51, 68)
    pub const CYAN_950: (u8, u8, u8) = (8, 51, 68);

    // Sky color palette
    /// Lightest sky color (240, 249, 255)
    pub const SKY_50: (u8, u8, u8) = (240, 249, 255);
    /// Very light sky color (224, 242, 254)
    pub const SKY_100: (u8, u8, u8) = (224, 242, 254);
    /// Light sky color (186, 230, 253)
    pub const SKY_200: (u8, u8, u8) = (186, 230, 253);
    /// Light-medium sky color (125, 211, 252)
    pub const SKY_300: (u8, u8, u8) = (125, 211, 252);
    /// Medium sky color (56, 189, 248)
    pub const SKY_400: (u8, u8, u8) = (56, 189, 248);
    /// Medium sky color (14, 165, 233)
    pub const SKY_500: (u8, u8, u8) = (14, 165, 233);
    /// Medium-dark sky color (2, 132, 199)
    pub const SKY_600: (u8, u8, u8) = (2, 132, 199);
    /// Dark sky color (3, 105, 161)
    pub const SKY_700: (u8, u8, u8) = (3, 105, 161);
    /// Very dark sky color (7, 89, 133)
    pub const SKY_800: (u8, u8, u8) = (7, 89, 133);
    /// Darkest sky color (12, 74, 110)
    pub const SKY_900: (u8, u8, u8) = (12, 74, 110);
    /// Ultra dark sky color (8, 47, 73)
    pub const SKY_950: (u8, u8, u8) = (8, 47, 73);

    // Blue color palette
    /// Lightest blue color (239, 246, 255)
    pub const BLUE_50: (u8, u8, u8) = (239, 246, 255);
    /// Very light blue color (219, 234, 254)
    pub const BLUE_100: (u8, u8, u8) = (219, 234, 254);
    /// Light blue color (191, 219, 254)
    pub const BLUE_200: (u8, u8, u8) = (191, 219, 254);
    /// Light-medium blue color (147, 197, 253)
    pub const BLUE_300: (u8, u8, u8) = (147, 197, 253);
    /// Medium blue color (96, 165, 250)
    pub const BLUE_400: (u8, u8, u8) = (96, 165, 250);
    /// Medium blue color (59, 130, 246)
    pub const BLUE_500: (u8, u8, u8) = (59, 130, 246);
    /// Medium-dark blue color (37, 99, 235)
    pub const BLUE_600: (u8, u8, u8) = (37, 99, 235);
    /// Dark blue color (29, 78, 216)
    pub const BLUE_700: (u8, u8, u8) = (29, 78, 216);
    /// Very dark blue color (30, 64, 175)
    pub const BLUE_800: (u8, u8, u8) = (30, 64, 175);
    /// Darkest blue color (30, 58, 138)
    pub const BLUE_900: (u8, u8, u8) = (30, 58, 138);
    /// Ultra dark blue color (23, 37, 84)
    pub const BLUE_950: (u8, u8, u8) = (23, 37, 84);

    // Indigo color palette
    /// Lightest indigo color (238, 242, 255)
    pub const INDIGO_50: (u8, u8, u8) = (238, 242, 255);
    /// Very light indigo color (224, 231, 255)
    pub const INDIGO_100: (u8, u8, u8) = (224, 231, 255);
    /// Light indigo color (199, 210, 254)
    pub const INDIGO_200: (u8, u8, u8) = (199, 210, 254);
    /// Light-medium indigo color (165, 180, 252)
    pub const INDIGO_300: (u8, u8, u8) = (165, 180, 252);
    /// Medium indigo color (129, 140, 248)
    pub const INDIGO_400: (u8, u8, u8) = (129, 140, 248);
    /// Medium indigo color (99, 102, 241)
    pub const INDIGO_500: (u8, u8, u8) = (99, 102, 241);
    /// Medium-dark indigo color (79, 70, 229)
    pub const INDIGO_600: (u8, u8, u8) = (79, 70, 229);
    /// Dark indigo color (67, 56, 202)
    pub const INDIGO_700: (u8, u8, u8) = (67, 56, 202);
    /// Very dark indigo color (55, 48, 163)
    pub const INDIGO_800: (u8, u8, u8) = (55, 48, 163);
    /// Darkest indigo color (49, 46, 129)
    pub const INDIGO_900: (u8, u8, u8) = (49, 46, 129);
    /// Ultra dark indigo color (30, 27, 75)
    pub const INDIGO_950: (u8, u8, u8) = (30, 27, 75);

    // Violet color palette
    /// Lightest violet color (245, 243, 255)
    pub const VIOLET_50: (u8, u8, u8) = (245, 243, 255);
    /// Very light violet color (237, 233, 254)
    pub const VIOLET_100: (u8, u8, u8) = (237, 233, 254);
    /// Light violet color (221, 214, 254)
    pub const VIOLET_200: (u8, u8, u8) = (221, 214, 254);
    /// Light-medium violet color (196, 181, 253)
    pub const VIOLET_300: (u8, u8, u8) = (196, 181, 253);
    /// Medium violet color (167, 139, 250)
    pub const VIOLET_400: (u8, u8, u8) = (167, 139, 250);
    /// Medium violet color (139, 92, 246)
    pub const VIOLET_500: (u8, u8, u8) = (139, 92, 246);
    /// Medium-dark violet color (124, 58, 237)
    pub const VIOLET_600: (u8, u8, u8) = (124, 58, 237);
    /// Dark violet color (109, 40, 217)
    pub const VIOLET_700: (u8, u8, u8) = (109, 40, 217);
    /// Very dark violet color (91, 33, 182)
    pub const VIOLET_800: (u8, u8, u8) = (91, 33, 182);
    /// Darkest violet color (76, 29, 149)
    pub const VIOLET_900: (u8, u8, u8) = (76, 29, 149);
    /// Ultra dark violet color (46, 16, 101)
    pub const VIOLET_950: (u8, u8, u8) = (46, 16, 101);

    // Purple color palette
    /// Lightest purple color (250, 245, 255)
    pub const PURPLE_50: (u8, u8, u8) = (250, 245, 255);
    /// Very light purple color (243, 232, 255)
    pub const PURPLE_100: (u8, u8, u8) = (243, 232, 255);
    /// Light purple color (233, 213, 255)
    pub const PURPLE_200: (u8, u8, u8) = (233, 213, 255);
    /// Light-medium purple color (216, 180, 254)
    pub const PURPLE_300: (u8, u8, u8) = (216, 180, 254);
    /// Medium purple color (192, 132, 252)
    pub const PURPLE_400: (u8, u8, u8) = (192, 132, 252);
    /// Medium purple color (168, 85, 247)
    pub const PURPLE_500: (u8, u8, u8) = (168, 85, 247);
    /// Medium-dark purple color (147, 51, 234)
    pub const PURPLE_600: (u8, u8, u8) = (147, 51, 234);
    /// Dark purple color (126, 34, 206)
    pub const PURPLE_700: (u8, u8, u8) = (126, 34, 206);
    /// Very dark purple color (107, 33, 168)
    pub const PURPLE_800: (u8, u8, u8) = (107, 33, 168);
    /// Darkest purple color (88, 28, 135)
    pub const PURPLE_900: (u8, u8, u8) = (88, 28, 135);
    /// Ultra dark purple color (59, 7, 100)
    pub const PURPLE_950: (u8, u8, u8) = (59, 7, 100);

    // Fuchsia color palette
    /// Lightest fuchsia color (253, 244, 255)
    pub const FUCHSIA_50: (u8, u8, u8) = (253, 244, 255);
    /// Very light fuchsia color (250, 232, 255)
    pub const FUCHSIA_100: (u8, u8, u8) = (250, 232, 255);
    /// Light fuchsia color (245, 208, 254)
    pub const FUCHSIA_200: (u8, u8, u8) = (245, 208, 254);
    /// Light-medium fuchsia color (240, 171, 252)
    pub const FUCHSIA_300: (u8, u8, u8) = (240, 171, 252);
    /// Medium fuchsia color (232, 121, 249)
    pub const FUCHSIA_400: (u8, u8, u8) = (232, 121, 249);
    /// Medium fuchsia color (217, 70, 239)
    pub const FUCHSIA_500: (u8, u8, u8) = (217, 70, 239);
    /// Medium-dark fuchsia color (192, 38, 211)
    pub const FUCHSIA_600: (u8, u8, u8) = (192, 38, 211);
    /// Dark fuchsia color (162, 28, 175)
    pub const FUCHSIA_700: (u8, u8, u8) = (162, 28, 175);
    /// Very dark fuchsia color (134, 25, 143)
    pub const FUCHSIA_800: (u8, u8, u8) = (134, 25, 143);
    /// Darkest fuchsia color (112, 26, 117)
    pub const FUCHSIA_900: (u8, u8, u8) = (112, 26, 117);
    /// Ultra dark fuchsia color (74, 4, 78)
    pub const FUCHSIA_950: (u8, u8, u8) = (74, 4, 78);

    // Pink color palette
    /// Lightest pink color (253, 242, 248)
    pub const PINK_50: (u8, u8, u8) = (253, 242, 248);
    /// Very light pink color (252, 231, 243)
    pub const PINK_100: (u8, u8, u8) = (252, 231, 243);
    /// Light pink color (251, 207, 232)
    pub const PINK_200: (u8, u8, u8) = (251, 207, 232);
    /// Light-medium pink color (249, 168, 212)
    pub const PINK_300: (u8, u8, u8) = (249, 168, 212);
    /// Medium pink color (244, 114, 182)
    pub const PINK_400: (u8, u8, u8) = (244, 114, 182);
    /// Medium pink color (236, 72, 153)
    pub const PINK_500: (u8, u8, u8) = (236, 72, 153);
    /// Medium-dark pink color (219, 39, 119)
    pub const PINK_600: (u8, u8, u8) = (219, 39, 119);
    /// Dark pink color (190, 24, 93)
    pub const PINK_700: (u8, u8, u8) = (190, 24, 93);
    /// Very dark pink color (157, 23, 77)
    pub const PINK_800: (u8, u8, u8) = (157, 23, 77);
    /// Darkest pink color (131, 24, 67)
    pub const PINK_900: (u8, u8, u8) = (131, 24, 67);
    /// Ultra dark pink color (80, 7, 36)
    pub const PINK_950: (u8, u8, u8) = (80, 7, 36);

    // Rose color palette
    /// Lightest rose color (255, 241, 242)
    pub const ROSE_50: (u8, u8, u8) = (255, 241, 242);
    /// Very light rose color (255, 228, 230)
    pub const ROSE_100: (u8, u8, u8) = (255, 228, 230);
    /// Light rose color (254, 205, 211)
    pub const ROSE_200: (u8, u8, u8) = (254, 205, 211);
    /// Light-medium rose color (253, 164, 175)
    pub const ROSE_300: (u8, u8, u8) = (253, 164, 175);
    /// Medium rose color (251, 113, 133)
    pub const ROSE_400: (u8, u8, u8) = (251, 113, 133);
    /// Medium rose color (244, 63, 94)
    pub const ROSE_500: (u8, u8, u8) = (244, 63, 94);
    /// Medium-dark rose color (225, 29, 72)
    pub const ROSE_600: (u8, u8, u8) = (225, 29, 72);
    /// Dark rose color (190, 18, 60)
    pub const ROSE_700: (u8, u8, u8) = (190, 18, 60);
    /// Very dark rose color (159, 18, 57)
    pub const ROSE_800: (u8, u8, u8) = (159, 18, 57);
    /// Darkest rose color (136, 19, 55)
    pub const ROSE_900: (u8, u8, u8) = (136, 19, 55);
    /// Ultra dark rose color (76, 5, 25)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        // Test that color constants are defined and have valid RGB values
        assert_eq!(Colors::SLATE_50, (248, 250, 252));
        assert_eq!(Colors::SLATE_500, (100, 116, 139));
        assert_eq!(Colors::SLATE_950, (2, 6, 23));

        assert_eq!(Colors::RED_50, (254, 242, 242));
        assert_eq!(Colors::RED_500, (239, 68, 68));
        assert_eq!(Colors::RED_950, (69, 10, 10));

        assert_eq!(Colors::BLUE_50, (239, 246, 255));
        assert_eq!(Colors::BLUE_500, (59, 130, 246));
        assert_eq!(Colors::BLUE_950, (23, 37, 84));
    }

    #[test]
    fn test_color_progression() {
        // Test that colors get darker as the number increases
        let slate_colors = [
            Colors::SLATE_50,
            Colors::SLATE_100,
            Colors::SLATE_200,
            Colors::SLATE_300,
            Colors::SLATE_400,
            Colors::SLATE_500,
            Colors::SLATE_600,
            Colors::SLATE_700,
            Colors::SLATE_800,
            Colors::SLATE_900,
            Colors::SLATE_950,
        ];

        // Each color should be darker than the previous (lower RGB values)
        for i in 1..slate_colors.len() {
            let (r1, g1, b1) = slate_colors[i - 1];
            let (r2, g2, b2) = slate_colors[i];

            // Generally, RGB values should decrease (get darker)
            // Allow some tolerance for color theory adjustments
            assert!(
                r2 as i32 <= r1 as i32 + 10,
                "Red should generally decrease: {} -> {}",
                r1,
                r2
            );
            assert!(
                g2 as i32 <= g1 as i32 + 10,
                "Green should generally decrease: {} -> {}",
                g1,
                g2
            );
            assert!(
                b2 as i32 <= b1 as i32 + 10,
                "Blue should generally decrease: {} -> {}",
                b1,
                b2
            );
        }
    }

    #[test]
    fn test_get_color_function() {
        // Test valid color lookups
        assert_eq!(get_color("slate", "50"), Some(Colors::SLATE_50));
        assert_eq!(get_color("slate", "500"), Some(Colors::SLATE_500));
        assert_eq!(get_color("slate", "950"), Some(Colors::SLATE_950));

        assert_eq!(get_color("red", "50"), Some(Colors::RED_50));
        assert_eq!(get_color("red", "500"), Some(Colors::RED_500));
        assert_eq!(get_color("red", "950"), Some(Colors::RED_950));

        assert_eq!(get_color("blue", "50"), Some(Colors::BLUE_50));
        assert_eq!(get_color("blue", "500"), Some(Colors::BLUE_500));
        assert_eq!(get_color("blue", "950"), Some(Colors::BLUE_950));

        // Test all neutral colors
        assert_eq!(get_color("gray", "500"), Some(Colors::GRAY_500));
        assert_eq!(get_color("zinc", "500"), Some(Colors::ZINC_500));
        assert_eq!(get_color("neutral", "500"), Some(Colors::NEUTRAL_500));
        assert_eq!(get_color("stone", "500"), Some(Colors::STONE_500));
    }

    #[test]
    fn test_get_color_invalid() {
        // Test invalid color names
        assert_eq!(get_color("invalid", "500"), None);
        assert_eq!(get_color("purple", "1000"), None);
        assert_eq!(get_color("", "500"), None);

        // Test invalid shades
        assert_eq!(get_color("red", "invalid"), None);
        assert_eq!(get_color("red", "1000"), None);
        assert_eq!(get_color("red", ""), None);
        assert_eq!(get_color("red", "25"), None); // Not a standard shade

        // Test both invalid
        assert_eq!(get_color("invalid", "invalid"), None);
    }

    #[test]
    fn test_all_color_families() {
        let color_families = [
            "slate", "gray", "zinc", "neutral", "stone", "red", "orange", "amber", "yellow",
            "lime", "green", "emerald", "teal", "cyan", "sky", "blue", "indigo", "violet",
            "purple", "fuchsia", "pink", "rose",
        ];

        let shades = [
            "50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950",
        ];

        // Test that all combinations exist
        for family in &color_families {
            for shade in &shades {
                let result = get_color(family, shade);
                assert!(result.is_some(), "Color {}-{} should exist", family, shade);

                let (r, g, b) = result.unwrap();
                // u8 values are always valid (0-255), just verify they exist
                let _ = (r, g, b); // Consume the values to verify they're accessible
            }
        }
    }

    #[test]
    fn test_color_uniqueness() {
        // Test that different colors are actually different
        assert_ne!(Colors::RED_500, Colors::BLUE_500);
        assert_ne!(Colors::GREEN_500, Colors::YELLOW_500);
        assert_ne!(Colors::PURPLE_500, Colors::PINK_500);

        // Test that different shades of same color are different
        assert_ne!(Colors::RED_100, Colors::RED_500);
        assert_ne!(Colors::RED_500, Colors::RED_900);
        assert_ne!(Colors::BLUE_50, Colors::BLUE_950);
    }

    #[test]
    fn test_extreme_shades() {
        // Test lightest shades (50) - should be very light
        let light_colors = [
            Colors::RED_50,
            Colors::BLUE_50,
            Colors::GREEN_50,
            Colors::YELLOW_50,
            Colors::PURPLE_50,
            Colors::PINK_50,
        ];

        for (r, g, b) in &light_colors {
            // Light colors should have high RGB values
            assert!(
                *r > 200 || *g > 200 || *b > 200,
                "Light color should have high RGB: ({}, {}, {})",
                r,
                g,
                b
            );
        }

        // Test darkest shades (950) - should be very dark
        let dark_colors = [
            Colors::RED_950,
            Colors::BLUE_950,
            Colors::GREEN_950,
            Colors::YELLOW_950,
            Colors::PURPLE_950,
            Colors::PINK_950,
        ];

        for (r, g, b) in &dark_colors {
            // Dark colors should have at least one low RGB value, and overall low brightness
            let brightness = (*r as u32 + *g as u32 + *b as u32) / 3;
            assert!(
                brightness < 150,
                "Dark color should have low overall brightness: ({}, {}, {}) avg={}",
                r,
                g,
                b,
                brightness
            );
        }
    }

    #[test]
    fn test_color_case_sensitivity() {
        // Test that color lookup is case sensitive
        assert_eq!(get_color("red", "500"), Some(Colors::RED_500));
        assert_eq!(get_color("RED", "500"), None); // Should be case sensitive
        assert_eq!(get_color("Red", "500"), None);
        assert_eq!(get_color("red", "500"), Some(Colors::RED_500));
        assert_eq!(get_color("red", "500"), Some(Colors::RED_500));
    }

    #[test]
    fn test_standard_shades() {
        // Test that all standard Tailwind shades exist for red
        let standard_shades = [
            "50", "100", "200", "300", "400", "500", "600", "700", "800", "900", "950",
        ];

        for shade in &standard_shades {
            assert!(
                get_color("red", shade).is_some(),
                "Red-{} should exist",
                shade
            );
            assert!(
                get_color("blue", shade).is_some(),
                "Blue-{} should exist",
                shade
            );
            assert!(
                get_color("green", shade).is_some(),
                "Green-{} should exist",
                shade
            );
        }
    }

    #[test]
    fn test_color_rgb_ranges() {
        // Test that all colors have valid RGB values (0-255)
        let all_colors = [
            Colors::SLATE_50,
            Colors::GRAY_100,
            Colors::ZINC_200,
            Colors::NEUTRAL_300,
            Colors::STONE_400,
            Colors::RED_500,
            Colors::ORANGE_600,
            Colors::AMBER_700,
            Colors::YELLOW_800,
            Colors::LIME_900,
            Colors::GREEN_950,
            Colors::EMERALD_50,
            Colors::TEAL_100,
            Colors::CYAN_200,
            Colors::SKY_300,
            Colors::BLUE_400,
            Colors::INDIGO_500,
            Colors::VIOLET_600,
            Colors::PURPLE_700,
            Colors::FUCHSIA_800,
            Colors::PINK_900,
            Colors::ROSE_950,
        ];

        // Every sampled palette entry is a distinct colour
        let distinct: std::collections::HashSet<_> = all_colors.iter().collect();
        assert_eq!(
            distinct.len(),
            all_colors.len(),
            "palette samples repeat a colour"
        );
        // Lighter shades (50) sum brighter than the darkest shades (950)
        let brightness = |(r, g, b): (u8, u8, u8)| u16::from(r) + u16::from(g) + u16::from(b);
        assert!(brightness(Colors::SLATE_50) > brightness(Colors::GREEN_950));
        assert!(brightness(Colors::EMERALD_50) > brightness(Colors::ROSE_950));
    }
}
