thread_local! {
    static CRC_TABLE: [u32; 256] = make_table();
}

pub struct Crc32 {
    value: u32
}

fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    for i in 0..256 {
        let mut value = i as u32;
        for _ in 0..8 {
            value = if (value & 1) == 1 {
                (value >> 1) ^ 0xedb88320
            } else {
                value >> 1
            }
        }
        table[i] = value
    }
    table
}

impl Crc32 {
    
    pub fn new() -> Crc32 {
        Crc32 {
            value: 0
        }
    }

    pub fn reset(&mut self) {
        self.value = 0
    }

    pub fn update(&mut self, bytes: &[u8]) {
        CRC_TABLE.with(|table| {
            let mut value = !self.value;
            for &i in bytes.iter() {
                value = table[((value as u8) ^ i) as usize] ^ (value >> 8)
            }
            self.value = !value;
        })
    }

    pub fn finish(&self) -> u32 {
        self.value
    }
}

#[test]
fn test_crc32() {
    let mut crc32 = Crc32::new();
    crc32.update(b"123456789");
    assert_eq!(crc32.finish(), 0xcbf43926);
}