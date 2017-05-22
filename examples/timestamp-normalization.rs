extern crate ffmpeg;
extern crate rustc_serialize;
extern crate getopts;
extern crate flv_toolbox_rs;

use flv_toolbox_rs::lib::{ FLVTagRead, FLVHeader, FLVTag, FLVTagType, FLVTagWrite, format_seconds_ms, AudioSpecificConfig };

use rustc_serialize::json::{Json};
use getopts::Options;

use std::path::Path;
use std::fs::File;
use std::io::{ Read, Write, Cursor, Seek, SeekFrom };
use std::collections::BTreeMap;

const PROGRAM_SIGN: &'static str = "audio gap fixed by timestamp-normalization, 2017";

type FLVInfo = Vec<TagProfile>;

type Id = u64;
const MAX_ID: u64 = std::u64::MAX;

#[derive(Debug, Clone)]
pub struct TagProfile {
    pub id: u64,
    pub tag_type: FLVTagType,
    pub timestamp: i64,
    pub position: u64,
    pub sequence_header: bool,
    pub keyframe: bool,
    pub decode_duration: i64,
    pub duration: i64,
    pub offset: i64,
    pub deleted: bool,
}

impl TagProfile {
    pub fn new_video(id: u64, timestamp: i64, position: u64, sequence_header: bool, keyframe: bool) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_VIDEO,
            timestamp,
            position,
            sequence_header,
            keyframe,
            decode_duration: 0,
            duration: 0,
            offset: 0,
            deleted: false,
        }
    }

    pub fn new_audio(id: u64, timestamp: i64, position: u64, sequence_header: bool, decode_duration: i64, duration: i64) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_AUDIO,
            timestamp,
            position,
            sequence_header,
            keyframe: sequence_header,
            decode_duration,
            duration,
            offset: 0,
            deleted: false,
        }
    }

    pub fn new_meta(id: u64, timestamp: i64, position: u64) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT,
            timestamp,
            position,
            sequence_header: false,
            keyframe: false,
            decode_duration: 0,
            duration: 0,
            offset: 0,
            deleted: false,
        }
    }

    pub fn tag(&self, file: &mut File) -> FLVTag {
        if self.id == MAX_ID { // generate mut audio tag
            TagProfile::new_mute_tag(self.timestamp)
        } else {
            file.seek(SeekFrom::Start(self.position));
            FLVTag::read(file).unwrap()
        }
    }

    pub fn new_mute(timestamp: i64) -> TagProfile {
        TagProfile::new_audio(MAX_ID, timestamp, 0, false, 23, 23)
    }

    pub fn new_mute_tag(timestamp: i64) -> FLVTag {
        let sound_rate = 44100_f64;
        let channels: u8 = 2;
        let sound_size: u8 = 16; // 16 | 8

        let data_list = ([0x01, 0x18, 0x20, 0x07], [0x21, 0x10, 0x04, 0x60, 0x8c, 0x1c]);
        let data: &[u8] = if channels == 2 { &data_list.1 } else { &data_list.0 };
        let data_size: usize = data.len() + 2;
        let channels_index   = channels - 1; 
        let sound_size_index = sound_size / 8 - 1;
        let sound_rate_index = [5512.5_f64, 11025_f64, 22050_f64, 44100_f64].iter().position(|&x| x == sound_rate).unwrap() as u8;
        let mut tag_data: Vec<u8> = Vec::with_capacity(data_size + 11 + 4);
        
        // tag type
        tag_data.push(8);
        // data size,
        tag_data.push(((data_size >> 16) & 0xff) as u8);
        tag_data.push(((data_size >>  8) & 0xff) as u8);
        tag_data.push(((data_size      ) & 0xff) as u8);
        // timestamp
    	tag_data.push(((timestamp >> 16) & 0xff) as u8);
    	tag_data.push(((timestamp >> 8 ) & 0xff) as u8);
    	tag_data.push(((timestamp      ) & 0xff) as u8);
        tag_data.push(((timestamp >> 24) & 0xff) as u8); // extended byte in unusual location
        // stream id
        tag_data.push(0);
        tag_data.push(0);
        tag_data.push(0);
        // audio header
        tag_data.push((10 << 4) | (sound_rate_index << 2) | (sound_size_index << 1) | channels_index);
        tag_data.push(1);
        // audio data
        tag_data.extend_from_slice(data);
        // prev size
        tag_data.push(0);
        tag_data.push(0);
        tag_data.push(0);
        tag_data.push(0);

        let mut tag = FLVTag::read(&mut &*tag_data).unwrap();
        tag
    }
}

fn get_info(path: &str) -> FLVInfo {
    
    let mut file = File::open(path).unwrap();
    let file_info = file.metadata().unwrap();
    let file_len = file_info.len();
    let mut parser = FLVTagRead::new(&mut file);

    let mut id: u64 = 0;

    let mut info: FLVInfo = Vec::new();

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
        print!("scan progress: {: >3.0}%\r", position as f64 / file_len as f64 * 100.);
        let nxt: Option<FLVTag> = parser.next();
        if nxt.is_none() {
            break;
        }
        let tag: FLVTag = nxt.unwrap();
        
        match tag.get_tag_type() {
            FLVTagType::TAG_TYPE_VIDEO => {
                info.push(TagProfile::new_video(id, tag.get_timestamp() as i64, position, tag.get_frame_type() == 1 && tag.get_avc_packet_type() == 0, tag.get_frame_type() == 1));
            },
            FLVTagType::TAG_TYPE_AUDIO => {
                if tag.is_acc_sequence_header() { // acc sequence header
                    asc = Some(tag.get_sound_audio_specific_config());
                    info.push(TagProfile::new_audio(id, tag.get_timestamp() as i64, position, true, 0, 0));
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
                    let mut duration: i64 = 0;
                    if result {
                        duration = (1000. * frame.samples() as f64 / frame.rate() as f64) as i64;
                        // println!("{:?}", (frame.channels(), frame.rate(), frame.samples(), duration));
                    }
                    info.push(TagProfile::new_audio(id, tag.get_timestamp() as i64, position, false, duration, 0));
                }
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                info.push(TagProfile::new_meta(id, tag.get_timestamp() as i64, position));
            }
        };
        id += 1;
    }
    println!("scan complete!\r");
    return info;
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

fn check_offset(info: &mut FLVInfo) -> bool {

    let mut last_audio_id: Id = 0;
    {// 计算实际的 frame duration
        let mut last_audio_profile: Option<&mut TagProfile> = None;
        for item in info.iter_mut() {
            let mut audio = false;
            {
                let &mut TagProfile {
                    ref id,
                    ref tag_type,
                    ref timestamp,
                    ..
                } = item;
                match *tag_type {
                    FLVTagType::TAG_TYPE_AUDIO => {
                        if let Some(&mut TagProfile { timestamp: ref last_timestamp, ref mut duration, .. }) = last_audio_profile {
                            *duration = *timestamp - *last_timestamp;
                        }
                        audio = true;
                        last_audio_id = *id;
                    }
                    _ => {
                    }
                }
            }
            if audio {
                last_audio_profile = Some(item);
            }
        }
    }
    // println!("{:?}", info.iter_mut().filter(|item| {
    //     match item {
    //         &&mut TagProfile::Audio(_, _, _, _, _) => {
    //             return true;
    //         }
    //         _ => {
    //             return false;
    //         }
    //     }
    // }).take(10).collect::<Vec<&mut TagProfile>>());

    let mut audio_time: i64 = 0;
    let mut has_gap: bool = false;

    for item in info.iter() {
        let &TagProfile {
            ref id,
            ref tag_type,
            ref timestamp,
            sequence_header: ref sh,
            ref decode_duration,
            ref duration,
            ..
        } = item;
        match *tag_type {
            FLVTagType::TAG_TYPE_AUDIO => {
                audio_time += *decode_duration;
                if (*duration - *decode_duration).abs() > 1 && *id != last_audio_id && !*sh {// 排除最后一个Tag(容器持续时间不能确定)，SequenceHeader
                    // 打印经过这个Tag后会发生的偏移
                    println!("{:>8} {} {:>8}", *id, format_seconds_ms(*timestamp as u64), (audio_time + decode_duration) - (*timestamp + *duration));
                    has_gap = true;
                }
            }
            _ => {}
        };
    }
    return has_gap;
}

fn get_fix_info(info: FLVInfo) -> FLVInfo {
    
    // 计算重新排列后的 Tag 布局
    // 不能扔的Tag, 比如 metadata avc sequence header
    let mut c_tags: Vec<TagProfile> = vec![];
    let mut v_tags: Vec<TagProfile> = vec![];
    let mut a_tags: Vec<TagProfile> = vec![];

    for item in info.iter() {
        let &TagProfile {
            ref tag_type,
            sequence_header: ref sh,
            ..
        } = item;
        match *tag_type {
            FLVTagType::TAG_TYPE_AUDIO => {
                if *sh {
                    c_tags.push(item.clone());
                } else {
                    a_tags.push(item.clone());
                }
            }
            FLVTagType::TAG_TYPE_VIDEO => {
                if *sh {
                    c_tags.push(item.clone());
                } else {
                    v_tags.push(item.clone());
                }
            }
            _ => {
                c_tags.push(item.clone());
            }
        }
    }

    let mut j: usize = 0;

    let mut timeline_offset: i64 = 0;
    let mut timeline_timestamp: i64 = 0;
    let mut timeline_decode_duration: i64 = 0;
//
//   tm + offset     delta
//   |           |<--- gap --->|
//   +-----------+             +-----------+
//   |dc_duration|             |           |
//   +-----------+             +-----------+
//   |<-------duration-------->|
//   |<-- timeline
//
    for i in 0..a_tags.len() {
        let TagProfile {
            timestamp: ref tm,
            ref decode_duration,
            ref mut offset,
            ..
        } = a_tags[i];

        let delta: i64 = (timeline_timestamp + timeline_offset + timeline_decode_duration) - (tm + timeline_offset);
        if delta.abs() > 1 {
            let gap_left = timeline_timestamp + timeline_offset + timeline_decode_duration;
            let gap_right = *tm + timeline_offset;
            loop {
                let TagProfile {
                    timestamp: ref tm,
                    ref mut offset,
                    ref mut deleted,
                    ..
                } = v_tags[j];
                if *tm + timeline_offset <= gap_left {
                    *offset = timeline_offset;
                    j += 1;
                    continue;
                } else {
                    if *tm + timeline_offset < gap_right {
                        *deleted = true;
                        j += 1;
                        continue;
                    } else {
                        break;
                    }
                }
            }
            println!("gap {:>6} {:>6} {:>6}", gap_left, gap_right, -delta);
            timeline_offset += delta;
        }
        timeline_decode_duration = *decode_duration;
        timeline_timestamp = *tm;
        *offset = timeline_offset;
    }

    // for (ref k, ref v) in delete_flags.iter() {
    //     println!("del vtag {:>6}", k);
    // }
    
    println!("before delete by gop, {} tags would be deleted", v_tags.iter().filter(|t| t.deleted).count());
    // del a gop when on frame has been deleted

    let mut for_delete_indices: Vec<usize> = vec![];
    let mut gop: Vec<usize> = vec![];
    let mut gop_delete: bool = false;
    for i in 0..v_tags.len() {
        let TagProfile {
            ref keyframe,
            ref deleted,
            ..
        } = v_tags[i];
        if *keyframe {
            if gop_delete {
                for_delete_indices.append(&mut gop);
            } else {
                gop.clear(); // clear
            }
            gop_delete = false;
        }
        gop.push(i);
        if *deleted {
            gop_delete = true;
        }
    }
    if gop_delete {
        for_delete_indices.append(&mut gop);
    }

    println!("after delete by gop, {} tags would be deleted", for_delete_indices.len());

    for i in for_delete_indices.into_iter() {
        v_tags[i].deleted = true;
    }

    // mux profiles
    let mut new_profiles: Vec<TagProfile> = vec![];
    new_profiles.append(&mut a_tags);
    new_profiles.append(&mut v_tags.into_iter().filter(|t| !t.deleted).collect::<Vec<TagProfile>>());
    new_profiles.sort_by_key(|t| t.timestamp + t.offset);
    c_tags.append(&mut new_profiles);
    return c_tags;
}

// 使用插入空白 audio frame 方法补齐比较大的 gap
fn get_fix_info2(info: FLVInfo, mute_tag: TagProfile) -> FLVInfo {
    
    // 计算重新排列后的 Tag 布局
    // 不能扔的Tag, 比如 metadata avc sequence header
    let mut c_tags: Vec<TagProfile> = vec![];
    let mut v_tags: Vec<TagProfile> = vec![];
    let mut a_tags: Vec<TagProfile> = vec![];

    for item in info.iter() {
        let &TagProfile {
            ref tag_type,
            sequence_header: ref sh,
            ..
        } = item;
        match *tag_type {
            FLVTagType::TAG_TYPE_AUDIO => {
                if *sh {
                    c_tags.push(item.clone());
                } else {
                    a_tags.push(item.clone());
                }
            }
            FLVTagType::TAG_TYPE_VIDEO => {
                if *sh {
                    c_tags.push(item.clone());
                } else {
                    v_tags.push(item.clone());
                }
            }
            _ => {
                c_tags.push(item.clone());
            }
        }
    }

    let mut j: usize = 0;
    let mut b_tags: Vec<TagProfile> = vec![];
    let TagProfile { decode_duration: ref mute_tag_dd, .. } = mute_tag;

    let mut timeline_offset: i64 = 0;
    let mut timeline_timestamp: i64 = 0;
    let mut timeline_decode_duration: i64 = 0;
//
//   tm + offset     delta
//   |           |<--- gap --->|
//   +-----------+             +-----------+
//   |dc_duration|             |           |
//   +-----------+             +-----------+
//   |<-------duration-------->|
//   |<-- timeline
//
    for i in 0..a_tags.len() {
        let TagProfile {
            timestamp: ref tm,
            ref decode_duration,
            ref mut offset,
            ..
        } = a_tags[i];

        let delta: i64 = (timeline_timestamp + timeline_offset + timeline_decode_duration) - (tm + timeline_offset);
        if delta.abs() > 1 {
            let mut gap_left = timeline_timestamp + timeline_offset + timeline_decode_duration;
            let gap_right = *tm + timeline_offset;
            while gap_right - gap_left >= *mute_tag_dd {
                b_tags.push(TagProfile::new_mute(gap_left));
                gap_left += *mute_tag_dd;
            }
            if gap_right - gap_left > 1 {
                // make some offset
                println!("remain offset {} {:>3}", format_seconds_ms(*tm as u64), gap_right - gap_left);
            }
        }
        timeline_decode_duration = *decode_duration;
        timeline_timestamp = *tm;
        *offset = timeline_offset;
    }

    println!("b_tags len {}", b_tags.len());
    // mux profiles
    let mut new_profiles: Vec<TagProfile> = vec![];
    new_profiles.append(&mut a_tags);
    new_profiles.append(&mut b_tags);
    new_profiles.append(&mut v_tags);
    new_profiles.sort_by_key(|t| t.timestamp + t.offset);
    c_tags.append(&mut new_profiles);
    return c_tags;
}

fn fix_file(input: &str, output: &str, info: FLVInfo) {

    use std::io::SeekFrom::{Current, Start};

    let mut file = File::open(input).unwrap();
    let file_info = file.metadata().unwrap();
    let file_len = file_info.len();
    
    let mut output_file: File = File::create(output).unwrap();
    let mut tag_write: FLVTagWrite<File> = FLVTagWrite::new(output_file);

    let header = FLVHeader::read(&mut file);
    tag_write.write_header(&header);

    // function from flv-split
    fn write_back_meta_tag<T: Write + Seek>(duration: u64, metatag: &mut FLVTag, times: &Vec<u64>, filepositions: &Vec<u64>, tag_write: &mut FLVTagWrite<T>) {
        let mut metas = metatag.get_objects();
        {
            metas[1].as_object_mut().unwrap().insert("duration".to_string(), Json::F64(duration as f64 / 1000.0));
            metas[1].as_object_mut().unwrap().insert("gapfixedby".to_string(), Json::String(PROGRAM_SIGN.to_string()));
            let key_times = Json::Array(times.iter().map(|&t| Json::F64(t as f64 / 1000.0)).collect::<Vec<Json>>());
            let key_positions = Json::Array(filepositions.iter().map(|&p| Json::F64(p as f64)).collect::<Vec<Json>>());
            let keyframes = metas[1].as_object_mut().unwrap().get_mut("keyframes").unwrap().as_object_mut().unwrap();
            keyframes.insert("times".to_string(), key_times);
            keyframes.insert("filepositions".to_string(), key_positions);
        }
        metatag.set_objects(&metas);
        tag_write.write_meta_tag(&metatag);
    }

    // create metatag
    let times = info.iter()
        .filter(|&&TagProfile { ref tag_type, ref keyframe, .. }| *tag_type == FLVTagType::TAG_TYPE_VIDEO && *keyframe )
        .map(|&TagProfile { timestamp: ref t, .. }| *t as u64).collect::<Vec<u64>>();
    let mut positions: Vec<u64> = vec![0u64; times.len()];
    let mut metatag = info.iter().find(|&&TagProfile { ref tag_type, .. }| *tag_type == FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT).unwrap().tag(&mut file);
    let duration = {
        let item = info.iter().filter(|&&TagProfile { ref tag_type, .. }| *tag_type == FLVTagType::TAG_TYPE_AUDIO).last().unwrap();
        (item.timestamp + item.decode_duration) as u64
    };

    // write metatag
    write_back_meta_tag(duration, &mut metatag, &times, &positions, &mut tag_write);

    let mut frame_index: usize = 0;

    for item in info.iter() {
        let &TagProfile {
            ref tag_type,
            ref keyframe,
            ref timestamp,
            ref offset,
            ..
        } = item;
        match *tag_type { // skip
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                continue;
            }
            FLVTagType::TAG_TYPE_VIDEO => {
                if *keyframe {
                    let p = tag_write.get_position();
                    positions[frame_index] = p;
                    frame_index += 1;
                }
            }
            _ => {}
        }
        let mut tag = item.tag(&mut file);
        tag.set_timestamp((*timestamp + *offset) as u64);
        tag_write.write_tag(&tag);
    }

    // write metatag
    write_back_meta_tag(duration, &mut metatag, &times, &positions, &mut tag_write);
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("o", "output", "output flv file", "OUTPUT");
    opts.optflag("d", "drop-video", "fix audio gap by drop video frames");
    opts.optflag("b", "fill-mute-audio", "fix audio gap by fill mute audio frames");
    opts.optflag("f", "offset", "fill mute audio, also offset video frame to avoid gap");
    opts.optflag("v", "verbose", "show more information");
    opts.optflag("h", "help", "print this help menu");

    let matches: getopts::Matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let input: String = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        println!("no input file.");
        print_usage(&program, opts);
        return;
    };

    let input_path: &Path = Path::new(&input);
    if !input_path.exists() {
        println!("input file does not exist.");
        print_usage(&program, opts);
        return;
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
            println!("no output file, use {}", output.to_str().unwrap());
            output.to_string_lossy().to_string()
        }
    };

    let verbose     = matches.opt_present("v");
    let drop_mode   = matches.opt_present("d");
    let fill_mode   = matches.opt_present("b");
    let offset_mode = matches.opt_present("f");

    println!("checking flv file: {}", input);
    let mut info = get_info(&input);
    let has_gap = check_offset(&mut info);

    let fix: bool = drop_mode || fill_mode;
    if has_gap {
        if fix {
            let new_info = if drop_mode {
                get_fix_info(info)
            } else {
                // println!("{:?}", (TagProfile::new_mute_tag(0)));
                get_fix_info2(info, TagProfile::new_mute(0))
            };
            fix_file(&input, &output, new_info);
            println!("flv fix complete.\nplease use `ffmpeg -i \"{}\" -acodec copy -vcodec copy \"{}\"` to get mp4 file.", &output, Path::new(&output).with_extension("mp4").to_str().unwrap());
        } else {
            println!("there are audio gaps, please set the fix mode (-b or -d) to fix them.");
        }
    } else {
        println!("no gap.");
    }
}