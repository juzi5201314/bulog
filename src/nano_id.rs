use smol_str::{SmolStr, SmolStrBuilder};

const ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i',
    'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U',
    'V', 'W', 'X', 'Y', 'Z',
];

const MASK: usize = ALPHABET.len().next_power_of_two() - 1;
const _: () = assert!(MASK == 63);

pub fn nanoid(size: usize) -> SmolStr {
    let mut id = SmolStrBuilder::new();
    let mut len = 0;

    loop {
        let mut bytes = [0; 64];
        fastrand::fill(&mut bytes);

        for byte in bytes {
            let byte = byte as usize & MASK;
            if ALPHABET.len() > byte {
                id.push(ALPHABET[byte]);
                len += 1;

                if len == size {
                    return id.finish();
                }
            }
        }
    }
}
