extern crate rustc_serialize;
extern crate getopts;
extern crate byteorder;
extern crate xml;

mod lib;

use lib::{FLVTag, write_flv_config};

use getopts::Options;

fn flv_config(flvs: &Vec<String>, config_path: &String, url_prefix: &String) {
    use std::fs::File;
    use std::io::{Seek, SeekFrom};
    use self::byteorder::{BigEndian, ReadBytesExt};

    let mut info_vec = Vec::<(u64, u64)>::new();
    let mut timelength = 0;
    for path in flvs.iter() {
        let mut file = File::open(path).unwrap_or_else(|e| {
            panic!(format!("try to open file {}, but {}", path, e))
        });
        file.seek(SeekFrom::End(-4)).expect("seek last tag size error");
        let last_tag_size = file.read_u32::<BigEndian>().expect("read last u32 error");
        let size = file.seek(SeekFrom::Current(0)).expect("get seek pos error");
        file.seek(SeekFrom::End(-(last_tag_size as i64) - 4)).expect("seek last tag error");
        let tag = FLVTag::read(&mut file).expect("last tag read error");
        timelength += tag.get_timestamp();
        info_vec.push((tag.get_timestamp(), size));
    }

    let mut file = File::create(config_path).unwrap_or_else(|e| {
        panic!(format!("try to create config file {}, but {}", config_path, e))
    });
    write_flv_config(&mut file, &info_vec, flvs, timelength, url_prefix);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE1 [FILE2 ...] [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("c", "config", "set partial config file name, default is config.xml", "CONFIG");
    opts.optflagopt("u", "url-prefix", "set url-prefix, default is none", "URL_PREFIX");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.free.is_empty() {
        print_usage(&program, opts);
        return;
    }
    let config = match matches.opt_default("c", "config.xml") {
        Some(c) => c,
        _ => {
            print_usage(&program, opts);
            return;
        }
    };
    let url_prefix = match matches.opt_default("u", "") {
        Some(c) => c,
        _ => {
            print_usage(&program, opts);
            return;
        }
    };
    flv_config(&matches.free, &config, &url_prefix);
}
