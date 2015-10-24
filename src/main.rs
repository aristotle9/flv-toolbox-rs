extern crate rustc_serialize;

use rustc_serialize::json::Json;

mod lib;
use lib::*;

fn flv_parse() {
    use std::fs::File;
    use std::path::Path;

    // return;
    // let path = Path::new("/Users/lanfan/projects/as3-projects/videos/av2998818-4737635/4737635-1.flv");
    let path = Path::new("/Users/lanfan/projects/as3-projects/videos/youku-1/0300010800561D2AD49F851468DEFEA585825F-9542-DC16-3713-AC06678EC8EB.flv");
    let file = File::open(path).unwrap();
    let file_meta = file.metadata().unwrap();
    let file_size = file_meta.len();
    println!("flv size: {}", file_size);
    let mut parser = FLVTagRead::new(file);//header has read

    //first tag
    let mut metatag = parser.next().unwrap();
    assert_eq!(metatag.get_tag_type(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);
    let v = metatag.get_objects();
    let _event_name = v[0].as_string().unwrap().to_string();
    let times = v[1].find_path(&["keyframes", "times"]).unwrap().as_array().unwrap();
    let times = times.iter().map(|val: &Json| {
        val.as_f64().unwrap()
    }).collect::<Vec<f64>>();
    let filepositions = v[1].find_path(&["keyframes", "filepositions"]).unwrap().as_array().unwrap();
    let filepositions = filepositions.iter().map(|val: &Json| {
        val.as_f64().unwrap() as u64
    }).collect::<Vec<u64>>();
    for (i, (t, p)) in (0u32..).zip(times.iter().zip(filepositions.iter())) {
        println!("{:3} {} {:8}", i, format_seconds_ms((t * 1000f64) as u64), p);
    }
    let vec = split_flv_6min(&times, &filepositions);
    println!("{:?}", vec.iter().map(|&(t, p, n)| (format_seconds_ms((t * 1000f64) as u64), p, n)).collect::<Vec<(String, u64, u64)>>());

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
                    println!("{:?}", (format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_frame_type(), tag.get_codec_id(), tag.get_avc_packet_type(), position));
                }
            },
            FLVTagType::TAG_TYPE_AUDIO => {
                if tag.is_acc_sequence_header() {
                    println!("{:?}", (format_seconds_ms(tag.get_timestamp()), tag.get_tag_type(), tag.get_data_size()));
                }
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                panic!("more than one metatag!");
            }
        };
        // println!("{:?}", (tag.get_tag_type(), tag.get_data_size(), tag.get_timestamp()));
    }
}

fn main() {
    flv_parse();
}
