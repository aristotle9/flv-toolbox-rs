#![feature(path_ext)]
extern crate rustc_serialize;
extern crate getopts;
extern crate xml;
extern crate colored;

use colored::*;
use rustc_serialize::json::Json;
use getopts::Options;

mod lib;
use lib::*;

mod crc32;
use crc32::Crc32;

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

fn flv_info(path: &String, show_meta: bool, all_frame: bool, video_frame: bool, audio_frame: bool) {
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

    println!("\r\ntags: kf: key_frame cd: codec_id pt: packet_type", );
    if audio_frame {
        println!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6}", "id", "time", "offset", "size", "type", "kf", "cd", "sr", "cts", "dts", "pts", "ddts");
    }
    println!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6}", "id", "time", "offset", "size", "type", "kf", "cd", "pt", "cts", "dts", "pts", "ddts");
    let mut i = 0;
    let mut last_v_tag: Option<FLVTag> = None;
    let mut last_a_tag: Option<FLVTag> = None;
    let mut asc: Option<AudioSpecificConfig> = None;
    loop {
        let position = parser.get_position();
        let tag = parser.next();
        if tag.is_none() {
            break;
        }
        let tag = tag.unwrap();
        match tag.get_tag_type() {
            FLVTagType::TAG_TYPE_VIDEO => {
                let dts_delta: i64 = if last_v_tag.is_some() {
                    (tag.get_timestamp() as i64) - (last_v_tag.unwrap().get_timestamp() as i64)
                } else {
                    0
                };
                if tag.get_frame_type() == 1 && video_frame{// FRAME_TYPE_KEYFRAME
                    if tag.get_avc_packet_type() == 0 { // AVC_PACKET_TYPE_SEQUENCE_HEADER
                        println!("{}", format!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6}"     , i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), 0, 0, 0, dts_delta).on_red());
                        // println!("{:?}", tag.get_avcc());
                        i += 1;
                    } else { // AVC_PACKET_TYPE_NALU
                        println!("{}", format!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6} | {}", i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), tag.get_avc_composition_time_offset(), tag.get_timestamp(), (tag.get_timestamp() as i64) + (tag.get_avc_composition_time_offset() as i64), dts_delta, tag.get_nal_uints_info()).on_blue());
                        // println!("{:?}", tag.get_nal_units());
                        i += 1;
                    }
                } else if all_frame && video_frame {
                    println!("{}", format!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6} | {}", i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), tag.get_avc_composition_time_offset(), tag.get_timestamp(), (tag.get_timestamp() as i64) + (tag.get_avc_composition_time_offset() as i64), dts_delta, tag.get_nal_uints_info()).on_magenta());
                    // println!("{:?}", tag.get_nal_units());
                    i += 1;
                }
                last_v_tag = Some(tag);
            },
            FLVTagType::TAG_TYPE_AUDIO => {
                let dts_delta: i64 = if last_a_tag.is_some() {
                    (tag.get_timestamp() as i64) - (last_a_tag.unwrap().get_timestamp() as i64)
                } else {
                    0
                };
                if tag.is_acc_sequence_header() && audio_frame {
                    asc = Some(tag.get_sound_audio_specific_config());
                    println!("{}", format!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6} | [{:>5} {:>5} {}]", i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, "", tag.get_sound_format(), tag.get_sound_channels(), "", tag.get_timestamp(), "", dts_delta, tag.get_sound_frame_duration(asc.as_ref().unwrap()), asc.as_ref().unwrap().get_sample_rate(), tag.get_sound_size()).on_cyan());
                    // println!("{:?}", asc.as_ref().unwrap());
                    // println!("{:?}", FLVTag::get_sound_adts_header_data(asc.as_ref().unwrap(), 9 + 7));
                    i += 1;
                }
                else if all_frame && audio_frame {
                    println!("{}", format!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2} | {:>4} | {:>6} | {:>6} | {:>6} | [{:>5} {:>5} {}]", i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, "", tag.get_sound_format(), tag.get_sound_channels(), "", tag.get_timestamp(), "", dts_delta, tag.get_sound_frame_duration(asc.as_ref().unwrap()), asc.as_ref().unwrap().get_sample_rate(), tag.get_sound_size()).on_yellow());
                    // println!("{:?}", FLVTag::get_sound_adts_header_data(asc.as_ref().unwrap(), tag.get_sound_data_size() + 7));
                    // println!("{:?}", tag.get_sound_data());
                    i += 1;
                }
                last_a_tag = Some(tag);
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                println!("{:>6} | {:>10} | {:>10} | {:>6} | {:>4} | {:>2} | {:>2} | {:>2}", i, format_seconds_ms(tag.get_timestamp()), position, tag.get_tag_size(), tag.get_tag_type() as usize, "", "", "");
                if show_meta {
                    print_metatag(&Json::Array(tag.get_objects()));
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
    opts.optflag("v", "video", "print video frames");
    opts.optflag("d", "audio", "print audio frames");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("c", "crc32", "calculate crc32 of tags");

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
    let crc32_file = matches.opt_present("c");
    let all_frame = matches.opt_present("a");
    let video_frame = matches.opt_present("v");
    let audio_frame = matches.opt_present("d");
    if crc32_file {
        flv_crc32(&input);
    } else {
        flv_info(&input, show_meta, all_frame, video_frame, audio_frame);
    }
}

fn flv_crc32(path: &String) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::open(&path).unwrap();
    let mut parser = FLVTagRead::new(&mut file);

    let mut i = 0;
    let mut key_time: f64 = 0f64;
    let mut key_pos: u64 = 0;
    let mut tmp: Vec<u32> = Vec::new();
    let mut ret: Vec<Json> = Vec::new();
    loop {
        let position = parser.get_position();
        let tag = parser.next();
        if tag.is_none() {
            break;
        }

        let tag = tag.unwrap();
        let mut bytes: Vec<u8> = Vec::with_capacity(tag.get_tag_size() as usize);
        tag.write(&mut bytes);
        let mut hash = Crc32::new();
        hash.update(&bytes);
        let crc32_hash: u32 = hash.finish();
        if tag.get_tag_type() == FLVTagType::TAG_TYPE_VIDEO && tag.get_frame_type() == 1 {
            if tmp.len() > 0 {
                ret.push(output_info(key_pos, key_time, &mut tmp));
            }
            key_pos = position;
            key_time = tag.get_timestamp() as f64 / 1000f64;
        }
        tmp.push(crc32_hash);
    }
    if tmp.len() > 0 {
        ret.push(output_info(key_pos, key_time, &mut tmp));
    }

    // println!("{}", rustc_serialize::json::as_pretty_json(&Json::Array(ret)));
    let output_path = format!("{}{}", path, ".crc32.json");
    let mut output_file = File::create(output_path).unwrap();
    write!(output_file, "{}", rustc_serialize::json::as_pretty_json(&Json::Array(ret)));
}

fn output_info(key_pos: u64, key_time: f64, crc_list: &mut Vec<u32>) -> Json {
    use std::collections::BTreeMap;
    let mut arr: Vec<Json> = 
        vec![("time"  , Json::F64(key_time)),
             ("offset", Json::U64(key_pos)),
             ("tags"  , Json::Array(crc_list.iter().map(|i| Json::U64(*i as u64)).collect()))]
            .into_iter().map(|(key, json)| {
                let mut obj: BTreeMap<String, Json> = BTreeMap::new();
                obj.insert(key.to_string(), json);
                Json::Object(obj)
            }).collect();

    crc_list.clear();
    Json::Array(arr)
}