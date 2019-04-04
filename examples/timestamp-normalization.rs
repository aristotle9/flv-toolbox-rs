extern crate rustc_serialize;
extern crate getopts;
extern crate flv_toolbox_rs;
extern crate libc;

use flv_toolbox_rs::lib::{ FLVTagRead, FLVHeader, FLVTag, FLVTagType, FLVTagWrite, format_seconds_ms, AudioSpecificConfig };

use rustc_serialize::json::{Json};
use rustc_serialize::{ Encodable, Encoder };
use getopts::Options;

use std::str::FromStr;
use std::path::Path;
use std::fs::File;
use std::io::{ Write, Seek, SeekFrom };
use std::collections::BTreeMap;

const PROGRAM_SIGN: &'static str = "audio gap fixed by timestamp-normalization, 2017";

type FLVInfo = Vec<TagProfile>;

const MAX_ID: u64 = std::u64::MAX;

#[derive(Debug, Clone)]
pub struct TagProfile {
    pub id: u64,
    pub tag_type: FLVTagType,
    pub timestamp_us: i64,
    pub position: u64, // position or channels for mute_tag
    pub sequence_header: bool,
    pub keyframe: bool,
    pub decode_duration_us: i64,// duration unit may ms or us
    pub offset_us: i64,
    pub deleted: bool,
}

impl TagProfile {
    pub fn new_video(id: u64, timestamp_us: i64, position: u64, sequence_header: bool, keyframe: bool, pts: i64) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_VIDEO,
            timestamp_us,
            position,
            sequence_header,
            keyframe,
            decode_duration_us: pts,
            offset_us: 0,
            deleted: false,
        }
    }

    pub fn new_audio(id: u64, timestamp_us: i64, position: u64, sequence_header: bool, decode_duration_us: i64) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_AUDIO,
            timestamp_us,
            position,
            sequence_header,
            keyframe: sequence_header,
            decode_duration_us,
            offset_us: 0,
            deleted: false,
        }
    }

    pub fn new_meta(id: u64, timestamp_us: i64, position: u64) -> Self {
        TagProfile {
            id,
            tag_type: FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT,
            timestamp_us,
            position,
            sequence_header: false,
            keyframe: false,
            decode_duration_us: 0,
            offset_us: 0,
            deleted: false,
        }
    }

    pub fn tag(&self, file: &mut File) -> FLVTag {
        if self.id == MAX_ID { // generate mut audio tag
            TagProfile::new_mute_tag(self.timestamp_us, self.position as u8)
        } else {
            file.seek(SeekFrom::Start(self.position)).unwrap();
            FLVTag::read(file).unwrap()
        }
    }

    pub fn new_mute(timestamp_us: i64, sample_rate: u32, channels: u8) -> TagProfile {
        // mute tag duration'unit is us
        TagProfile::new_audio(MAX_ID, timestamp_us, channels as u64, false, (1000_000.0 * 1024.0 / sample_rate as f64) as i64)
    }

    pub fn with_timestamp_us(mut self, timestamp_us: i64) -> Self {
        self.timestamp_us = timestamp_us;
        self
    }

    pub fn new_mute_tag(timestamp_us: i64, channels: u8) -> FLVTag {
        let sound_rate = 44100_f64;
        let sound_size: u8 = 16; // 16 | 8

        let data_list = ([0x01, 0x18, 0x20, 0x07], [0x21, 0x10, 0x04, 0x60, 0x8c, 0x1c]);
        let data: &[u8] = if channels == 2 { &data_list.1 } else { &data_list.0 };
        let data_size: usize = data.len() + 2;
        let channels_index   = channels - 1; 
        let sound_size_index = sound_size / 8 - 1;
        let sound_rate_index = [5512.5_f64, 11025_f64, 22050_f64, 44100_f64].iter().position(|&x| x == sound_rate).unwrap() as u8;
        let mut tag_data: Vec<u8> = Vec::with_capacity(data_size + 11 + 4);
        
        let timestamp: i64 = timestamp_us / 1000;
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

        let tag = FLVTag::read(&mut &*tag_data).unwrap();
        tag
    }
}

fn get_info(path: &str) -> Result<(FLVInfo, u32, u8), String> {
    
    let mut file = File::open(path).unwrap();
    let file_info = file.metadata().unwrap();
    let file_len = file_info.len();
    let mut parser = FLVTagRead::new(&mut file);
    
    // 只有一路av流，不存在音画不同步
    if !(parser.header.hasAudioTags && parser.header.hasVideoTags) {
        return Err("only one video/audio stream.".to_string());
    }

    let mut id: u64 = 0;

    let mut info: FLVInfo = Vec::new();

    // asc
    let mut asc: Option<AudioSpecificConfig> = None;

    loop {
        let position = parser.get_position();
        write!(std::io::stderr(), "scan progress: {: >3.0}%\r", position as f64 / file_len as f64 * 100.).unwrap();
        let nxt: Option<FLVTag> = parser.next();
        if nxt.is_none() {
            break;
        }
        let tag: FLVTag = nxt.unwrap();
        
        match tag.get_tag_type() {
            FLVTagType::TAG_TYPE_VIDEO => {
                let timestamp = tag.get_timestamp() as i64;
                let sequence_header = tag.get_frame_type() == 1 && tag.get_avc_packet_type() == 0;
                let keyframe = tag.get_frame_type() == 1;
                let cts = if sequence_header { 0 } else { tag.get_avc_composition_time_offset() as i64 };
                let pts = cts + timestamp;
                info.push(TagProfile::new_video(id, timestamp * 1000, position, sequence_header, keyframe, 
                    pts));
            },
            FLVTagType::TAG_TYPE_AUDIO => {
                if tag.get_sound_format() != flv_toolbox_rs::lib::SOUND_FORMAT_AAC {
                    return Err("sound format is not aac.".to_string());
                }
                if tag.is_acc_sequence_header() { // acc sequence header
                    asc = Some(tag.get_sound_audio_specific_config());
                    info.push(TagProfile::new_audio(id, tag.get_timestamp() as i64 * 1000, position, true, 0));
                } else { // normal frames
                    let duration: i64 = (1000. * tag.get_sound_frame_duration(asc.as_ref().unwrap())) as i64;
                    info.push(TagProfile::new_audio(id, tag.get_timestamp() as i64 * 1000, position, false, duration));
                }
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                info.push(TagProfile::new_meta(id, tag.get_timestamp() as i64 * 1000, position));
            }
        };
        id += 1;
    }
    eprintln!("scan complete!\r");
    match asc {
        None => Err("asc is none.".to_string()),
        Some(asc) => Ok((info, asc.get_sample_rate(), asc.channel_config))
    }
}

fn _top_duration(pairs: &BTreeMap<i64, u64>) -> i64 {
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

#[repr(C)]
pub struct OffsetInfo {
    id_from: libc::uint64_t,
    tm_from: libc::int64_t,
    id_to: libc::uint64_t,
    tm_to: libc::int64_t,
    current_offset: libc::int64_t,
    total_offset: libc::int64_t,
}

impl Encodable for OffsetInfo {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_struct("OffsetInfo", 6, |s| {
            try!(s.emit_struct_field("id_from", 0, |s| {
                s.emit_u64(self.id_from)
            }));
            try!(s.emit_struct_field("tm_from", 1, |s| {
                s.emit_i64(self.tm_from)
            }));
            try!(s.emit_struct_field("id_to", 2, |s| {
                s.emit_u64(self.id_to)
            }));
            try!(s.emit_struct_field("tm_to", 3, |s| {
                s.emit_i64(self.tm_to)
            }));
            try!(s.emit_struct_field("current_offset", 4, |s| {
                s.emit_i64(self.current_offset)
            }));
            try!(s.emit_struct_field("total_offset", 5, |s| {
                s.emit_i64(self.total_offset)
            }));
            Ok(())
        })
    }
}

fn check_offset(info: &mut FLVInfo) -> (Vec<OffsetInfo>, i64, i64) {

    let mut last_id: u64 = 0;
    let mut last_tm_us: i64 = 0;
    let mut last_dd_us: i64 = 0;
    let mut audio_duration_us: i64 = 0;
    // let mut has_gap: bool = false;
    let mut offset_infos: Vec<OffsetInfo> = Vec::new();
    let mut audio_tag_count: i64 = 0;

    for item in info.iter().filter(|&&TagProfile { ref tag_type, sequence_header: ref sh, ..}| *tag_type == FLVTagType::TAG_TYPE_AUDIO && !*sh) {
        let &TagProfile {
            ref id,
            timestamp_us: ref tm,
            decode_duration_us: ref dd,
            ..
        } = item;
        let delta: i64 = *tm - (last_tm_us + last_dd_us);
        if delta.abs() > 1000 {
            // has_gap = true;
            eprintln!("{:>6} {} -> {:>6} {} {:>8} {:>8}", last_id, format_seconds_ms(last_tm_us as u64 / 1000), *id, format_seconds_ms(*tm as u64 / 1000), delta, *tm - audio_duration_us);
            offset_infos.push( OffsetInfo {
                id_from: last_id,
                tm_from: last_tm_us,
                id_to: *id,
                tm_to: *tm,
                current_offset: delta,
                total_offset: *tm - audio_duration_us,
            });
        }
        audio_duration_us += *dd;
        last_id = *id;
        last_dd_us = *dd;
        last_tm_us = *tm;
        audio_tag_count += 1;
    }
    let count = offset_infos.len() as i64;
    (offset_infos, count, audio_tag_count)
}

fn offset_analysis(offset_infos: &Vec<OffsetInfo>) -> (i64, i64) {
    offset_infos.iter().fold((0, 0), |(max_offset, sum_offset), &OffsetInfo { current_offset: ref off, total_offset: ref total, .. }| {
        (if max_offset.abs() < (*total).abs() {
            *total
        } else {
            max_offset
        } , sum_offset + (*off).abs())
    })
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
            timestamp_us: ref tm,
            ref decode_duration_us,
            ref mut offset_us,
            ..
        } = a_tags[i];

        let delta: i64 = (timeline_timestamp + timeline_offset + timeline_decode_duration) - (*tm + timeline_offset);
        if delta.abs() > 1000 {
            let gap_left = timeline_timestamp + timeline_offset + timeline_decode_duration;
            let gap_right = *tm + timeline_offset;
            loop {
                let TagProfile {
                    timestamp_us: ref tm,
                    ref mut offset_us,
                    ref mut deleted,
                    ..
                } = v_tags[j];
                if *tm + timeline_offset <= gap_left {
                    *offset_us = timeline_offset;
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
            eprintln!("gap {:>6} {:>6} {:>6}", gap_left, gap_right, -delta);
            timeline_offset += delta;
        }
        timeline_decode_duration = *decode_duration_us;
        timeline_timestamp = *tm;
        *offset_us = timeline_offset;
    }
    // 处理剩下的v_tags
    while j < v_tags.len() {
        v_tags[j].offset_us = timeline_offset;
        j += 1;
    }

    // for (ref k, ref v) in delete_flags.iter() {
    //     eprintln!("del vtag {:>6}", k);
    // }
    
    eprintln!("before delete by gop, {} tags would be deleted", v_tags.iter().filter(|t| t.deleted).count());
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

    eprintln!("after delete by gop, {} tags would be deleted", for_delete_indices.len());

    for i in for_delete_indices.into_iter() {
        v_tags[i].deleted = true;
    }

    // mux profiles
    let mut new_profiles: Vec<TagProfile> = vec![];
    new_profiles.append(&mut a_tags);
    new_profiles.append(&mut v_tags.into_iter().filter(|t| !t.deleted).collect::<Vec<TagProfile>>());
    new_profiles.sort_by_key(|t| t.timestamp_us + t.offset_us);
    c_tags.append(&mut new_profiles);
    return c_tags;
}

// 使用插入空白 audio frame 方法补齐比较大的 gap
fn get_fix_info2(info: FLVInfo, mute_tag: TagProfile, offset_mode: bool) -> FLVInfo {
    
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
    let TagProfile { decode_duration_us: ref mute_tag_dd_us, .. } = mute_tag;

    let mut timeline_offset_us: i64 = 0;
    let mut timeline_timestamp_us: i64 = 0;
    let mut timeline_decode_duration_us: i64 = 0;
    let mut delta_acc: i64 = 0;
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
            timestamp_us: ref tm,
            ref decode_duration_us,
            ref mut offset_us,
            ..
        } = a_tags[i];

        let delta: i64 = (timeline_timestamp_us + timeline_offset_us + timeline_decode_duration_us) - (*tm + timeline_offset_us);
        if delta.abs() > 1000 {
            let mut gap_left_us = timeline_timestamp_us + timeline_offset_us + timeline_decode_duration_us;
            let gap_right_us = *tm + timeline_offset_us;
            // offset tags
            if offset_mode {
                // 填充到重叠
                while gap_right_us > gap_left_us {
                    b_tags.push(mute_tag.clone().with_timestamp_us(gap_left_us));
                    gap_left_us += *mute_tag_dd_us;
                }
                while j < v_tags.len() {
                    let TagProfile {
                        timestamp_us: ref tm,
                        ref mut offset_us,
                        ..
                    } = v_tags[j];
                    if *tm + timeline_offset_us <= gap_left_us {
                        *offset_us = timeline_offset_us;
                        j += 1;
                        continue;
                    } else {
                        break;
                    }
                }
                timeline_offset_us += gap_left_us - gap_right_us; // always increase offset
            } else {
                if gap_right_us > gap_left_us {
                    while gap_right_us - gap_left_us >= *mute_tag_dd_us {
                        b_tags.push(mute_tag.clone().with_timestamp_us(gap_left_us));
                        gap_left_us += *mute_tag_dd_us;
                    }
                    if gap_right_us - gap_left_us > 1000 {
                        eprintln!("remain offset {} {:>3}", format_seconds_ms(*tm as u64 / 1000), gap_right_us - gap_left_us);
                    }
                } else {
                    // 非 offset 模式, overlay 不能消除
                    eprintln!("overlay at {} {}", format_seconds_ms(*tm as u64 / 1000), gap_left_us - gap_right_us);
                }
                // gap accumulate
                delta_acc += gap_right_us - gap_left_us;
                while delta_acc >= *mute_tag_dd_us {
                    b_tags.push(mute_tag.clone().with_timestamp_us(gap_left_us));
                    gap_left_us += *mute_tag_dd_us;
                    delta_acc -= *mute_tag_dd_us;
                    eprintln!("accumulate gap fixed.");
                }
            }
        }
        timeline_decode_duration_us = *decode_duration_us;
        timeline_timestamp_us = *tm;
        *offset_us = timeline_offset_us;
    }
    // 处理完剩下的 video
    while j < v_tags.len() {
        v_tags[j].offset_us = timeline_offset_us;
        j += 1;
    }

    if offset_mode {
        // 计算可偏移点：目前为止最大的pts帧
        let mut max_pts: i64 = -1;
        for item in v_tags.iter_mut() {
            let &mut TagProfile {
                decode_duration_us: ref pts,
                deleted: ref mut allow_offset,
                ..
            } = item;
            if *pts > max_pts {
                *allow_offset = true;
                max_pts = *pts;
            }
        }
        // 处理 offset 帧的顺序问题: 增量的帧要从IDR帧或P帧开始
        let mut last_offset: i64 = 0;
        let mut delay_offset: Option<i64> = None;
        for item in v_tags.iter_mut() {
            let &mut TagProfile {
                ref mut offset_us,
                deleted: ref allow_offset,
                ..
            } = item;
            if *offset_us != last_offset { // 有变化 delay_offset 永远记录最新的 offset
                delay_offset = Some(*offset_us);
            }
            if *allow_offset { // 可偏移点
                if delay_offset.is_some() { // keyframe 消除一个delay，并且更新 last_offset
                    *offset_us = *(delay_offset.as_ref().unwrap());
                    last_offset = *offset_us;
                    delay_offset = None;
                }
            } else {
                if delay_offset.is_some() {
                    *offset_us = last_offset;
                }
            }
        }
    }

    eprintln!("b_tags len {}", b_tags.len());
    // mux profiles
    let mut new_profiles: Vec<TagProfile> = vec![];
    new_profiles.append(&mut a_tags);
    new_profiles.append(&mut b_tags);
    new_profiles.append(&mut v_tags);
    new_profiles.sort_by_key(|t| t.timestamp_us + t.offset_us);
    c_tags.append(&mut new_profiles);
    return c_tags;
}

fn fix_file(input: &str, output: &str, info: FLVInfo) -> Result<(), String> {

    // use std::io::SeekFrom::{Current, Start};
    {
        // do some guard
        // avc sequence header must be 1
        // aac sequence header must be 1
        // metadata must be 1 or 0
        let a_sh_len = info.iter().filter(|&&TagProfile { ref tag_type, sequence_header: ref sh, .. }| *tag_type == FLVTagType::TAG_TYPE_AUDIO && *sh).count();
        if a_sh_len != 1 {
            return Err(format!("audio sequence header tag count is not 1 but {}.", a_sh_len));
        }
        let v_sh_len = info.iter().filter(|&&TagProfile { ref tag_type, sequence_header: ref sh, .. }| *tag_type == FLVTagType::TAG_TYPE_VIDEO && *sh).count();
        if v_sh_len != 1 {
            return Err(format!("video sequence header tag count is not 1 but {}.", v_sh_len));
        }
        let m_len = info.iter().filter(|&&TagProfile { ref tag_type, .. }| *tag_type == FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT).count();
        if m_len > 1 {
            return Err(format!("metadata tag count is not 0 or 1 but {}.", m_len));
        }
    }

    let mut file = File::open(input).map_err(|e| format!("open input file err: {}", e))?;
    // let file_info = file.metadata().unwrap();
    // let file_len = file_info.len();
    
    let output_file: File = File::create(output).map_err(|e| format!("creat output file err: {}", e))?;
    let mut tag_write: FLVTagWrite<File> = FLVTagWrite::new(output_file);

    let header = FLVHeader::read(&mut file);
    tag_write.write_header(&header);

    // function from flv-split
    fn write_back_meta_tag<T: Write + Seek>(duration: u64, metatag: &mut FLVTag, times: &Vec<u64>, filepositions: &Vec<u64>, tag_write: &mut FLVTagWrite<T>) -> Result<(), String> {
        let mut metas = metatag.get_objects();
        {
            // if the updating of metadata was failed, then would write back the original metadata
            let r: Result<(), String> = (|| {
                metas[1].as_object_mut().ok_or("meta[1] is not object.".to_string())?.insert("duration".to_string(), Json::F64(duration as f64 / 1000.0));
                metas[1].as_object_mut().unwrap().insert("gapfixedby".to_string(), Json::String(PROGRAM_SIGN.to_string()));
                let key_times = Json::Array(times.iter().map(|&t| Json::F64(t as f64 / 1000.0)).collect::<Vec<Json>>());
                let key_positions = Json::Array(filepositions.iter().map(|&p| Json::F64(p as f64)).collect::<Vec<Json>>());
                let keyframes = metas[1].as_object_mut().unwrap().get_mut("keyframes").ok_or("meta[1].keyframes dose not exits.".to_string())?.as_object_mut().ok_or("meta[1].keyframes is not object.".to_string())?;
                keyframes.insert("times".to_string(), key_times);
                keyframes.insert("filepositions".to_string(), key_positions);
                Ok(())
            })();
            match r {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("update metadata error: {}", err);
                }
            }
        }
        metatag.set_objects(&metas);
        tag_write.write_meta_tag(&metatag);
        Ok(())
    }

    // create metatag
    let times = info.iter()
        .filter(|&&TagProfile { ref tag_type, ref keyframe, .. }| *tag_type == FLVTagType::TAG_TYPE_VIDEO && *keyframe )
        .map(|&TagProfile { timestamp_us: ref t, .. }| *t as u64 / 1000).collect::<Vec<u64>>();
    let mut positions: Vec<u64> = vec![0u64; times.len()];
    let mut metatag = info.iter().find(|&&TagProfile { ref tag_type, .. }| *tag_type == FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT).map(|item| item.tag(&mut file));
    let duration = {
        let item = info.iter().filter(|&&TagProfile { ref tag_type, .. }| *tag_type == FLVTagType::TAG_TYPE_AUDIO).last().ok_or("no any audio tags.".to_string())?;
        (item.timestamp_us + item.decode_duration_us) as u64 / 1000
    };

    if metatag.is_some() {
        // write metatag
        match write_back_meta_tag(duration, metatag.as_mut().unwrap(), &times, &positions, &mut tag_write) {
            Ok(_) => {},
            Err(msg) => {
                eprintln!("write metatag err, but fix is proceeding: {}", msg);
            }
        };
    } else {
        eprintln!("can't find metatag, but fix is proceeding.");
    }

    let mut frame_index: usize = 0;

    for item in info.iter() {
        let &TagProfile {
            ref tag_type,
            ref keyframe,
            ref timestamp_us,
            ref offset_us,
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
        tag.set_timestamp((*timestamp_us + *offset_us) as u64 / 1000);
        tag_write.write_tag(&tag);
    }

    if metatag.is_some() {
        // write metatag
        match write_back_meta_tag(duration, metatag.as_mut().unwrap(), &times, &positions, &mut tag_write) {
            Ok(_) => {},
            Err(msg) => {
                eprintln!("write metatag err, but fix is proceeding: {}", msg);
            }
        };
    }
    Ok(())
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    write!(std::io::stderr(), "{}", opts.usage(&brief)).unwrap();
}

fn return_code(code: i32, output: bool, msg: Option<&str>, data: Option<(Vec<OffsetInfo>, i64, i64, i64, i64)>, need_fix: Option<bool>) -> i32 {

    let mut out = std::io::stdout();
    write!(out, "{{\"code\": {}", code).unwrap();
    write!(out, ", \"output\": {}", output).unwrap();
    if msg.is_some() {
        write!(out, ", \"message\": {}", rustc_serialize::json::encode(msg.as_ref().unwrap()).unwrap()).unwrap();
    }
    match data {
        Some((infos, max_offset, sum_offset, offset_tag_count, audio_tag_count)) => {
            write!(out, ", \"data\": {}", rustc_serialize::json::encode(&infos).unwrap()).unwrap();
            write!(out, ", \"max_offset\": {}", max_offset).unwrap();
            write!(out, ", \"sum_offset\": {}", sum_offset).unwrap();
            write!(out, ", \"offset_tag_count\": {}", offset_tag_count).unwrap();
            write!(out, ", \"audio_tag_count\": {}", audio_tag_count).unwrap();
            write!(out, ", \"offset_rate\": {}", offset_tag_count as f64 / audio_tag_count as f64).unwrap();
        }
        _ => {}
    };
    match need_fix {
        Some(nf) => {
            write!(out, ", \"need_fix\": {}", nf).unwrap();
        }
        _ => {}
    }
    write!(out, "}}\n").unwrap();
    return code;
}

fn main() {
    let ret: i32 = app();
    if ret == -1 {
        std::process::exit(ret);
    }
}

fn app() -> i32 {
    
    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflagopt("o", "output", "output flv file", "OUTPUT");
    opts.optflagopt("t", "threshold", "when in fix mode, only max_offset > threshold would be fixed, in microseconds, default 0", "THRESHOLD");
    opts.optflagopt("r", "offset_rate_threshold", "when in fix mode, only offset_rate < offset_rate_threshold would be fixed, in 0-1, double number, default 0.01", "OFFSET_RATE");
    opts.optflag("d", "drop-video", "fix audio gap by drop video frames");
    opts.optflag("b", "fill-mute-audio", "fix audio gap by fill mute audio frames");
    opts.optflag("f", "offset", "fill mute audio, also offset video frame to avoid gap");
    opts.optflag("v", "verbose", "show more information");
    opts.optflag("h", "help", "print this help menu");

    let matches: getopts::Matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            // panic!(f.to_string())
            return return_code(-1, false, Some(&f.to_string()), None, None);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, opts);
        return return_code(-1, false, None, None, None);
    }

    let input: String = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        eprintln!("no input file.");
        print_usage(&program, opts);
        return return_code(-1, false, None, None, None);
    };

    let input_path: &Path = Path::new(&input);
    if !input_path.exists() {
        eprintln!("input file does not exist.");
        print_usage(&program, opts);
        return return_code(-1, false, None, None, None);
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
            eprintln!("no output file, use {}", output.to_str().unwrap());
            output.to_string_lossy().to_string()
        }
    };

    let _verbose    = matches.opt_present("v");
    let drop_mode   = matches.opt_present("d");
    let fill_mode   = matches.opt_present("b");
    let offset_mode = matches.opt_present("f");
    let fix_mode    = drop_mode || fill_mode || offset_mode;

    let mut has_threshold: bool = false;
    let threshold = match matches.opt_default("t", "0") {
        Some(t) => {
            has_threshold = true;
            match i64::from_str_radix(&t, 10) {
                Ok(i) => i.abs(),
                Err(e) => {
                    if fix_mode { // fix mode
                        eprintln!("param threshold parse err: {}, use default 0.", e);
                    }
                    0
                }
            }
        },
        _ => {
            0
        }
    };

    let offset_rate_threshold = match matches.opt_default("r", "0.01") {
        Some(t) => {
            has_threshold = true;
            match f64::from_str(&t) {
                Ok(f) => f,
                Err(e) => {
                    if fix_mode {
                        eprintln!("param offset_rate_threshold parse err: {}, use default 0.01", e);
                    }
                    0.01_f64
                }
            }
        },
        _ => {
            0.01_f64
        }
    };

    eprintln!("checking flv file: {}", input);
    let (mut info, sample_rate, channels) = match get_info(&input) {
        Ok(ret) => ret,
        Err(msg) => {
            eprintln!("{}", msg);
            return return_code(-1, false, Some(&msg), None, None);
        }
    };
    let (offset_infos, offset_tag_count, audio_tag_count) = check_offset(&mut info);
    let offset_rate = offset_tag_count as f64 / audio_tag_count as f64;
    let has_gap = offset_infos.len() != 0;

    let mut need_fix: Option<bool> = None;
    if has_gap {
        let (max_offset, sum_offset) = offset_analysis(&offset_infos);
        if has_threshold {
            need_fix = Some(max_offset.abs() > threshold && offset_rate <= offset_rate_threshold);
        }
        eprintln!("max_offset: {}, sum_offset: {}, offset_rate: {:.6}, offset_rate_threshold: {:.6}", max_offset, sum_offset, offset_rate, offset_rate_threshold);
        if fix_mode {
            if max_offset.abs() < threshold {
                let msg = format!("max_offset(abs({})) < threshold({}), no fix.", max_offset, threshold);
                eprintln!("{}", msg);
                return return_code(1, false, Some(&msg), Some((offset_infos, max_offset, sum_offset, offset_tag_count, audio_tag_count)), need_fix); 
            }
            if offset_rate > offset_rate_threshold {
                let msg = format!("offset_rate({}) < offset_rate_threshold({}), no fix.", offset_rate, offset_rate_threshold);
                eprintln!("{}", msg);
                return return_code(1, false, Some(&msg), Some((offset_infos, max_offset, sum_offset, offset_tag_count, audio_tag_count)), need_fix); 
            }
            let new_info = if drop_mode {
                get_fix_info(info)
            } else {
                // eprintln!("{:?}", (TagProfile::new_mute_tag(0)));
                get_fix_info2(info, TagProfile::new_mute(0, sample_rate, channels), offset_mode)
            };
            match fix_file(&input, &output, new_info) {
                Ok(_) => {
                    eprintln!("flv fix complete.\nplease use `ffmpeg -i \"{}\" -acodec copy -vcodec copy \"{}\"` to get mp4 file.", &output, Path::new(&output).with_extension("mp4").to_str().unwrap());
                    return return_code(1, true, None, Some((offset_infos, max_offset, sum_offset, offset_tag_count, audio_tag_count)), need_fix);
                }
                Err(msg) => {
                    eprintln!("fix flv file err: {}", msg);
                    return return_code(-1, false, Some(&msg), None, need_fix);
                }
            };
        } else {
            eprintln!("there are audio gaps, please set the complete fix mode (-b -f) to fix them.");
            return return_code(1, false, None, Some((offset_infos, max_offset, sum_offset, offset_tag_count, audio_tag_count)), need_fix);
        }
    } else {
        eprintln!("no gap.");
        return return_code(0, false, None, None, need_fix);
    }
}

#[no_mangle]
pub extern fn check(input_str: *const libc::c_char) -> *mut libc::c_char {
    use std::ffi::{ CStr, CString };
    use std::fmt::Write;

    fn code(c: i32, msg: Option<&str>, data: Option<Vec<OffsetInfo>>) -> *mut libc::c_char {
        let mut out: String = String::new();
        write!(out, "{{ \"code\": {}", c).unwrap();
        if msg.is_some() {
            write!(out, ", \"message\": {}", rustc_serialize::json::encode(msg.as_ref().unwrap()).unwrap()).unwrap();
        }
        if data.is_some() {
            write!(out, ", \"data\": {}", rustc_serialize::json::encode(&data).unwrap()).unwrap();
        }
        write!(out, "}}").unwrap();
        CString::new(out).unwrap().into_raw()
    }

    let input_cstr = unsafe {
        assert!(!input_str.is_null());
        CStr::from_ptr(input_str)
    };
    let input: String = input_cstr.to_string_lossy().to_string();
    let input_path: &Path = Path::new(&input);
    if !input_path.exists() {
        return code(-1, Some("input file does not exist."), None);
    }

    let (mut info, _sample_rate, _) = match get_info(&input) {
        Ok(ret) => ret,
        Err(msg) => {
            return code(-1, Some(&msg), None);
        }
    };
    let (offset_infos, _, _) = check_offset(&mut info);
    code(if offset_infos.len() == 0 { 0 } else { 1 }, None, Some(offset_infos))
}

#[no_mangle]
pub extern fn check_free(ret: *mut libc::c_char) {
    use std::ffi::CString;

    unsafe {
        if ret.is_null() { return }
        CString::from_raw(ret)
    };
}