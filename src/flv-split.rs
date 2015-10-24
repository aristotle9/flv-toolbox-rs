#![feature(path_ext)]
extern crate rustc_serialize;

use rustc_serialize::json::Json;

mod lib;
use lib::*;

fn flv_split(path: &String, min: u64) {
    use std::fs::File;
    use std::fs::PathExt;
    use std::path::Path;

    let path = Path::new(path);
    if !path.exists() {
        panic!(format!("file dosen't exist: {}", path.display()));
    } else {
        println!("start to split flv {}, each file has {}min(s)", path.display(), min);
    }

    let file = File::open(path).unwrap();
    let file_meta = file.metadata().unwrap();
    let file_size = file_meta.len();
    println!("file size: {}", file_size);
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
    let vec = split_flv_by_min(&times, &filepositions, min);
    println!("{:?}", vec.iter().map(|&(t, p, n)| (format_seconds_ms((t * 1000f64) as u64), p, n)).collect::<Vec<(String, u64, u64)>>());

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
    let mut tag_index: u64 = 0;
    let mut times: Option<Vec<u64>> = None;
    let mut filepositions: Option<Vec<u64>> = None;

    fn write_back_meta_tag(metatag: &mut FLVTag, times: &Vec<u64>, filepositions: &Vec<u64>, tag_write: &mut FLVTagWrite<File>) {
        let mut metas = metatag.get_objects();
        {
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
                write_back_meta_tag(&mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write.as_mut().unwrap());
            }
            seg_index += 1;
            let file_name = format!("seg-{}.flv", seg_index + 1);
            tag_write = Some(FLVTagWrite::new(File::create(file_name).unwrap()));
            let tag_write = tag_write.as_mut().unwrap();
            let key_tag_len = (vec[seg_index as usize].2 + 1) as usize;
            times = Some(vec![0; key_tag_len]);
            filepositions = Some(vec![0; key_tag_len]);
            tag_write.write_header(&parser.header);
            //modify metatag
            write_back_meta_tag(&mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write);
            filepositions.as_mut().unwrap()[0] = tag_write.get_position();
            tag_write.write_tag(&video_metatag);
            tag_write.write_tag(&audio_metatag);
            tag_index = 1;
            time_offset = tag.as_ref().unwrap().get_timestamp();
        }

        if tag.is_none() {
            write_back_meta_tag(&mut metatag, times.as_ref().unwrap(), filepositions.as_ref().unwrap(), tag_write.as_mut().unwrap());
            break;
        }
        let mut tag = tag.unwrap();
        //change tag timestamp
        let timestamp = tag.get_timestamp() - time_offset;
        tag.set_timestamp(timestamp);
        if tag.get_tag_type() == FLVTagType::TAG_TYPE_VIDEO && tag.get_frame_type() == 1 {
            // println!("write keyframe tag {}/{}", tag_index, filepositions.as_ref().unwrap().len() - 1);
            filepositions.as_mut().unwrap()[tag_index as usize] = tag_write.as_ref().unwrap().get_position();
            times.as_mut().unwrap()[tag_index as usize] = timestamp;
            tag_index += 1;
        }
        tag_write.as_mut().unwrap().write_tag(&tag);
    }
}

fn main() {
    // let path = "/Users/lanfan/projects/as3-projects/videos/av2998818-4737635/4737635-1.flv";
    let path = "/Users/lanfan/projects/as3-projects/videos/youku-1/0300010800561D2AD49F851468DEFEA585825F-9542-DC16-3713-AC06678EC8EB.flv".to_string();
    let min = 1;
    flv_split(&path, min);
}
