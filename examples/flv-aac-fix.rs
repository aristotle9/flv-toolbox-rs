extern crate rustc_serialize;
extern crate getopts;
extern crate byteorder;
extern crate flv_toolbox_rs;

use std::path::Path;
use std::fs::File;
use std::io::{ Seek, SeekFrom, Read, Write };

use self::byteorder::{BigEndian, WriteBytesExt};
use rustc_serialize::json::Json;
use getopts::Options;

use flv_toolbox_rs::lib::{ FLVTagRead, FLVHeader, FLVTagType, FLVTag };

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    eprintln!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("o", "output", "output flv file", "OUTPUT");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("t", "check-only", "test, check only");

    let usage_str = {
        let brief = format!("Usage: {} FILE [options]", program);
        format!("{}", opts.usage(&brief))
    };

    let exit_with_usage = || {
        eprintln!("{}", usage_str);
        std::process::exit(-1);
    };

    let matches: getopts::Matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            eprintln!("{}", f.to_string());
            return exit_with_usage();
        }
    };

    if matches.opt_present("h") {
        return exit_with_usage();
    }

    let test = matches.opt_present("t");

    let input: String = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        eprintln!("no input file.");
        return exit_with_usage();
    };

    let input_path: &Path = Path::new(&input);
    if !input_path.exists() {
        eprintln!("input file does not exist.");
        return exit_with_usage();
    }

    let output = match matches.opt_default("o", "") {
        Some(c) => c,
        _ => {
            let file = input_path.file_stem().unwrap().to_string_lossy().to_string();
            let mut output = input_path.with_file_name(format!("{}-fixed.flv", &file));
            let mut i: i32 = 0;
            while output.exists() {
                i += 1;
                output = input_path.with_file_name(format!("{}-fixed({}).flv", &file, i));
            }
            if !test {
                eprintln!("no output file, use {}", output.to_str().unwrap());
            }
            output.to_string_lossy().to_string()
        }
    };

    let need_fix = detect_flv_acc(input_path);
    if need_fix {
        match fix_flv_acc(input_path, &output, test) {
            Ok(_) => {
                if test {
                    eprintln!("test complete.");
                } else {
                    eprintln!("fixed.");
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("fix err: {}", e);
                std::process::exit(-1);
            }
        }
    } else {
        eprintln!("pass");
    }
}

fn next_tag_of_type<'a, R: Read>(parser: &mut FLVTagRead<'a, R>, tag_type: FLVTagType) -> Option<FLVTag> {
    loop {
        if let Some(tag) = parser.next() {
            if tag.get_tag_type() == tag_type {
                break Some(tag);
            } else {
                continue;
            }
        } else {
            break None;
        }
    }
}

fn detect_flv_acc(flv_path: &Path) -> bool {
    let mut file = std::fs::File::open(flv_path).unwrap();
    let mut parser = FLVTagRead::new(&mut file);
    {
        let header: &FLVHeader = &parser.header;
        if !header.hasAudioTags {
            eprintln!("no audio stream");
            return false;
        }
    }

    let audio_tag = next_tag_of_type(&mut parser, FLVTagType::TAG_TYPE_AUDIO);
    if audio_tag.is_none() {
        return false;
    }
    let audio_tag: FLVTag = audio_tag.unwrap();
    if !audio_tag.is_acc_sequence_header() {
        eprintln!("first audio tag is not acc_sequence_header, exit");
        return false;
    }
    let data_size = audio_tag.get_data_size();
    // println!("{:?}", audio_tag.get_sound_audio_specific_config());
    return data_size == 2;
}

fn fix_flv_acc(flv_path: &Path, output_path: &str, test: bool) -> Result<(), String> {
    let new_tag = {
        let mut file = std::fs::File::open(flv_path).map_err(|_| "cannot open output file.".to_owned())?;
        let mut parser = FLVTagRead::new(&mut file);
        
        let meta_tag = next_tag_of_type(&mut parser, FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT).ok_or("no meta tag".to_string())?;
        let mut acc_tag = next_tag_of_type(&mut parser, FLVTagType::TAG_TYPE_AUDIO).ok_or::<String>("no acc_sequence_header".into())?;
        assert!(acc_tag.is_acc_sequence_header());
        assert_eq!(acc_tag.get_data_size(), 2);
        let a_tag = next_tag_of_type(&mut parser, FLVTagType::TAG_TYPE_AUDIO).ok_or::<String>("only one acc_sequence_header".into())?;
        let meta_obj = &meta_tag.get_objects()[1];
        println!("{:?}", meta_obj);
        let sample: i64 = meta_obj.find("audiosamplerate").ok_or("no audiosamplerate in meta, can't fix.".to_owned())?.as_f64().ok_or("audiosamplerate is not f64, can't fix.".to_string())? as _;
        let stereo = meta_obj.find("stereo").ok_or("no stereo in meta, can't fix.".to_owned())?.as_boolean().ok_or("no stereo in meta or stereo is not boolean, can't fix.".to_owned())?;
        if meta_obj.find("keyframes").is_some() {
            eprintln!("warning: flv has keyframes table. filepositions should adjust, but not.");
        }
        let original_audio_object_type = 2;
        let sample_index = [96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350].iter().position(|i: &i64| *i == sample).ok_or("sample not in sample list.".to_owned())?;
        let channel_config = if stereo { 2 } else { 1 };

        eprintln!("use config: original_audio_object_type {} sample_index {} channel_config {}", original_audio_object_type, sample_index, channel_config);
        let mut data: Vec<u8> = Vec::with_capacity(11 + 4 + 4);
        acc_tag.write(&mut data);
        eprintln!("{:?}", data);
        // set data size
        data[1] = ((4 >> 16) & 0xff) as u8;
        data[2] = ((4 >>  8) & 0xff) as u8;
        data[3] = ((4      ) & 0xff) as u8;
        data[13] = ((original_audio_object_type as u8 & 0x1f) << 3) | ((sample_index as u8 & 0xf) >> 1);
        data[14] = ((sample_index as u8 & 1) << 7) | ((channel_config & 0xf) << 3);
        data.pop();
        data.pop();
        data.write_u32::<BigEndian>(15).unwrap();
        eprintln!("{:?}", data);
        let new_tag = FLVTag::read(&mut &*data).unwrap();
        eprintln!("{:?}", (new_tag.get_tag_type(), new_tag.get_data_size(), new_tag.get_sound_audio_specific_config()));
        new_tag
    };
    if test {
        return Ok(());
    }
    let mut new_tag = Some(new_tag);
    // reopen
    let mut file = std::fs::File::open(flv_path).map_err(|_| "cannot open output file.".to_owned())?;
    let mut parser = FLVTagRead::new(&mut file);
    let mut ofile = std::fs::File::create(output_path).map_err(|_| "cannot open output file.".to_owned())?;
    parser.header.write(&mut ofile);
    while let Some(mut tag) = parser.next() {
        if new_tag.is_some() && tag.get_tag_type() == FLVTagType::TAG_TYPE_AUDIO && tag.is_acc_sequence_header() {
            tag = new_tag.take().unwrap();// only switch once
        }
        tag.write(&mut ofile);
    }
    Ok(())
}