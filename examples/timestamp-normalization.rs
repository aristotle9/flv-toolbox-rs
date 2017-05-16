extern crate ffmpeg;
extern crate flv_toolbox_rs;

use flv_toolbox_rs::lib::{ FLVTagRead, FLVTag, FLVTagType, FLVTagWrite, format_seconds_ms, AudioSpecificConfig };

use std::fs::File;
use std::io::{ Read, Write };
use std::slice;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct FlvTimeInfo {
    pub ref_video_frame_duration: u64,
    pub ref_audio_frame_duration: u64,
    pub real_video_frame_duration: i64,
    pub real_audio_frame_duration: i64,
    pub timestamps: Vec<TagTimestamp>,
}

#[derive(Debug, Clone)]
pub enum TagTimestamp {
    Video(i64, i64),
    Audio(i64, i64, i64),
}

fn get_info(path: &str) -> FlvTimeInfo {
    
    let mut file = File::open(path).unwrap();
    let file_info = file.metadata().unwrap();
    let file_len = file_info.len();
    let mut parser = FLVTagRead::new(&mut file);

    let mut ref_video_frame_duration: u64 = 0;
    let mut ref_audio_frame_duration: u64 = 0;
    let mut real_video_frame_duration: i64 = 0;
    let mut real_audio_frame_duration: i64 = 0;

    let mut last_video_tag: Option<FLVTag> = None;
    let mut video_duration_map: BTreeMap<i64, u64> = BTreeMap::new();

    let mut last_audio_tag: Option<FLVTag> = None;
    let mut audio_duration_map: BTreeMap<i64, u64> = BTreeMap::new();
    let mut last_audio_decode_duration: i64 = 0;

    let mut timestamps: Vec<TagTimestamp> = vec![];

    // ffmpeg
    ffmpeg::init().unwrap();
    let codec = ffmpeg::decoder::find(ffmpeg::codec::id::Id::AAC).unwrap();
    let context = ffmpeg::codec::Context::new();
    let opened = context.decoder().open_as(codec).unwrap();
    let mut decoder = opened.audio().unwrap();
    // asc
    let mut asc: Option<AudioSpecificConfig> = None;

    loop {
        let position = parser.get_position();
        print!("progress: {: >3.0}%\r", position as f64 / file_len as f64 * 100.);
        let nxt: Option<FLVTag> = parser.next();
        if nxt.is_none() {
            break;
        }
        let tag: FLVTag = nxt.unwrap();
        
        match tag.get_tag_type() {
            FLVTagType::TAG_TYPE_VIDEO => {

                if last_video_tag.is_some() {
                    let time_delta = tag.get_timestamp() as i64 - last_video_tag.as_ref().unwrap().get_timestamp() as i64;
                    timestamps.push(TagTimestamp::Video(last_video_tag.as_ref().unwrap().get_timestamp() as i64, time_delta));
                    *(video_duration_map.entry(time_delta).or_insert(0)) += 1;
                }

                match tag.get_frame_type() {
                    1 => {
                        match tag.get_avc_packet_type() {
                            0 => { // AVC_PACKET_TYPE_SEQUENCE_HEADER

                            },
                            _ => { // normal keyframes
                                last_video_tag = Some(tag);
                            }
                        };
                    }
                    _ => { // normal frames
                        last_video_tag = Some(tag);
                    }
                };
            },
            FLVTagType::TAG_TYPE_AUDIO => {

                if last_audio_tag.is_some() {
                    let time_delta = tag.get_timestamp() as i64 - last_audio_tag.as_ref().unwrap().get_timestamp() as i64;
                    timestamps.push(TagTimestamp::Audio(last_audio_tag.as_ref().unwrap().get_timestamp() as i64, time_delta, last_audio_decode_duration));
                    *(audio_duration_map.entry(time_delta).or_insert(0)) += 1;
                }

                if tag.is_acc_sequence_header() { // acc sequence header
                    asc = Some(tag.get_sound_audio_specific_config());
                } else { // normal frames
                    // decode audio frame samples by ffmpeg
                    let mut audio_buffer: Vec<u8> = Vec::with_capacity(tag.get_sound_data_size() as usize + 7);
                    audio_buffer.write(&FLVTag::get_sound_adts_header_data(asc.as_ref().unwrap(), tag.get_sound_data_size() + 7));
                    audio_buffer.write(&tag.get_sound_data());
                    // println!("{:?}", audio_buffer);
                    // break;

                    let packet = ffmpeg::codec::packet::Packet::borrow(&audio_buffer);
                    let mut frame = ffmpeg::util::frame::Audio::empty();
                    let result = decoder.decode(&packet, &mut frame).unwrap();
                    let duration = 1000. * frame.samples() as f64 / frame.rate() as f64;
                    last_audio_decode_duration = duration as i64;
                    // println!("{:?}", (frame.channels(), frame.rate(), frame.samples(), duration));
                    
                    last_audio_tag = Some(tag);
                }
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {

            }
        };
    }
    println!("complete!\r");

    real_video_frame_duration = top_duration(&video_duration_map);
    real_audio_frame_duration = top_duration(&audio_duration_map);

    return FlvTimeInfo {
        ref_video_frame_duration  ,
        ref_audio_frame_duration  ,
        real_video_frame_duration ,
        real_audio_frame_duration ,
        timestamps                ,
    };
}

fn top_duration(pairs: &BTreeMap<i64, u64>) -> i64 {
    let mut list: Vec<(&i64, &u64)> = pairs.iter().collect();
    match list.len() {
        0 => 0,
        1 => *(list[0].0),
        _ => {
            list.as_mut_slice().sort_by_key(|&(_, &count)| count );
            *(list.last().unwrap().0)
        },
    }
}

fn check_offset(info: &FlvTimeInfo) {
    let &FlvTimeInfo {
        ref ref_video_frame_duration  ,
        ref ref_audio_frame_duration  ,
        ref real_video_frame_duration ,
        ref real_audio_frame_duration ,
        ref timestamps                ,
    } = info;

    let mut video_time: i64 = 0;
    let mut audio_time: i64 = 0;
    let mut current_time = 0;

    for tm in timestamps.iter() {
        let mut changed = false;
        match tm {
            &TagTimestamp::Video(ref time, ref duration) => {
                // current_time = *time;
                if (*duration - *real_video_frame_duration).abs() > 1 {
                    // println!("v offset {}", video_time - *time);
                    // changed = true;
                    video_time += *real_video_frame_duration;
                } else {
                    video_time += *duration;
                }
            }
            &TagTimestamp::Audio(ref time, ref duration, ref decode_duration) => {
                current_time = *time;
                if (*duration - *decode_duration).abs() > 1 {
                    // println!("a offset {}", audio_time - *time);
                    changed = true;
                    audio_time += *decode_duration;
                } else {
                    audio_time += *decode_duration;
                }
            }
        };
        if changed {
            println!("{}: av offset: {} {} {}", format_seconds_ms(current_time as u64), audio_time - video_time, audio_time - current_time, video_time - current_time);
        }
    }
}

fn main() {
    let path = std::env::args().nth(1).unwrap();
    println!("checking flv: {}", path);
    let info = get_info(&path);
    // println!("{:?}", info);
    check_offset(&info);
}