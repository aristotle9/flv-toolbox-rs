extern crate rustc_serialize;
extern crate getopts;
extern crate byteorder;
extern crate xml;
extern crate flv_toolbox_rs;

use std::io::Write;
use flv_toolbox_rs::lib::{FLVTag, write_flv_config};

use getopts::Options;

fn flv_config(flvs: &Vec<String>, config_path: &String, url_prefix: &String, json: bool) {
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
    if json {
        write_flv_config_json(&mut file, &info_vec, flvs, timelength, url_prefix);
    } else {
        write_flv_config(&mut file, &info_vec, flvs, timelength, url_prefix);
    }
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
    opts.optflag("j", "json", "output as json format");

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
    let json: bool = matches.opt_present("j");
    let config = match matches.opt_default("c", &format!("config.{}", if json { "json" } else { "xml" })) {
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
    flv_config(&matches.free, &config, &url_prefix, json);
}

pub fn write_flv_config_json<W: Write>(w: &mut W, info_vec: &Vec<(u64, u64)>, flvs: &Vec<String>, timelength: u64, url_prefix: &String) {
    
    use rustc_serialize::json::{as_pretty_json, Json};
    use std::collections::BTreeMap;
    
    let mut ret: BTreeMap<String, Json> = BTreeMap::new();
    
    ret.insert("timelength".to_string(), Json::U64(timelength));
    
    let mut durl: Vec<Json> = Vec::with_capacity(info_vec.len());
    for (path, &(t, s)) in flvs.iter().zip(info_vec.iter()) {
        let mut obj: BTreeMap<String, Json> = BTreeMap::new();
        obj.insert("length".to_string(), Json::U64(t));
        obj.insert("size".to_string(), Json::U64(s));
        obj.insert("url".to_string(), Json::String(format!("{}{}", url_prefix, path)));
        durl.push(Json::Object(obj));
    }

    ret.insert("durl".to_string(), Json::Array(durl));
    
    write!(w, "{}", as_pretty_json(&Json::Object(ret)));
}