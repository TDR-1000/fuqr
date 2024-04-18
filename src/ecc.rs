use crate::qr::{QRCode, ECL};

const fn make_ecc_table() -> [[u16; 41]; 4] {
    let mut table = [[0; 41]; 4];
    table[ECL::Low as usize] = L_TABLE;
    table[ECL::Medium as usize] = M_TABLE;
    table[ECL::Quartile as usize] = Q_TABLE;
    table[ECL::High as usize] = H_TABLE;
    table
}
pub const ECC_TABLE: [[u16; 41]; 4] = make_ecc_table();

const L_TABLE: [u16; 41] = [
    0, 7, 10, 15, 20, 26, 36, 40, 48, 60, 72, 80, 96, 104, 120, 132, 144, 168, 180, 196, 224, 224,
    252, 270, 300, 312, 336, 360, 390, 420, 450, 480, 510, 540, 570, 570, 600, 630, 660, 720, 750,
];
const M_TABLE: [u16; 41] = [
    0, 10, 16, 26, 36, 48, 64, 72, 88, 110, 130, 150, 176, 198, 216, 240, 280, 308, 338, 364, 416,
    442, 476, 504, 560, 588, 644, 700, 728, 784, 812, 868, 924, 980, 1036, 1064, 1120, 1204, 1260,
    1316, 1372,
];
const Q_TABLE: [u16; 41] = [
    0, 13, 22, 36, 52, 72, 96, 108, 132, 160, 192, 224, 260, 288, 320, 360, 408, 448, 504, 546,
    600, 644, 690, 750, 810, 870, 952, 1020, 1050, 1140, 1200, 1290, 1350, 1440, 1530, 1590, 1680,
    1770, 1860, 1950, 2040,
];
const H_TABLE: [u16; 41] = [
    0, 17, 28, 44, 64, 88, 112, 130, 156, 192, 224, 264, 308, 352, 384, 432, 480, 532, 588, 650,
    700, 750, 816, 900, 960, 1050, 1110, 1200, 1260, 1350, 1440, 1530, 1620, 1710, 1800, 1890,
    1980, 2100, 2220, 2310, 2430,
];

pub fn get_blocks(qrcode: &QRCode) -> u32 {
    if qrcode.ecc == ECL::Medium {
        match qrcode.version.0 {
            15 => return 10,
            19 => return 14,
            38 => return 45,
            _ => (),
        }
    }

    let total = qrcode.version.num_codewords();
    let error = ECC_TABLE[qrcode.ecc as usize][qrcode.version.0 as usize] as u32;
    // let data = total - error;

    let error = error / 2;
    if error <= 15 {
        return 1;
    }
    for i in (8..=15).rev() {
        if error % i == 0 {
            let res = error / i;
            if res == 3 {
                // edgecases or pattern? idk
                continue;
            }
            return res;
        }
    }

    unreachable!("num blocks not found");
}
