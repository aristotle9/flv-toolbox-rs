#![feature(path_ext)]
extern crate rustc_serialize;
extern crate getopts;
extern crate xml;

use std::fs::File;

use rustc_serialize::json::{Json, ToJson};
use getopts::Options;

mod lib;
use lib::*;

const PROGRAM_SIGN: &'static str = "modified by flv-split, 2015";

#[inline]
fn delta(v1: u64, v2: u64) -> u64 {
    if v1 >= v2 {
        v1 - v2
    } else {
        v2 - v1
    }
}

//扫描关键点的视频音频位置信息
fn flv_scan(file: &mut File, verbose: bool, min: u64, win: u64) -> Vec<(u64, u64, u64, u64)> {//video offset, next audio offset, position
    use std::io::SeekFrom;
    use std::io::Seek;

    let header = FLVHeader::read(file);
    let mut metatag = FLVTag::read(file).expect("read meta tag err");
    assert_eq!(metatag.get_tag_type(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);

    let v = metatag.get_objects();
    let _event_name = v[0].as_string().unwrap().to_string();
    let filepositions = v[1].find_path(&["keyframes", "filepositions"]).unwrap().as_array().unwrap();
    let filepositions = filepositions.iter().map(|val: &Json| {
        val.as_f64().unwrap() as u64
    }).collect::<Vec<u64>>();

    let mut info_vec: Vec<(u64, u64, u64)> = Vec::with_capacity(filepositions.len());
    for (i, pos) in filepositions.iter().enumerate() {
        file.seek(SeekFrom::Start(*pos)).expect("seek flv file err");
        let ktag = FLVTag::read(file).expect("read video keyframe err");
        let atag = FLVTag::read(file).expect("read next auto tag err");
        let t1 = ktag.get_timestamp();
        let t2 = if atag.get_tag_type() == FLVTagType::TAG_TYPE_AUDIO {
            atag.get_timestamp()
        } else {
            let mut tag = FLVTag::read(file);
            while tag.is_some() && tag.as_ref().unwrap().get_tag_type() != FLVTagType::TAG_TYPE_AUDIO {
                tag = FLVTag::read(file);
            }
            if tag.is_none() {
                t1 + 100
            } else {
                tag.as_ref().unwrap().get_timestamp()
            }
        };
        let dt = delta(t1, t2);
        info_vec.push((t1, dt, *pos));
        if verbose {
            println!("{:3} {} {:8} {:8} {:8}", i, format_seconds_ms(t1), t1, dt, *pos);
        }
    }

    let vec = split_flv_by_min(&info_vec, min, win);
    if verbose {
        println!("{:?}", vec.iter().map(|&(t, p, n, dt)| (format_seconds_ms(t), p, n, dt)).collect::<Vec<(String, u64, u64, u64)>>());
    }

    file.seek(SeekFrom::Start(0)).expect("flv seek err");
    vec
}

fn flv_split(path: &String, min: u64, win: u64, prefix: &String, verbose: bool, config_path: &String, url_prefix: &String) {
    use std::fs::File;
    use std::fs::PathExt;
    use std::path::Path;

    let path = Path::new(path);
    if !path.exists() {
        panic!(format!("file dosen't exist: {}", path.display()));
    } else {
        println!("begin to split flv {}. each file is {} min(s), with name {}[n].flv. partial config file is {}", path.display(), min, prefix, config_path);
    }

    let mut file = File::open(path).unwrap();
    let file_meta = file.metadata().unwrap();
    let file_size = file_meta.len();
    println!("file size: {}", file_size);
    //split config
    let vec = flv_scan(&mut file, verbose, min, win);

    let mut parser = FLVTagRead::new(&mut file);//header has read
    let mut metatag = parser.next().unwrap();
    assert_eq!(metatag.get_tag_type(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);

    let video_metatag = parser.next().unwrap();
    assert_eq!(video_metatag.get_tag_type(), FLVTagType::TAG_TYPE_VIDEO);
    assert_eq!(video_metatag.get_avc_packet_type(), 0);//avc tag

    let audio_metatag = parser.next().unwrap();
    assert_eq!(audio_metatag.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
    assert!(audio_metatag.is_acc_sequence_header());

    //init
    assert_eq!(vec[0].1, parser.get_position());//begin first real frames
    let mut seg_index: i64 = -1;
    let mut tag_write: Option<FLVTagWrite<File>> = None;
    let mut time_offset: u64 = 0;
    let mut timestamp: u64 = 0;
    let mut tag_index: u64 = 0;
    let mut times: Option<Vec<u64>> = None;
    let mut filepositions: Option<Vec<u64>> = None;
    let mut duration_filesize: Vec<(u64, u64)> = Vec::new();

    fn write_back_meta_tag(duration: u64, metatag: &mut FLVTag, times: &Vec<u64>, filepositions: &Vec<u64>, tag_write: &mut FLVTagWrite<File>) {
        let mut metas = metatag.get_objects();
        {
            metas[1].as_object_mut().unwrap().insert("duration".to_string(), Json::F64(duration as f64 / 1000.0));
            metas[1].as_object_mut().unwrap().insert("metadatacreator".to_string(), PROGRAM_SIGN.to_string().to_json());
            let key_times = Json::Array(times.iter().map(|&t| Json::F64(t as f64 / 1000.0)).collect::<Vec<Json>>());
            let key_positions = Json::Array(filepositions.iter().map(|&p| Json::F64(p as f64)).collect::<Vec<Json>>());
            let keyframes = metas[1].as_object_mut().unwrap().get_mut("keyframes").unwrap().as_object_mut().unwrap();
            keyframes.insert("times".to_string(), key_times);
            keyframes.insert("filepositions".to_string(), key_positions);
        }
        metatag.set_objects(&metas);
        tag_write.write_meta_tag(&metatag);
    }

    loop {
        let position = parser.get_position();
        let tag = parser.next();
        if seg_index < vec.len() as i64 - 1 && vec[(seg_index + 1) as usize].1 == position {
            //fillback metatag
            if tag_write.is_some() {
                write_back_meta_tag(tag.as_ref().unwrap().get_timestamp() - time_offset, &mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write.as_mut().unwrap());
                duration_filesize.push((tag.as_ref().unwrap().get_timestamp() - time_offset, tag_write.as_ref().unwrap().get_position()));
            }
            seg_index += 1;
            let file_name = format!("{}{}.flv", prefix, seg_index + 1);
            tag_write = Some(FLVTagWrite::new(File::create(file_name).unwrap()));
            let tag_write = tag_write.as_mut().unwrap();
            let key_tag_len = (vec[seg_index as usize].2 + 1) as usize;
            times = Some(vec![0; key_tag_len]);
            filepositions = Some(vec![0; key_tag_len]);
            tag_write.write_header(&parser.header);
            //modify metatag
            write_back_meta_tag(0, &mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write);
            filepositions.as_mut().unwrap()[0] = tag_write.get_position();
            tag_write.write_tag(&video_metatag);
            tag_write.write_tag(&audio_metatag);
            tag_index = 1;
            time_offset = tag.as_ref().unwrap().get_timestamp();
        }

        if tag.is_none() {
            write_back_meta_tag(timestamp - time_offset, &mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write.as_mut().unwrap());
            duration_filesize.push((timestamp - time_offset, tag_write.as_ref().unwrap().get_position()));
            break;
        }
        let mut tag = tag.unwrap();
        //change tag timestamp
        timestamp = tag.get_timestamp();
        tag.set_timestamp(timestamp - time_offset);
        if tag.get_tag_type() == FLVTagType::TAG_TYPE_VIDEO && tag.get_frame_type() == 1 {
            // println!("write keyframe tag {}/{}", tag_index, filepositions.as_ref().unwrap().len() - 1);
            filepositions.as_mut().unwrap()[tag_index as usize] = tag_write.as_ref().unwrap().get_position();
            times.as_mut().unwrap()[tag_index as usize] = timestamp - time_offset;
            tag_index += 1;
        }
        tag_write.as_mut().unwrap().write_tag(&tag);
    }

    //output partial config
    let mut file = File::create(config_path).unwrap_or_else(|e| {
        panic!(format!("try to create config file {}, but {}", config_path, e))
    });
    write_flv_config(&mut file, &duration_filesize, &(0..vec.len()).map(|i| format!("{}{}.flv", prefix, i + 1)).collect(), timestamp, url_prefix);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("m", "min", "set the number of minutes for each part, default is 6", "MINS");
    opts.optflagopt("w", "win", "set the number of seconds to search split point, default is 20", "WIN");
    opts.optflagopt("p", "prefix", "set the prefix name of part, default is \"seg-\"", "PREFIX");
    opts.optflagopt("c", "config", "set partial config file name, default is PREFIXconfig.xml", "CONFIG");
    opts.optflagopt("u", "url-prefix", "set url-prefix, default is none", "URL_PREFIX");
    opts.optflag("v", "verbose", "show more information");
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
    let min = match matches.opt_default("m", "6") {
        Some(m_str) => std::str::FromStr::from_str(&m_str).unwrap(),
        _ => 6
    };
    let win = match matches.opt_default("w", "20") {
        Some(win_str) => std::str::FromStr::from_str(&win_str).unwrap(),
        _ => 20
    };
    let prefix = match matches.opt_default("p", "seg-") {
        Some(s) => s,
        _ => {
            print_usage(&program, opts);
            return;
        }
    };
    let config = match matches.opt_default("c", &format!("{}config.xml", prefix)) {
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
    let verbose = matches.opt_present("v");
    flv_split(&input, min, win, &prefix, verbose, &config, &url_prefix);
}
