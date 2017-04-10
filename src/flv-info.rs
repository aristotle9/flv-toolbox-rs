#![feature(path_ext)]
extern crate rustc_serialize;
extern crate getopts;
extern crate xml;

use rustc_serialize::json::Json;
use getopts::Options;

mod lib;
use lib::*;

fn print_metatag(json: &Json) -> Result<(), Option<String>> {
    let event_name = json.as_array().ok_or(None)?[0].as_string().ok_or(None)?;
    let obj = &json.as_array().ok_or(None)?[1];
    let times = obj.find_path(&["keyframes", "times"]).ok_or(None)?.as_array().ok_or(None)?;
    let times: Vec<f64> = times.iter().map(|val: &Json| {
        val.as_f64().unwrap()
    }).collect();
    let filepositions = obj.find_path(&["keyframes", "filepositions"]).ok_or(None)?.as_array().ok_or(None)?;
    let filepositions: Vec<u64> = filepositions.iter().map(|val: &Json| {
        val.as_f64().unwrap() as u64
    }).collect();

    println!("metadata: {}", event_name);
    println!("{}", rustc_serialize::json::as_pretty_json(&obj));
    for (i, (t, p)) in (0u32..).zip(times.iter().zip(filepositions.iter())) {
        println!("{:3} {} {:8}", i, format_seconds_ms((t * 1000f64) as u64), p);
    }
    Ok(())
}

fn flv_info(path: &String, show_meta: bool, all_frame: bool) {
    use std::fs::File;
    use std::path::Path;
    use std::fs;

    let path = Path::new(path);
    if fs::metadata(path).is_err() {
        panic!(format!("file dosen't exist: {}", path.display()));
    }
    else {
        println!("show info for {}", path.display());
    }
    let mut file = File::open(path).unwrap();
    let file_meta = file.metadata().unwrap();
    let file_size = file_meta.len();
    println!("file size: {}", file_size);
    let mut parser = FLVTagRead::new(&mut file);//header has read

    println!("\r\ntags:", );
    let mut i = 1;
    loop {
        let position = parser.get_position();
        let tag = parser.next();
        if tag.is_none() {
            break;
        }
        let tag = tag.unwrap();
        match tag.get_tag_type() {
            FLVTagType::TAG_TYPE_VIDEO => {
                if tag.get_frame_type() == 1 {//key frames
                    println!("{:?}", (i, format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), position));
                    i += 1;
                }
                else if all_frame {
                    println!("{:?}", (i, format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), position));
                    i += 1;
                }
            },
            FLVTagType::TAG_TYPE_AUDIO => {
                if tag.is_acc_sequence_header() {
                    println!("{:?}", (i, format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_data_size(), position));
                    i += 1;
                }
                else if all_frame {
                    println!("{:?}", (i, format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_data_size(), position));
                    i += 1;
                }
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                if show_meta {
                    print_metatag(&Json::Array(tag.get_objects()));
                } else {
                    println!("metatag");
                }
                i += 1;
            }
        };
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("m", "meta", "show metadata");
    opts.optflag("a", "all", "print all frames");
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
    let input = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };
    let show_meta = matches.opt_present("m");
    let all_frame = matches.opt_present("a");
    flv_info(&input, show_meta, all_frame);
}
