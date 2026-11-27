//! The software keyboard ("menu    keypadus", "menu    keypad  " in Japanese; handler DAT_10663f64, vtable
//! 0x10338a88; MODLOG "Software keyboard"): the pad's RB with config 0xc9 on opens it over the open text field (or
//! the chat line), and its keys type there.
//! Addresses are FFXiMain 30261102_0.

use super::World;
use super::game_ui::GameUi;
use crate::audio::Ui;

use crate::dat::items::Lang;

/// The client language is Japanese (DAT_10457bd8 +8 = 0): the keyboard's window, caps, pages and keys go by it.
fn ja() -> bool {
    crate::dat::lang::client() == Lang::Ja
}

/// FUN_101b0100 / FUN_101b0150 / FUN_101b0180: "menu    keypad  " in Japanese, "menu    keypadus" in the others.
pub fn name() -> &'static str {
    if ja() { "menu    keypad" } else { "menu    keypadus" }
}

/// FUN_101b0090: the caps' library, key1top in Japanese, key1topu in the others.
fn caps_lib() -> &'static str {
    if ja() { "menu    key1top" } else { "menu    key1topu" }
}

const HAND: &str = "menu    yubi";

/// What a page's keys type: kind 1 a byte each, kind 2 a Shift-JIS pair each (lead byte low).
enum Keys {
    Bytes([u8; 75]),
    Pairs([u16; 75]),
}

/// A record of table 0x1037d320 (13 of 7 dwords): its keys, each key's cap (key1top / key1topu), child 4's cap, the
/// single-byte flag, the page after SHIFT ([5]) and after the pad's X ([6]). Pages 0..5 are the Japanese keyboard's
/// (hiragana, katakana, full-width letters, half-width letters, symbols, half-width symbols), 6..12 the others'.
struct Page {
    caps: [u16; 75],
    keys: Keys,
    c4: u16,
    single: bool,
    shift: u8,
    x: u8,
}

/// Copied from FFXiMain 30261102_0.
const PAGES: [Page; 13] = [
    // Page 0.
    Page {
        caps: [
            165, 166, 167, 168, 169, 190, 191, 192, 193, 194, 275, 276, 21, 107, 108, 170, 171, 172, 173, 174, 195,
            196, 197, 198, 199, 211, 212, 213, 214, 215, 175, 176, 177, 178, 179, 200, 21, 201, 21, 202, 216, 217, 218,
            219, 21, 180, 181, 182, 183, 184, 203, 204, 205, 206, 207, 277, 278, 279, 280, 281, 185, 186, 187, 188,
            189, 208, 21, 209, 21, 210, 104, 105, 21, 21, 21,
        ],
        keys: Keys::Pairs([
            0xa082, 0xa282, 0xa482, 0xa682, 0xa882, 0xcd82, 0xd082, 0xd382, 0xd682, 0xd982, 0x4a81, 0x4b81, 0x4081,
            0x5e81, 0x9781, 0xa982, 0xab82, 0xad82, 0xaf82, 0xb182, 0xdc82, 0xdd82, 0xde82, 0xdf82, 0xe082, 0x9f82,
            0xa182, 0xa382, 0xa582, 0xa782, 0xb382, 0xb582, 0xb782, 0xb982, 0xbb82, 0xe282, 0x4081, 0xe482, 0x4081,
            0xe682, 0xc182, 0xe182, 0xe382, 0xe582, 0x4081, 0xbd82, 0xbf82, 0xc282, 0xc482, 0xc682, 0xe782, 0xe882,
            0xe982, 0xea82, 0xeb82, 0x4181, 0x4281, 0x5b81, 0x4981, 0x4881, 0xc882, 0xc982, 0xca82, 0xcb82, 0xcc82,
            0xed82, 0x4081, 0xf082, 0x4081, 0xf182, 0x4681, 0x4581, 0x4081, 0x4081, 0x4081,
        ]),
        c4: 3,
        single: false,
        shift: 1,
        x: 2,
    },
    // Page 1.
    Page {
        caps: [
            220, 221, 222, 223, 224, 245, 246, 247, 248, 249, 275, 276, 21, 107, 108, 225, 226, 227, 228, 229, 250,
            251, 252, 253, 254, 266, 267, 268, 269, 270, 230, 231, 232, 233, 234, 255, 21, 256, 21, 257, 271, 272, 273,
            274, 21, 235, 236, 237, 238, 239, 258, 259, 260, 261, 262, 277, 278, 279, 280, 281, 240, 241, 242, 243,
            244, 263, 21, 264, 21, 265, 104, 105, 21, 21, 21,
        ],
        keys: Keys::Pairs([
            0x4183, 0x4383, 0x4583, 0x4783, 0x4983, 0x6e83, 0x7183, 0x7483, 0x7783, 0x7a83, 0x4a81, 0x4b81, 0x4081,
            0x5e81, 0x9781, 0x4a83, 0x4c83, 0x4e83, 0x5083, 0x5283, 0x7d83, 0x7e83, 0x8083, 0x8183, 0x8283, 0x4083,
            0x4283, 0x4483, 0x4683, 0x4883, 0x5483, 0x5683, 0x5883, 0x5a83, 0x5c83, 0x8483, 0x4081, 0x8683, 0x4081,
            0x8883, 0x6283, 0x8383, 0x8583, 0x8783, 0x4081, 0x5e83, 0x6083, 0x6383, 0x6583, 0x6783, 0x8983, 0x8a83,
            0x8b83, 0x8c83, 0x8d83, 0x4181, 0x4281, 0x5b81, 0x4981, 0x4881, 0x6983, 0x6a83, 0x6b83, 0x6c83, 0x6d83,
            0x8f83, 0x4081, 0x9283, 0x4081, 0x9383, 0x4681, 0x4581, 0x4081, 0x4081, 0x4081,
        ]),
        c4: 4,
        single: false,
        shift: 0,
        x: 2,
    },
    // Page 2.
    Page {
        caps: [
            94, 95, 96, 97, 98, 99, 100, 101, 102, 103, 104, 337, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115,
            116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 127, 128, 129, 130, 131, 132, 133, 134, 320, 280,
            281, 21, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
            156, 157, 158, 159, 160, 161, 162, 163, 164, 427, 21,
        ],
        keys: Keys::Pairs([
            0x5082, 0x5182, 0x5282, 0x5382, 0x5482, 0x5582, 0x5682, 0x5782, 0x5882, 0x4f82, 0x4681, 0x4781, 0x5181,
            0x5e81, 0x9781, 0x8182, 0x8282, 0x8382, 0x8482, 0x8582, 0x8682, 0x8782, 0x8882, 0x8982, 0x8a82, 0x8b82,
            0x8c82, 0x8d82, 0x8e82, 0x8f82, 0x9082, 0x9182, 0x9282, 0x9382, 0x9482, 0x9582, 0x9682, 0x9782, 0x9882,
            0x9982, 0x9a82, 0x7c81, 0x4981, 0x4881, 0x4081, 0x6082, 0x6182, 0x6282, 0x6382, 0x6482, 0x6582, 0x6682,
            0x6782, 0x6882, 0x6982, 0x6a82, 0x6b82, 0x6c82, 0x6d82, 0x6e82, 0x6f82, 0x7082, 0x7182, 0x7282, 0x7382,
            0x7482, 0x7582, 0x7682, 0x7782, 0x7882, 0x7982, 0x4381, 0x4481, 0x5f81, 0x4081,
        ]),
        c4: 5,
        single: false,
        shift: 3,
        x: 4,
    },
    // Page 3.
    Page {
        caps: [
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 410, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
            49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 320, 64, 65, 21, 66, 67, 68, 69, 70, 71, 72,
            73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 426, 21,
        ],
        keys: Keys::Bytes(*b"1234567890:;_/@abcdefghijklmnopqrstuvwxyz-!? ABCDEFGHIJKLMNOPQRSTUVWXYZ,.\\ "),
        c4: 6,
        single: false,
        shift: 2,
        x: 5,
    },
    // Page 4.
    Page {
        caps: [
            283, 284, 285, 286, 287, 288, 289, 290, 291, 292, 293, 294, 295, 296, 297, 298, 299, 300, 301, 302, 303,
            304, 305, 306, 307, 308, 309, 310, 311, 314, 312, 313, 338, 339, 319, 320, 321, 322, 323, 324, 107, 341,
            325, 326, 327, 315, 316, 317, 318, 277, 278, 163, 164, 105, 104, 337, 340, 334, 335, 336, 108, 332, 330,
            342, 328, 329, 280, 281, 282, 279, 333, 331, 343, 344, 345,
        ],
        keys: Keys::Pairs([
            0x9e81, 0x9f81, 0xa081, 0xa181, 0x9b81, 0x9c81, 0xa281, 0xa381, 0x9981, 0x9a81, 0xa481, 0xa581, 0x9d81,
            0x8981, 0x8a81, 0x8381, 0x8481, 0x7581, 0x7681, 0x7781, 0x7881, 0x7181, 0x7281, 0x7381, 0x7481, 0x6d81,
            0x6e81, 0x6f81, 0x7081, 0xf481, 0x6981, 0x6a81, 0x6581, 0x6681, 0x7b81, 0x7c81, 0x7e81, 0x8081, 0x8181,
            0x9381, 0x5e81, 0x9681, 0xa184, 0xa684, 0xa284, 0xaa81, 0xab81, 0xa981, 0xa881, 0x4181, 0x4281, 0x4381,
            0x4481, 0x4581, 0x4681, 0x4781, 0x6881, 0xa584, 0xa984, 0xa784, 0x9781, 0x9581, 0xa681, 0x9481, 0x8f81,
            0x9081, 0x4981, 0x4881, 0x6381, 0x5b81, 0x6081, 0xa781, 0xa484, 0xa884, 0xa384,
        ]),
        c4: 5,
        single: false,
        shift: 5,
        x: 0,
    },
    // Page 5.
    Page {
        caps: [
            346, 347, 348, 349, 350, 351, 352, 353, 354, 355, 356, 357, 358, 359, 360, 361, 362, 363, 364, 365, 366,
            367, 368, 369, 370, 371, 372, 373, 37, 374, 375, 376, 377, 378, 379, 380, 381, 382, 383, 384, 385, 386,
            387, 388, 389, 390, 391, 392, 393, 394, 395, 396, 368, 397, 341, 342, 339, 338, 333, 401, 402, 403, 404,
            405, 406, 407, 408, 409, 319, 320, 323, 324, 107, 425, 399,
        ],
        keys: Keys::Pairs([
            0xbf85, 0xc085, 0xc185, 0xc385, 0xc685, 0xc785, 0xc885, 0xc985, 0xca85, 0xcb85, 0xcc85, 0xcd85, 0xce85,
            0xd085, 0xd185, 0xd285, 0xd385, 0xd585, 0xd885, 0xd985, 0xda85, 0xdb85, 0xbe85, 0x5c85, 0x6185, 0x8085,
            0x8f81, 0x9081, 0x9781, 0x9581, 0x9f85, 0xa085, 0xa185, 0xa385, 0xa685, 0xa785, 0xa885, 0xa985, 0xaa85,
            0xab85, 0xac85, 0xad85, 0xae85, 0xb085, 0xb185, 0xb285, 0xb385, 0xb585, 0xb885, 0xb985, 0xba85, 0xbb85,
            0xbe85, 0x4c85, 0x9681, 0x9481, 0x6581, 0x6681, 0x6081, 0x4f81, 0x8381, 0x8481, 0x6d81, 0x6e81, 0x6f81,
            0x7081, 0x6981, 0x6a81, 0x7b81, 0x7c81, 0x8181, 0x9381, 0x5e81, 0x6281, 0x6881,
        ]),
        c4: 6,
        single: false,
        shift: 4,
        x: 0,
    },
    // Page 6.
    Page {
        caps: [
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 21, 21, 21, 21, 21, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
            50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 21, 21, 21, 21, 66, 67, 68, 69, 70, 71, 72, 73, 74,
            75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 21, 21, 21, 21,
        ],
        keys: Keys::Bytes(*b"1234567890     abcdefghijklmnopqrstuvwxyz    ABCDEFGHIJKLMNOPQRSTUVWXYZ    "),
        c4: 17,
        single: true,
        shift: 6,
        x: 6,
    },
    // Page 7.
    Page {
        caps: [
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 402, 403, 21, 21, 21, 21, 21, 21, 21, 21, 404,
            405, 406, 407, 21, 408, 409, 400, 400, 319, 320, 21, 21, 323, 324, 36, 412, 21, 21, 21, 21, 21, 21, 21, 21,
            21, 92, 93, 21, 104, 410, 399, 21, 21, 21, 108, 374, 401, 411, 426, 373, 64, 65, 106, 320, 398, 425, 21,
            21, 21,
        ],
        keys: Keys::Bytes(*b"               <>        []{} ()`'+-  =%/*         ,. :;\"   @&^#\\$!?_-~|   "),
        c4: 17,
        single: true,
        shift: 8,
        x: 8,
    },
    // Page 8.
    Page {
        caps: [
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 410, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
            49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 320, 64, 65, 21, 66, 67, 68, 69, 70, 71, 72,
            73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 426, 21,
        ],
        keys: Keys::Bytes(*b"1234567890:;_/@abcdefghijklmnopqrstuvwxyz-!? ABCDEFGHIJKLMNOPQRSTUVWXYZ,.\\ "),
        c4: 17,
        single: true,
        shift: 7,
        x: 7,
    },
    // Page 9.
    Page {
        caps: [
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49,
            50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 21, 21, 21, 21, 66, 67, 68, 69, 70, 71, 72, 73, 74,
            75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 21, 21, 21, 21,
        ],
        keys: Keys::Bytes(*b"               abcdefghijklmnopqrstuvwxyz    ABCDEFGHIJKLMNOPQRSTUVWXYZ    "),
        c4: 17,
        single: true,
        shift: 9,
        x: 9,
    },
    // Page 10.
    Page {
        caps: [
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
            21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21, 21,
        ],
        keys: Keys::Bytes(*b"1234567890                                                                 "),
        c4: 17,
        single: true,
        shift: 10,
        x: 10,
    },
    // Page 11.
    Page {
        caps: [
            23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 93, 92, 400, 399, 374, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48,
            49, 50, 108, 411, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 408, 409, 346, 347, 348, 349, 350,
            351, 352, 353, 354, 355, 356, 357, 358, 404, 405, 359, 360, 361, 362, 363, 364, 365, 366, 367, 369, 368,
            306, 307, 64, 65,
        ],
        keys: Keys::Pairs([
            0x5082, 0x5182, 0x5282, 0x5382, 0x5482, 0x5582, 0x5682, 0x5782, 0x5882, 0x4f82, 0x4481, 0x4381, 0x6681,
            0x6881, 0x9581, 0x8182, 0x8282, 0x8382, 0x8482, 0x8582, 0x8682, 0x8782, 0x8882, 0x8982, 0x8a82, 0x8b82,
            0x8c82, 0x8d82, 0x9781, 0x9481, 0x8e82, 0x8f82, 0x9082, 0x9182, 0x9282, 0x9382, 0x9482, 0x9582, 0x9682,
            0x9782, 0x9882, 0x9982, 0x9a82, 0x6981, 0x6a81, 0xbf85, 0xc085, 0xc185, 0xc385, 0xc685, 0xc785, 0xc885,
            0xc985, 0xca85, 0xcb85, 0xcc85, 0xcd85, 0xce85, 0x6d81, 0x6e81, 0xd085, 0xd185, 0xd285, 0xd385, 0xd585,
            0xd885, 0xd985, 0xda85, 0xdb85, 0x5c85, 0xbe85, 0x6b85, 0x7b85, 0x4981, 0x4881,
        ]),
        c4: 17,
        single: false,
        shift: 12,
        x: 12,
    },
    // Page 12.
    Page {
        caps: [
            319, 320, 412, 36, 323, 373, 426, 324, 104, 410, 401, 398, 106, 425, 276, 66, 67, 68, 69, 70, 71, 72, 73,
            74, 75, 76, 77, 78, 430, 431, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 406, 407, 375, 376, 377,
            378, 379, 380, 381, 382, 383, 384, 385, 386, 387, 402, 403, 388, 389, 390, 391, 392, 393, 394, 395, 396,
            397, 368, 432, 432, 370, 371,
        ],
        keys: Keys::Pairs([
            0x7b81, 0x7c81, 0x9681, 0x5e81, 0x8181, 0x9081, 0x5f81, 0x9381, 0x4681, 0x4781, 0x4f81, 0x6081, 0x5181,
            0x6281, 0x7085, 0x6082, 0x6182, 0x6282, 0x6382, 0x6482, 0x6582, 0x6682, 0x6782, 0x6882, 0x6982, 0x6a82,
            0x6b82, 0x6c82, 0x4085, 0x6385, 0x6d82, 0x6e82, 0x6f82, 0x7082, 0x7182, 0x7282, 0x7382, 0x7482, 0x7582,
            0x7682, 0x7782, 0x7882, 0x7982, 0x6f81, 0x7081, 0x9f85, 0xa085, 0xa185, 0xa385, 0xa685, 0xa785, 0xa885,
            0xa985, 0xaa85, 0xab85, 0xac85, 0xad85, 0xae85, 0x8381, 0x8481, 0xb085, 0xb185, 0xb285, 0xb385, 0xb585,
            0xb885, 0xb985, 0xba85, 0xbb85, 0x4c85, 0xbe85, 0x5685, 0x5785, 0x6185, 0x8085,
        ]),
        c4: 17,
        single: false,
        shift: 11,
        x: 11,
    },
];

/// Table 0x1037d568: the pair each byte from 0x20 stands for (FUN_101b5a20).
const BYTE_PAIRS: [u16; 0xe0] = [
    0x4081, 0x4981, 0x6881, 0x9481, 0x9081, 0x9381, 0x9581, 0x6681, 0x6981, 0x6a81, 0x9681, 0x7b81, 0x4381, 0x7c81,
    0x4481, 0x5e81, 0x4f82, 0x5082, 0x5182, 0x5282, 0x5382, 0x5482, 0x5582, 0x5682, 0x5782, 0x5882, 0x4681, 0x4781,
    0x8381, 0x8181, 0x8481, 0x4881, 0x9781, 0x6082, 0x6182, 0x6282, 0x6382, 0x6482, 0x6582, 0x6682, 0x6782, 0x6882,
    0x6982, 0x6a82, 0x6b82, 0x6c82, 0x6d82, 0x6e82, 0x6f82, 0x7082, 0x7182, 0x7282, 0x7382, 0x7482, 0x7582, 0x7682,
    0x7782, 0x7882, 0x7982, 0x6d81, 0x5f81, 0x6e81, 0x4f81, 0x5181, 0x6581, 0x8182, 0x8282, 0x8382, 0x8482, 0x8582,
    0x8682, 0x8782, 0x8882, 0x8982, 0x8a82, 0x8b82, 0x8c82, 0x8d82, 0x8e82, 0x8f82, 0x9082, 0x9182, 0x9282, 0x9382,
    0x9482, 0x9582, 0x9682, 0x9782, 0x9882, 0x9982, 0x9a82, 0x6f81, 0x6281, 0x7081, 0x6081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4281, 0x7581, 0x7681, 0x4181, 0x4581, 0xf082, 0x9f82, 0xa182, 0xa382, 0xa582, 0xa782,
    0xe182, 0xe382, 0xe582, 0xc182, 0x5b81, 0xa082, 0xa282, 0xa482, 0xa682, 0xa882, 0xa982, 0xab82, 0xad82, 0xaf82,
    0xb182, 0xb382, 0xb582, 0xb782, 0xb982, 0xbb82, 0xbd82, 0xbf82, 0xc282, 0xc482, 0xc682, 0xc882, 0xc982, 0xca82,
    0xcb82, 0xcc82, 0xcd82, 0xd082, 0xd382, 0xd682, 0xd982, 0xdc82, 0xdd82, 0xde82, 0xdf82, 0xe082, 0xe282, 0xe482,
    0xe682, 0xe782, 0xe882, 0xe982, 0xea82, 0xeb82, 0xed82, 0xf182, 0x4a81, 0x4b81, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
];

/// Table 0x1037d730: a second pair for the bytes from 0xa0.
const BYTE_PAIRS_HIGH: [u16; 0x60] = [
    0x4081, 0x4281, 0x7581, 0x7681, 0x4181, 0x4581, 0x9283, 0x4083, 0x4283, 0x4483, 0x4683, 0x4883, 0x8383, 0x8583,
    0x8783, 0x6283, 0x5b81, 0x4183, 0x4383, 0x4583, 0x4783, 0x4983, 0x4a83, 0x4c83, 0x4e83, 0x5083, 0x5283, 0x5483,
    0x5683, 0x5883, 0x5a83, 0x5c83, 0x5e83, 0x6083, 0x6383, 0x6583, 0x6783, 0x6983, 0x6a83, 0x6b83, 0x6c83, 0x6d83,
    0x6e83, 0x7183, 0x7483, 0x7783, 0x7a83, 0x7d83, 0x7e83, 0x8083, 0x8183, 0x8283, 0x8483, 0x8683, 0x8883, 0x8983,
    0x8a83, 0x8b83, 0x8c83, 0x8d83, 0x8f83, 0x9383, 0x4a81, 0x4b81, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
    0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081, 0x4081,
];

/// FUN_101b0c00: the page record; past 12 none.
fn page(p: u8) -> Option<&'static Page> {
    PAGES.get(p as usize)
}

/// FUN_101b0200: the caps drawn dim (European letters). 0x114, 0x132 and 0x133 are dim only outside Japanese.
fn dim(sprite: u16) -> bool {
    matches!(sprite, 0x15a..=0x173 | 0x177..=0x18d | 0x1ae..=0x1b0)
        || (!ja() && matches!(sprite, 0x114 | 0x132 | 0x133))
}

/// The text engine's input mode (T+0xf10c, copied to the keyboard's +0x24 as it opens): 1 when the field takes
/// characters as they are typed. FUN_101b6e20 sets it to 1 outside Japanese and to the field's own mode in Japanese,
/// where the chat line's 0 sends what is typed to the IME's composition (FUN_101b2b40). Ours has no composition, so
/// its fields are direct (1) in every language.
const MODE: u32 = 1;

/// The field's character class (T+0xf110: 0 letters, 1 digits, 2 / 5 any, 3 alphanumeric, 4 printable). Ours: every
/// field is taken as the chat line's 5 until their classes are mapped.
const CLASS: u32 = 5;

/// FUN_101b0b50 as the keyboard opens: the first page by the mode and the field's class. With mode 1 the class picks
/// it (2 / 5: 3 in Japanese, 11 in the others); mode 2 is page 9; other modes keep the page (+0x2c, FUN_101b0010: 0
/// in Japanese, 11 in the others), or 0 when it is a single-byte one.
fn first_page(mode: u32, class: u32) -> u8 {
    let kept = if ja() { 0 } else { 0xb };
    let any = if ja() { 3 } else { 11 };
    match (mode, class) {
        (1, 0) | (2, _) => 9,
        (1, 1) => 10,
        (1, 2) | (1, 5) => any,
        (1, 3) => 6,
        (1, 4) => 8,
        (1, _) => kept,
        _ if page(kept).is_some_and(|p| p.single) => 0,
        _ => kept,
    }
}

/// FUN_101b1110: a page SHIFT or X may turn to; mode 1 keeps off the kana and full-width pages 0, 1, 2 and 4.
fn turnable(to: u8) -> bool {
    match to {
        0 | 1 | 2 | 4 => MODE != 1,
        3 | 5..=12 => true,
        _ => false,
    }
}

/// FUN_101b11b0: the tabs (children 1..3) choose a page; mode 1 lets a class 2 field have page 3 only and a class 5
/// field pages 3 and 5. Whether the page is `to` after it (false buzzes).
fn choose(page: &mut u8, to: u8) -> bool {
    if *page == to {
        return false;
    }
    let ok = match MODE {
        0 => true,
        1 => match CLASS {
            2 => to == 3,
            5 => matches!(to, 3 | 5),
            _ => false,
        },
        _ => false,
    };
    if ok {
        *page = to;
    }
    *page == to
}

/// FUN_101b1150 (SHIFT, `x` false) and FUN_101b1180 (the pad's X): the page's [5] / [6] when it is another page that
/// may be turned to.
fn turn(page: &mut u8, x: bool) -> bool {
    let Some(p) = self::page(*page) else { return false };
    let to = if x { p.x } else { p.shift };
    if to != *page && turnable(to) {
        *page = to;
        return true;
    }
    false
}

/// FUN_101b5a20: a pair as one byte (tables 0x1037d568 from 0x20, 0x1037d730 from 0xa0; 0x8f81 none). A European
/// letter (lead 0x85) gives its byte with the flag set.
fn pair_byte(pair: u16) -> (u8, bool) {
    if pair == 0x8f81 {
        return (0, false);
    }
    if pair & 0xff == 0x85 {
        let hi = (pair >> 8) as u8;
        let b = if hi < 0x7f { hi.wrapping_add(0x40) } else { hi.wrapping_add(0x41) };
        return (if hi < 0x7f && b == 0xc0 { 0xbf } else { b }, true);
    }
    if let Some(i) = BYTE_PAIRS.iter().position(|&v| v == pair) {
        return ((i as u8).wrapping_add(0x20), false);
    }
    match BYTE_PAIRS_HIGH.iter().position(|&v| v == pair) {
        Some(i) => ((i as u8).wrapping_add(0xa0), false),
        None => (0, false),
    }
}

/// A byte (T vt+4 kind 5) or a Shift-JIS pair (kind 6) typed into the field.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Typed {
    Byte(u8),
    Pair(u16),
}

/// What a character key types (FUN_101b0e30's default case, FUN_101b0c20): kind 1 its byte; kind 2 its pair, made
/// one byte outside Japanese or on page 5 (a European letter then types nothing, one with no byte types the pair,
/// the full-width space a space).
fn typed(page: u8, i: usize) -> Option<Typed> {
    let p = self::page(page)?;
    match &p.keys {
        Keys::Bytes(b) => Some(Typed::Byte(b[i])),
        Keys::Pairs(v) => {
            let pair = v[i];
            if !ja() || page == 5 {
                let (b, european) = pair_byte(pair);
                if european {
                    return None;
                }
                if b != 0 {
                    return Some(Typed::Byte(b));
                }
            }
            Some(if pair == 0x4081 { Typed::Byte(b' ') } else { Typed::Pair(pair) })
        }
    }
}

/// A typed byte or pair as the text the field reads.
fn text(t: Typed) -> String {
    match t {
        Typed::Byte(b) if b < 0x80 => char::from(b).to_string(),
        Typed::Byte(b) => crate::dat::font::decode_sjis(&[b]),
        Typed::Pair(p) => crate::dat::font::decode_sjis(&[p as u8, (p >> 8) as u8]),
    }
}

/// The open keyboard: its page (FUN_101b0b50 at each open), whether children 1..3 lost their sprites to drawn caps at
/// the open (win_keypad_v04: Japanese, a single-byte page), the cursor child, and where it stands (fixed when it
/// first draws, win_keypad_v04).
#[derive(Clone, Debug)]
pub struct Keyboard {
    page: u8,
    tabs_capped: bool,
    pub cursor: u16,
    at: std::cell::Cell<Option<(f32, f32)>>,
}

impl Keyboard {
    /// FUN_101b0100 -> win_keypad_v04: the first page, the cursor on child 8 (FUN_1011a240(8, 0)).
    fn open() -> Self {
        let page = first_page(MODE, CLASS);
        let tabs_capped = ja() && self::page(page).is_some_and(|p| p.single);
        Keyboard { page, tabs_capped, cursor: 8, at: Default::default() }
    }
}

/// The children drawn by the generic draw as the window's own (their sprites kept): Backspace, Tab, Config, and
/// outside Japanese SHIFT, SPACE and Enter; in Japanese the tabs 1..3 (unless capped), 5 and 0x54 too.
fn sprite_kept(k: &Keyboard, c: u16) -> bool {
    if ja() {
        matches!(c, 5..=7 | 0x53 | 0x54) || (matches!(c, 1..=3) && !k.tabs_capped)
    } else {
        matches!(c, 6 | 7 | 0x53 | 0x55..=0x57)
    }
}

/// Children in state 0x10 (no draw, no hit): outside Japanese the tabs 1..5 and 0x54 (FUN_10119ff0(0x10, ..)).
fn off(c: u16) -> bool {
    !ja() && matches!(c, 1..=5 | 0x54)
}

/// FUN_101b29c0, the pad's RB (key function 0x24): with config 0xc9 on, the keyboard over the open text field, or
/// over the chat line it opens when none is. Nothing with 0xc9 off, or while it is up.
pub fn rb(w: &mut World) {
    if w.game.get(0xc9) == 0 || w.swkbd.is_some() {
        return;
    }
    if !super::text_field_open(w) {
        w.typing = true;
    }
    w.swkbd = Some(Keyboard::open());
}

fn key_event(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key { key, physical_key: Some(key), pressed, repeat: false, modifiers: egui::Modifiers::default() }
}

/// What a key types into the field, as the keyboard would type it (T vt+4 kinds 5 / 6 and 3): next frame's input.
fn send_key(w: &mut World, key: egui::Key) {
    w.swkbd_out.extend([key_event(key, true), key_event(key, false)]);
}

fn send_text(w: &mut World, t: Typed) {
    w.swkbd_out.push(egui::Event::Text(text(t)));
}

/// Kind 8, close (FUN_101b72a0): the chat line closes with it, its text kept; a window field only loses the keyboard.
fn close(w: &mut World) {
    if w.typing {
        w.typing = false;
    }
    w.swkbd = None;
}

/// menu_configwi_fn (FUN_101b1210), Config: in the field with no event, the field closes and configwi opens (SE 0002
/// again).
fn config(w: &mut World) {
    if super::event::active(w) {
        w.sounds.push(Ui::Error as u32);
        return;
    }
    w.sounds.push(Ui::Select as u32);
    w.typing = false;
    w.swkbd = None;
    let mut cfg = super::configui::ConfigUi::open("configwi");
    cfg.mon = super::monstrosity::active(w);
    cfg.cam_keys = w.cam_keys;
    // conf3win's Show / Hide: the 0x0B4 config block's anonymous bit (FUN_100e4830(0)).
    cfg.anon = w.conf_sys[0] & 4 != 0;
    cfg.recall(w.cursor_recall(super::configui::CONFIGWI_MENU));
    w.config = Some(cfg);
}

/// FUN_101b0e30: A (or a click) on child `c` (its table 0x101b10ac). A page change that fails buzzes (FUN_10037c50);
/// one that is made commits the composition (vt+4 kind 2), which ours never has.
fn press(w: &mut World, c: u16) {
    let Some(k) = w.swkbd.as_mut() else { return };
    let ja = ja();
    let turned = match c {
        // The tabs: page 0; 2 or 3 and 4 or 5 (the lower one while on page 0, 1, 2 or 4, FUN_101b01b0).
        1 => choose(&mut k.page, 0),
        2 | 3 => {
            let low = matches!(k.page, 0..=2 | 4);
            let to = (if c == 2 { 3 } else { 5 }) - low as u8;
            choose(&mut k.page, to)
        }
        // SHIFT: the page's [5]. Outside Japanese child 0x55 is this; in Japanese it is a key of the composition
        // (Tab in mode 0), and buzzes in mode 1.
        4 => turn(&mut k.page, false),
        0x55 if !ja => turn(&mut k.page, false),
        0x55 => {
            if MODE == 0 {
                send_key(w, egui::Key::Tab);
                return;
            }
            false
        }
        5 => return close(w),
        6 => return send_key(w, egui::Key::Backspace),
        7 => return send_key(w, if ja { egui::Key::Escape } else { egui::Key::Tab }),
        // Config is 0x53 outside Japanese, 0x54 in it (0x53 does nothing there).
        0x53 if ja => return,
        0x53 | 0x54 => return config(w),
        // SPACE: a space outside Japanese; in Japanese the conversion of the composition (edit mode, mode 0 only),
        // which ours never has.
        0x56 if !ja => return send_text(w, Typed::Byte(b' ')),
        0x56 => false,
        0x57 => return send_key(w, egui::Key::Enter),
        8..=0x52 => {
            if let Some(t) = typed(k.page, (c - 8) as usize) {
                send_text(w, t);
            }
            return;
        }
        _ => return,
    };
    if !turned {
        w.sounds.push(Ui::Error as u32);
    }
}

/// One frame of the open keyboard (FUN_101acdf0's events, then the generic handler and win_keypad_onkey): the pad's
/// key functions in window context. It goes when its field does, and on any key of the keyboard (menu_fep_open_4
/// 0x101b3fc0; the key still goes to the field).
pub fn update(w: &mut World, gu: Option<&GameUi>, input: &mut super::input::Input) {
    use super::input::mode::{PRESS, REPEAT};
    if w.swkbd.is_none() {
        return;
    }
    if !super::text_field_open(w) || input.key_pressed_any() {
        w.swkbd = None;
        return;
    }
    // The d-pad, one a frame (Up, Down, Left, Right): the layout's links [2] Up, [3] Down, [5] Left, [4] Right.
    if let Some(&(_, l)) = [(0x2b, 2), (0x2c, 3), (0x2e, 5), (0x2d, 4)].iter().find(|(f, _)| input.test(*f, REPEAT)) {
        let cursor = w.swkbd.as_ref().map_or(8, |k| k.cursor);
        let to = gu.and_then(|g| g.menu_links(name(), cursor)).map_or(0xff, |links| links[l]);
        if to != 0xff && to as u16 != cursor {
            if let Some(k) = w.swkbd.as_mut() {
                k.cursor = to as u16;
            }
            w.sounds.push(Ui::Cursor as u32);
        }
    }
    let cursor = w.swkbd.as_ref().map_or(8, |k| k.cursor);
    if input.test(0x93, PRESS) {
        w.sounds.push(Ui::Select as u32);
        if let Some(g) = gu {
            g.press_on(name(), cursor);
        }
        press(w, cursor);
    }
    // The pad's other buttons (win_keypad_onkey 0x101b0760). Y (message 0xc) is Enter and RB (0x11) a space outside
    // Japanese; in Japanese RB is Enter and Y the composition's conversion (mode 0 only, so it buzzes in ours).
    if input.test(0x94, PRESS) {
        w.sounds.push(Ui::Cancel as u32);
        send_key(w, egui::Key::Escape);
    }
    for (f, key) in [
        (0x86, egui::Key::ArrowLeft),
        (0x85, egui::Key::ArrowRight),
        (0x14, egui::Key::Tab),
        (0x1f, egui::Key::Backspace),
    ] {
        if input.test(f, PRESS) {
            send_key(w, key);
        }
    }
    if input.test(0x22, PRESS) {
        if !ja() {
            send_key(w, egui::Key::Enter);
        } else if MODE != 0 {
            w.sounds.push(Ui::Error as u32);
        }
    }
    if input.test(0x1e, PRESS) {
        if ja() {
            send_key(w, egui::Key::Enter);
        } else {
            send_text(w, Typed::Byte(b' '));
        }
    }
    // X: the page's [6] (FUN_101b1180), silently nothing where it may not turn.
    if input.test(0x25, PRESS)
        && let Some(k) = w.swkbd.as_mut()
    {
        turn(&mut k.page, true);
    }
    if input.test(0x2f, PRESS) {
        close(w);
    }
    // Its window has the focus: the field's key functions don't see this frame's presses (FUN_10121730).
    input.consume();
}

/// Ours: the keyboard drawn at a size that reads (retail's 334 x 88 is a sliver of a large screen): about a third of
/// the UI's width, in whole steps so its pixel art stays sharp.
fn scale(sw: f32) -> f32 {
    (sw * 0.35 / 334.0).round().max(1.0)
}

/// win_keypad_v04's placement, as the window's top left (`kh` its height): below the macro editor, above the
/// comment editor, above the log window (its bottom 2 px over the log's top, or 34 px over its bottom while it has no
/// lines), above the chat line (32 more), else (32, 200).
fn place(gu: &GameUi, w: &World, kh: f32, sh: f32) -> (f32, f32) {
    let drawn = |name: &str| w.win_hits.borrow().iter().find(|h| h.0 == name).map(|h| h.1);
    let d = kh + 2.0;
    if !w.typing {
        if let Some(r) = drawn(super::macroedit::MACRO) {
            return (r[0], r[1] + r[3] + 2.0);
        }
        if let Some(r) = drawn(super::seacom::EDITOR) {
            return (r[0], r[1] - d);
        }
    }
    if let (Some(top), Some(bottom)) = (gu.log_top, gu.log_bottom) {
        let x = gu.window_xy("menu    logwindo", 0.0, 0.0).map_or(16.0, |p| p.0);
        return (x, (bottom - 32.0).min(top) - d);
    }
    if w.typing {
        return (16.0, sh - 38.0 - d - 32.0);
    }
    (32.0, 200.0)
}

/// The European caps' glyphs (they type nothing), by key, for the modern layer's labels.
fn glyph(page: u8, i: usize) -> Option<char> {
    let row = |s: &str, from: usize| s.chars().nth(i.checked_sub(from)?);
    match (page, i / 15) {
        (11, 3) => row("àáâäçèéêëìíîï", 45),
        (11, 4) => row("ñòóôöùúûüœß«»", 60),
        (12, 0) => row("°", 14),
        (12, 1) => row("€£", 28),
        (12, 3) => row("ÀÁÂÄÇÈÉÊËÌÍÎÏ", 45),
        (12, 4) => row("ÑÒÓÔÖÙÚÛÜŒß–—¡¿", 60),
        _ => None,
    }
}

/// Ours (the modern layer): key `i`'s label and whether it is inert (a European cap).
pub fn label(k: &Keyboard, i: usize) -> (String, bool) {
    match typed(k.page, i) {
        None => (glyph(k.page, i).map(String::from).unwrap_or_default(), true),
        Some(Typed::Byte(b' ')) => (String::new(), false),
        Some(t) => (text(t), false),
    }
}

/// Where it stands: fixed the first time it is asked (`place` gives it).
pub fn place_once(k: &Keyboard, place: impl FnOnce() -> (f32, f32)) -> (f32, f32) {
    match k.at.get() {
        Some(a) => a,
        None => {
            let a = place();
            k.at.set(Some(a));
            a
        }
    }
}

/// SHIFT's page is up (12 over 11, 7 over 8; in Japanese katakana over hiragana, 2 over 3, 5 over 4).
pub fn shifted(k: &Keyboard) -> bool {
    if ja() { matches!(k.page, 1 | 2 | 5) } else { matches!(k.page, 12 | 7) }
}

/// The Japanese keyboard's tabs and Config for the modern layer: (child, label).
pub fn ja_tabs() -> Option<[(u16, &'static str); 5]> {
    ja().then_some([(1, "かな"), (2, "英数"), (3, "記号"), (4, "Shift"), (0x54, "Config")])
}

/// The pointer on child `c` (the modern layer): the cursor moves there (SE 0001).
pub fn hover(w: &mut World, c: u16) {
    if let Some(k) = w.swkbd.as_mut().filter(|k| k.cursor != c) {
        k.cursor = c;
        w.sounds.push(Ui::Cursor as u32);
    }
}

/// A click on child `c` (the modern layer): A on it.
pub fn tap(w: &mut World, c: u16) {
    if let Some(k) = w.swkbd.as_mut() {
        k.cursor = c;
    }
    w.sounds.push(Ui::Select as u32);
    press(w, c);
}

/// win_keypad_v10 over the generic draw: the page's 75 caps at their child + (1, 1) (FUN_101b0d20: the cursor's
/// tinted 0x80805020, the European ones dim 0x40402810), in Japanese the caps of child 4 (the page's), SHIFT (0x13 on a
/// single-byte page, else 0xc), SPACE (0x13, else 0x1ad), Enter (0x1b1) and, on a single-byte page, the tabs 1..3
/// (0x11) (FUN_101b0dc0); the yubi hand on the cursor. Returns the frame.
pub fn draw(gu: &mut GameUi, w: &World, sw: f32, sh: f32) -> Option<[f32; 4]> {
    let k = w.swkbd.as_ref().filter(|_| !w.modern)?;
    let name = name();
    let s = scale(sw);
    let kh = gu.menu_def(name)?.elements[0].h as f32 * s;
    let at = place_once(k, || place(gu, w, kh, sh));
    let ids: Vec<u16> = gu.menu_def(name)?.elements.iter().skip(1).map(|e| e.id).collect();
    let hide: Vec<u16> = ids.iter().copied().filter(|&c| !sprite_kept(k, c)).collect();
    let sel = sprite_kept(k, k.cursor).then_some(k.cursor);
    let win = gu.window_scaled(name, at, s, sel, &hide)?;
    let kids: Vec<(u16, [f32; 4])> = gu
        .menu_def(name)?
        .elements
        .iter()
        .skip(1)
        .filter(|e| !off(e.id))
        .map(|e| (e.id, [at.0 + e.x as f32 * s, at.1 + e.y as f32 * s, e.w as f32 * s, e.h as f32 * s]))
        .collect();
    let p = page(k.page)?;
    let mut caps: Vec<(u16, u16, bool)> =
        (8..=0x52u16).map(|c| (c, p.caps[(c - 8) as usize], dim(p.caps[(c - 8) as usize]))).collect();
    if ja() {
        caps.push((4, p.c4, false));
        if p.single {
            caps.extend([
                (1, 0x11, false),
                (2, 0x11, false),
                (3, 0x11, false),
                (0x55, 0x13, false),
                (0x56, 0x13, false),
            ]);
        } else {
            caps.extend([(0x55, 0xc, false), (0x56, 0x1ad, false)]);
        }
        caps.push((0x57, 0x1b1, false));
    }
    let lib = caps_lib();
    for (id, cap, dim) in caps {
        let Some((_, r)) = kids.iter().find(|(c, _)| *c == id) else { continue };
        let tint = if id == k.cursor {
            0x8080_5020
        } else if dim {
            0x4040_2810
        } else {
            0x8080_8080
        };
        gu.sprite_scaled(lib, cap as usize, r[0] + s, r[1] + s, s, s, tint);
    }
    // The hand at the cursor child + the window header's (14, 14) *(inf)*.
    if let Some((_, r)) = kids.iter().find(|(id, _)| *id == k.cursor) {
        gu.sprite_scaled(HAND, 0, r[0] + 14.0 * s, r[1] + 14.0 * s, s, s, 0x8080_8080);
    }
    super::winhits::record_rects(w, name, win.frame, kids);
    Some(win.frame)
}

/// The mouse on the keyboard (the generic window's): hovering a key moves the cursor, a click presses it; the right
/// click does nothing here.
pub fn pointer(w: &mut World, gu: &GameUi, p: (f32, f32), clicked: bool) -> bool {
    let Some(cursor) = w.swkbd.as_ref().filter(|_| !w.modern).map(|k| k.cursor) else { return false };
    let name = name();
    if !super::winhits::over(w, name, p) {
        return false;
    }
    super::winhits::generic(
        w,
        gu,
        name,
        cursor,
        1..=0x57,
        p,
        clicked,
        false,
        |w, id| {
            if let Some(k) = w.swkbd.as_mut() {
                k.cursor = id;
            }
        },
        |w, gu, key| {
            if key == egui::Key::Enter {
                let c = w.swkbd.as_ref().map_or(8, |k| k.cursor);
                w.sounds.push(Ui::Select as u32);
                gu.press_on(name, c);
                press(w, c);
            }
        },
    );
    true
}

/// A zone change (win_keypad_v40 message 8): the keyboard closes, and comes back with the chat line (message 4).
pub fn zone_out(w: &mut World) {
    w.swkbd_reopen = w.swkbd.take().is_some() && w.typing;
}

pub fn zone_in(w: &mut World) {
    if std::mem::take(&mut w.swkbd_reopen) && w.typing {
        w.swkbd = Some(Keyboard::open());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(p: u8) -> &'static [u8; 75] {
        match &page(p).unwrap().keys {
            Keys::Bytes(b) => b,
            Keys::Pairs(_) => panic!("page {p} is kind 2"),
        }
    }

    fn typed_str(p: u8, r: std::ops::Range<usize>) -> String {
        r.map(|i| typed(p, i).map_or(String::from("."), text)).collect()
    }

    /// Outside Japanese (ours' default) page 11 is the chat line's: digits, lower case and its punctuation, the
    /// European caps dim and silent; SHIFT and X go to page 12 and back; the restricted pages keep themselves.
    #[test]
    fn pages() {
        assert!(!ja());
        assert_eq!(typed_str(11, 0..15), "1234567890.,'\"&");
        assert_eq!(typed_str(11, 15..30), "abcdefghijklm@#");
        assert_eq!(typed_str(11, 43..47), "()..");
        assert_eq!(typed_str(11, 58..60), "[]");
        assert_eq!(typed_str(11, 73..75), "!?");
        assert_eq!(typed_str(12, 0..16), "+-*/=$\\%:;^~_|.A");
        let p = page(11).unwrap();
        assert!(dim(p.caps[45]) && !dim(p.caps[15]));
        // 0x114, 0x132 and 0x133 are dim outside Japanese only.
        assert!(dim(0x114) && dim(0x133) && dim(0x15a) && !dim(0x113));
        assert_eq!(&bytes(8)[..15], b"1234567890:;_/@");
        assert_eq!((first_page(1, 5), first_page(1, 2), first_page(1, 1), first_page(1, 3)), (11, 11, 10, 6));
        assert_eq!((first_page(1, 0), first_page(2, 5), first_page(0, 5), first_page(1, 9)), (9, 9, 11, 11));
        let mut pg = 11;
        assert!(turn(&mut pg, false) && pg == 12 && turn(&mut pg, true) && pg == 11);
        let mut pg = 6;
        assert!(!turn(&mut pg, false) && !turn(&mut pg, true) && pg == 6);
        // Mode 1 never turns to the Japanese pages 0, 1, 2, 4; the tabs choose 3 or 5 only.
        assert!(!turnable(0) && !turnable(2) && !turnable(4) && turnable(3) && turnable(12) && !turnable(13));
        let mut pg = 3;
        assert!(!choose(&mut pg, 0) && !choose(&mut pg, 3) && choose(&mut pg, 5) && pg == 5);
    }

    /// FUN_101b5a20 over the table: a full-width letter is its byte, the full-width space a space, a European letter
    /// flagged, ￥ (0x8f81) none; the kana pages type their pairs when not made bytes.
    #[test]
    fn pairs() {
        assert_eq!(pair_byte(0x4081), (b' ', false));
        assert_eq!(pair_byte(0x8182), (b'a', false));
        assert_eq!(pair_byte(0x8f81), (0, false));
        assert!(pair_byte(0x9f85).1);
        assert_eq!(typed_str(2, 15..18), "abc");
        assert_eq!(typed_str(5, 26..28), "￥$");
        assert_eq!(
            text(Typed::Pair(match &page(0).unwrap().keys {
                Keys::Pairs(v) => v[0],
                Keys::Bytes(_) => unreachable!(),
            })),
            "あ"
        );
    }

    /// The page table (0x1037d320) and the byte tables (0x1037d568, 0x1037d730) are FFXiMain's. Needs the installed
    /// game; skipped without it.
    #[test]
    fn tables_are_ffximain() {
        use crate::dat::battle::va_bytes;
        let dir = crate::config::Config::load().ffxi_dir();
        let Ok(dll) = std::fs::read(crate::dat::ffximain(&dir)) else { return };
        let u32_at = |va: u32| u32::from_le_bytes(va_bytes(&dll, va, 4).unwrap().try_into().unwrap());
        let u16_at = |va: u32| u16::from_le_bytes(va_bytes(&dll, va, 2).unwrap().try_into().unwrap());
        for (n, p) in PAGES.iter().enumerate() {
            let rec: Vec<u32> = (0..7).map(|i| u32_at(0x1037d320 + 4 * (7 * n as u32 + i))).collect();
            let caps: Vec<u16> = (0..75).map(|i| u32_at(rec[1] + 4 * i) as u16).collect();
            assert_eq!(caps, p.caps, "page {n} caps");
            match &p.keys {
                Keys::Bytes(b) => assert_eq!((rec[2], &b[..]), (1, va_bytes(&dll, rec[0], 75).unwrap()), "page {n}"),
                Keys::Pairs(v) => {
                    assert_eq!(rec[2], 2, "page {n}");
                    assert!((0..75).all(|i| u16_at(rec[0] + 2 * i as u32) == v[i]), "page {n} keys");
                }
            }
            assert_eq!(rec[3..], [p.c4 as u32, p.single as u32, p.shift as u32, p.x as u32], "page {n}");
        }
        assert!((0..0xe0).all(|i| u16_at(0x1037d568 + 2 * i as u32) == BYTE_PAIRS[i]));
        assert!((0..0x60).all(|i| u16_at(0x1037d730 + 2 * i as u32) == BYTE_PAIRS_HIGH[i]));
    }

    /// Both keyboards' layouts are in their language's menu file with the children the code names, and every cap a
    /// page or a special key draws is in its library. Needs the installed game; skipped without it.
    #[test]
    fn layouts_and_caps() {
        use crate::dat::ui::{MENU, UiFile};
        let Ok(index) = crate::dat::DatIndex::open(&crate::config::Config::load().ffxi_dir()) else { return };
        for (file, name, lib, pages) in [
            (MENU[0], "menu    keypad", "menu    key1top", 0..6),
            (MENU[1], "menu    keypadus", "menu    key1topu", 6..13),
        ] {
            let Ok(d) = index.read(file) else { continue };
            let ui = UiFile::parse(&d).unwrap();
            let ids: Vec<u16> = ui.menu(name).unwrap().elements.iter().skip(1).map(|e| e.id).collect();
            assert_eq!(ids, (1..=0x57).collect::<Vec<_>>(), "{name}");
            let n = ui.libs.iter().find(|l| l.name == lib).unwrap().sprites.len();
            for p in pages {
                let pg = page(p).unwrap();
                assert!(pg.caps.iter().chain([&pg.c4]).all(|&c| (c as usize) < n), "page {p}");
            }
            assert!([0x11usize, 0x13, 0xc, 0x1ad, 0x1b1].iter().all(|&c| c < n));
        }
    }
}
