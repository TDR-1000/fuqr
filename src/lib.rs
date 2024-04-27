use std::cmp::min;

use error_correction::{ECL, GEN_POLYNOMIALS, NUM_BLOCKS, NUM_CODEWORDS};
use math::{ANTILOG_TABLE, LOG_TABLE};
use qr::{encode_alphanumeric, QRCode, Symbol};
use version::Version;

use crate::version::format_information;

pub mod error_correction;
pub mod math;
pub mod qr;
pub mod version;

pub fn encode(input: &str) -> QRCode {
    let mut qrcode = QRCode {
        data: Vec::new(),
        ecl: ECL::Low,
        mask: 1,
        version: Version(1),
    };

    // todo, ensure version can contain before encode, mathable

    encode_alphanumeric(&mut qrcode, "GREETINGS TRAVELER");
    let modules = qrcode.version.num_data_modules();
    let codewords = modules / 8;
    let remainder_bits = modules % 8;

    let num_ec_codewords = NUM_CODEWORDS[qrcode.ecl as usize][qrcode.version.0 as usize] as usize;
    let num_data_codewords = codewords - num_ec_codewords;

    // terminator
    let remainder_data_bits = (num_data_codewords * 8) - (qrcode.data.len());
    qrcode.push_bits(0, min(4, remainder_data_bits));

    // byte align
    let byte_pad = (8 - (qrcode.data.len() % 8)) % 8;
    qrcode.push_bits(0, byte_pad);

    // fill data capacity
    let data_pad = num_data_codewords - (qrcode.data.len() / 8);
    let mut alternating_byte = 0b11101100;
    for _ in 0..data_pad {
        qrcode.push_bits(alternating_byte, 8);
        alternating_byte ^= 0b11111101;
    }

    let blocks = NUM_BLOCKS[qrcode.ecl as usize][qrcode.version.0 as usize] as usize;

    let group_2_blocks = codewords % blocks;
    let group_1_blocks = blocks - group_2_blocks;

    let data_codeword_vec = qrcode.get_u8_data();
    let data_codewords = data_codeword_vec.as_slice();

    let data_per_g1_block = num_data_codewords / blocks;
    let data_per_g2_block = data_per_g1_block + 1;

    let ecc_per_block = num_ec_codewords / blocks;
    let mut final_sequence = vec![0; codewords];

    for i in 0..group_1_blocks * data_per_g1_block {
        let col = i % data_per_g1_block;
        let row = i / data_per_g1_block;
        final_sequence[col * blocks + row] = data_codewords[i];
    }
    for i in 0..group_2_blocks * data_per_g2_block {
        let col = i % data_per_g2_block;
        let row = i / data_per_g2_block;
        final_sequence[col * blocks + row + group_1_blocks] = data_codewords[i];
    }

    for i in 0..group_1_blocks {
        let ec_codewords = remainder(
            &data_codewords[(i * data_per_g1_block)..(i + data_per_g1_block)],
            &GEN_POLYNOMIALS[ecc_per_block][..ecc_per_block],
        );

        for j in 0..ec_codewords.len() {
            final_sequence[num_data_codewords + j * blocks + i] = ec_codewords[j];
        }
    }

    let group_2_start = group_1_blocks * data_per_g1_block;

    for i in 0..group_2_blocks {
        let ec_codewords = remainder(
            &data_codewords[(group_2_start + i * data_per_g2_block)..(i + data_per_g2_block)],
            &GEN_POLYNOMIALS[ecc_per_block][..ecc_per_block],
        );

        for j in 0..ec_codewords.len() {
            final_sequence[num_data_codewords + j * blocks + i + group_1_blocks] = ec_codewords[j];
        }
    }

    qrcode
}

pub fn place(qrcode: &QRCode) -> Symbol {
    let mut symbol = Symbol::new(qrcode.version.0);
    let width = qrcode.version.0 as usize * 4 + 17;

    fn place_finder(symbol: &mut Symbol, mut row: usize, col: usize) {
        for i in 0..7 {
            symbol.set(row, col + i);
        }
        row += 1;

        symbol.set(row, col + 0);
        symbol.set(row, col + 6);
        row += 1;

        for _ in 0..3 {
            symbol.set(row, col + 0);

            symbol.set(row, col + 2);
            symbol.set(row, col + 3);
            symbol.set(row, col + 4);

            symbol.set(row, col + 6);
            row += 1;
        }

        symbol.set(row, col + 0);
        symbol.set(row, col + 6);

        row += 1;
        for i in 0..7 {
            symbol.set(row, col + i);
        }
    }

    fn place_format(symbol: &mut Symbol, format_info: u32) {
        let first_coords = [
            [0, 8],
            [1, 8],
            [2, 8],
            [3, 8],
            [4, 8],
            [5, 8],
            [7, 8],
            [8, 8],
            [8, 7],
            [8, 5],
            [8, 4],
            [8, 3],
            [8, 2],
            [8, 1],
            [8, 0],
        ];
        let second_coords = [
            [8, symbol.width - 1],
            [8, symbol.width - 2],
            [8, symbol.width - 3],
            [8, symbol.width - 4],
            [8, symbol.width - 5],
            [8, symbol.width - 6],
            [8, symbol.width - 7],
            [8, symbol.width - 8],
            [symbol.width - 7, 8],
            [symbol.width - 6, 8],
            [symbol.width - 5, 8],
            [symbol.width - 4, 8],
            [symbol.width - 3, 8],
            [symbol.width - 2, 8],
            [symbol.width - 1, 8],
        ];
        for i in 0..first_coords.len() {
            if (format_info & (1 << i)) != 0 {
                symbol.set(first_coords[i][0], first_coords[i][1]);
            }
        }
        for i in 0..second_coords.len() {
            if (format_info & (1 << i)) != 0 {
                symbol.set(second_coords[i][0], second_coords[i][1]);
            }
        }

        // always set
        symbol.set(symbol.width - 8, 8);
    }

    place_finder(&mut symbol, 0, 0);
    place_finder(&mut symbol, 0, width - 7);
    place_finder(&mut symbol, width - 7, 0);

    let format_info = format_information(qrcode);
    place_format(&mut symbol, format_info);
    symbol
}

fn remainder(data: &[u8], generator: &[u8]) -> Vec<u8> {
    let num_codewords = generator.len();

    let mut base = [0; 60];
    base[..data.len()].copy_from_slice(data);

    for i in 0..data.len() {
        if base[i] == 0 {
            continue;
        }

        let alpha_diff = ANTILOG_TABLE[data[i] as usize];

        for j in 0..generator.len() {
            base[i + j + 1] ^= LOG_TABLE[(generator[j] as usize + alpha_diff as usize) % 255];
        }
    }

    base[data.len()..(data.len() + num_codewords)].to_vec()
}
