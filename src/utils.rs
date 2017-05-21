use std::fmt;

pub struct PrettyHex<'a> {
    data: &'a [u8],
    pub show_col_num: bool,
    pub show_row_num: bool,
    pub show_ascii_chrs: bool,
}

impl<'a> PrettyHex<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        PrettyHex {
            data: data,
            show_col_num: true,
            show_row_num: true,
            show_ascii_chrs: false,
        }
    }

    pub fn with_opts(mut self, options: (bool, bool, bool)) -> Self {
        self.show_col_num = options.0;
        self.show_row_num = options.1;
        self.show_ascii_chrs = options.2;
        self
    }

    #[inline]
    fn display_hex_header(f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "\r\n      "));
        for i in 0..0x08 {
            try!(write!(f, "{:02x} ", i))
        }
        try!(write!(f, " "));
        for i in 0x08..0x10 {
            try!(write!(f, "{:02x} ", i))
        }
        write!(f, "\r\n")
    }

    #[inline]
    fn display_ascii_chrs(f: &mut fmt::Formatter, chrs: &[u8]) -> fmt::Result {
        try!(write!(f, " "));
        for (i, u) in chrs.iter().enumerate() {
            if i % 0x10 == 0x08 {
                try!(write!(f, " "))
            }

            let c = *u as char;
            if c.is_control() {
                try!(write!(f, "."))
            } else {
                try!(write!(f, "{}", c))
            }
        }
        Ok(())
    }
}

impl<'a> fmt::Display for PrettyHex<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut ascii_buf = [0u8; 0x10];
        if self.show_col_num {
            PrettyHex::display_hex_header(f).unwrap();
        } else {
            try!(write!(f, "\r\n"))
        }
        for (i, ref b) in self.data.iter().enumerate() {
            if self.show_row_num {
                if i % 0x10 == 0 {
                    try!(write!(f, "{:04x}  ", i))
                }
            }
            if i % 0x10 == 0x08 {
                try!(write!(f, " "))
            }
            try!(write!(f, "{:02x} ", b));
            ascii_buf[i % 0x10] = **b;

            if i % 0x10 == 0x0f {
                if self.show_ascii_chrs {
                    PrettyHex::display_ascii_chrs(f, &ascii_buf).unwrap();
                }
                try!(write!(f, "\r\n"))
            }
        }
        let rem = self.data.len() % 0x10;
        if rem != 0 {
            if self.show_ascii_chrs {
                let white_space_len = (0x10 - rem) * 3 + if rem < 0x08 { 1 } else { 0 };
                for _ in 0..white_space_len {
                    try!(write!(f, " "))
                }
                PrettyHex::display_ascii_chrs(f, &ascii_buf[0..rem]).unwrap();
            }
            try!(write!(f, "\r\n"))
        }
        Ok(())
    }
}
