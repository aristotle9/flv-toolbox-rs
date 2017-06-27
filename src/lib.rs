extern crate rustc_serialize;
extern crate byteorder;
extern crate xml;

use std::collections::BTreeMap;
use std::io::{Read, Write, Cursor, Seek, SeekFrom};
use self::byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use self::rustc_serialize::json::Json;
use std::fmt;

pub trait ReadAMF0Ext : ReadBytesExt {
    fn read_amf0_number(&mut self) -> Json {
        Json::F64(self.read_f64::<BigEndian>().unwrap())
    }

    fn read_amf0_boolean(&mut self) -> Json {
        Json::Boolean(self.read_u8().unwrap() != 0)
    }

    fn read_amf0_string(&mut self) -> Json {
        let len = self.read_u16::<BigEndian>().unwrap();
        Json::String(self.read_amf0_raw_string(len as usize))
    }

    fn read_amf0_raw_string(&mut self, len: usize) -> String {
        let mut buffer: Vec<u8> = Vec::with_capacity(len);
        let mut handle = self.take(len as u64);
        let read_len = handle.read_to_end(&mut buffer).unwrap();
        assert_eq!(len, read_len);
        String::from_utf8(buffer).unwrap()
    }

    fn read_amf0_ecma_array(&mut self) -> Json {
        let _count = self.read_u32::<BigEndian>().unwrap();
        self.read_amf0_object()
    }

    fn read_amf0_strict_array(&mut self) -> Json {
        let count = self.read_u32::<BigEndian>().unwrap();
        let mut v: Vec<Json> = Vec::with_capacity(count as usize);
        for _ in 0..count {
            v.push(self.read_amf0_value());
        }
        Json::Array(v)
    }

    fn read_amf0_object(&mut self) -> Json {
        let mut obj = BTreeMap::new();
        loop {
            let len = self.read_u16::<BigEndian>().unwrap() as usize;
            if len == 0 {
                let end_mark = self.read_u8().unwrap();
                assert_eq!(0x09, end_mark);
                break;
            }
            else {
                let key = self.read_amf0_raw_string(len);
                let val = self.read_amf0_value();
                obj.insert(key, val);
            }
        }
        Json::Object(obj)
    }

    fn read_amf0_value(&mut self) -> Json {
        match self.read_u8().unwrap() {
            0x00 => self.read_amf0_number(),
            0x01 => self.read_amf0_boolean(),
            0x02 => self.read_amf0_string(),
            0x03 => self.read_amf0_object(),
            0x05 => Json::Null,
            0x06 => Json::Null,
            0x08 => self.read_amf0_ecma_array(),
            0x0A => self.read_amf0_strict_array(),
            n => panic!(format!("unsupported mark {}", n))
        }
    }
}

impl<R: Read + ?Sized> ReadAMF0Ext for R {}

pub trait WriteAMF0Ext : WriteBytesExt {
    fn write_amf0_number(&mut self, f: f64) {
        self.write_u8(0x00).unwrap();
        self.write_f64::<BigEndian>(f).unwrap();
    }

    fn write_amf0_raw_string(&mut self, s: &String) {
        let len = s.as_bytes().len();
        self.write_u16::<BigEndian>(len as u16).unwrap();
        let write_len = self.write(s.as_bytes()).unwrap();
        assert_eq!(len, write_len);
    }

    fn write_amf0_string(&mut self, s: &String) {
        self.write_u8(0x02).unwrap();
        self.write_amf0_raw_string(s);
    }

    fn write_amf0_boolean(&mut self, b: bool) {
        self.write_u8(0x01).unwrap();
        self.write_u8(if b { 1 } else { 0 }).unwrap();
    }

    fn write_amf0_strict_array(&mut self, vec: &Vec<Json>) {
        self.write_u8(0x0A).unwrap();
        let len = vec.len();
        self.write_u32::<BigEndian>(len as u32).unwrap();
        for v in vec.iter() {
            self.write_amf0_value(v)
        }
    }

    fn write_amf0_object(&mut self, o: &BTreeMap<String, Json>) {
        self.write_u8(0x03).unwrap();
        for (ref key, ref v) in o.iter() {
            self.write_amf0_raw_string(key);
            self.write_amf0_value(v);
        }
        self.write_u16::<BigEndian>(0).unwrap();
        self.write_u8(0x09).unwrap();
    }

    fn write_amf0_null(&mut self) {
        self.write_u8(0x05).unwrap();
    }

    fn write_amf0_value(&mut self, obj: &Json) {
        use rustc_serialize::json::Json::*;
        match obj {
            &I64(ref i) => self.write_amf0_number(*i as f64),
            &U64(ref u) => self.write_amf0_number(*u as f64),
            &F64(ref f) => self.write_amf0_number(*f),
            &String(ref s) => self.write_amf0_string(s),
            &Boolean(ref b) => self.write_amf0_boolean(*b),
            &Array(ref a) => self.write_amf0_strict_array(a),
            &Object(ref o) => self.write_amf0_object(o),
            &Null => self.write_amf0_null(),
        }
    }
}

impl<W: Write + ?Sized> WriteAMF0Ext for W {}

#[test]
fn write_and_read() {
    return;
    fn test_amf(v: Json) {
        let mut c = Cursor::new(Vec::<u8>::new());
        c.write_amf0_value(&v);
        c.set_position(0);
        let v2 = c.read_amf0_value();
        assert_eq!(v, v2);
        println!("test success: {:?}", v);
    }

    test_amf(Json::Null);
    test_amf(Json::F64(1f64));
    test_amf(Json::Boolean(true));
    test_amf(Json::Boolean(false));
    test_amf(Json::String("Hello".to_string()));
    test_amf(Json::Array(vec![Json::Null, Json::F64(0f64)]));
    let mut obj = BTreeMap::<String, Json>::new();
    obj.insert("Hello".to_string(), Json::Boolean(false));
    test_amf(Json::Object(obj));
}

fn read_u24_be(r: &mut Read) -> u32 {
    let (b1, b2, b3) = (r.read_u8().unwrap() as u32, r.read_u8().unwrap() as u32, r.read_u8().unwrap() as u32);
    b1 << 16 | b2 << 8 | b3
}

fn write_u24_be(w: &mut Write, n: u32) {
    w.write_u8(((n >> 16) & 0xff) as u8).unwrap();
    w.write_u8(((n >> 8 ) & 0xff) as u8).unwrap();
    w.write_u8(((n      ) & 0xff) as u8).unwrap();
}

pub fn format_seconds_ms(ms: u64) -> String {
    let mut ms = ms;
    let mm = ms % 1000;
	ms /= 1000;
    let s = ms % 60;
	ms /= 60;
    let m = ms % 60;
	ms /= 60;
	let h = ms;

	if h != 0 {
		format!("{:02}:{:02}:{:02}.{:03}", h, m, s, mm)
	}
    else {
        format!("{:02}:{:02}.{:03}", m, s, mm)
    }
}

const TAG_HEADER_BYTE_COUNT: u32 = 11;
const PREV_TAG_BYTE_COUNT: u32 = 4;
const MIN_FILE_HEADER_BYTE_COUNT: u32 = 9;

#[derive(Debug, PartialEq, Clone)]
pub enum FLVTagType {
    TAG_TYPE_AUDIO = 8,
    TAG_TYPE_VIDEO = 9,
    TAG_TYPE_SCRIPTDATAOBJECT = 18,
}

impl From<u8> for FLVTagType {
    fn from(t: u8) -> FLVTagType {
        use std;

        let v: Vec<u8> = vec![8, 9, 18];
        if !v.contains(&t) {
            panic!(format!("unknown tagType: {}", t));
        }
        unsafe {
            std::mem::transmute(t)
        }
    }
}

pub struct FLVHeader {
    pub hasAudioTags: bool,
    pub hasVideoTags: bool,
}

impl FLVHeader {
    pub fn read(r: &mut Read) -> FLVHeader {
        assert_eq!(r.read_u8().unwrap(), 'F' as u8);
        assert_eq!(r.read_u8().unwrap(), 'L' as u8);
        assert_eq!(r.read_u8().unwrap(), 'V' as u8);
        assert_eq!(r.read_u8().unwrap(), 1);
        let flags = r.read_u8().unwrap();
        assert_eq!(r.read_u32::<BigEndian>().unwrap(), 9);
        r.read_u32::<BigEndian>().unwrap();

        FLVHeader {
            hasAudioTags: (flags & 0x04) > 0,
            hasVideoTags: (flags & 0x01) > 0,
        }
    }

    pub fn write(&self, w: &mut Write) {
        w.write_u8('F' as u8).unwrap();
        w.write_u8('L' as u8).unwrap();
        w.write_u8('V' as u8).unwrap();
        w.write_u8(1).unwrap();
        let mut flags = 0;
        if self.hasAudioTags {
            flags |= 0x04;
        }
        if self.hasVideoTags {
            flags |= 0x01;
        }
        w.write_u8(flags).unwrap();
        w.write_u32::<BigEndian>(9).unwrap();
        w.write_u32::<BigEndian>(0).unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct FLVTag {
    data: Vec<u8>//tag without last 4 bytes
}

impl FLVTag {
    pub fn get_tag_type(&self) -> FLVTagType {
        FLVTagType::from(self.data[0])
    }

    pub fn get_tag_size(&self) -> u32 {
        TAG_HEADER_BYTE_COUNT + self.get_data_size()
    }

    pub fn get_data_size(&self) -> u32 {
        ((self.data[1] as u32) << 16) | ((self.data[2] as u32) << 8) | (self.data[3] as u32)
    }

    pub fn set_data_size(&mut self, value: u32) {
        self.data[1] = ((value >> 16) & 0xff) as u8;
		self.data[2] = ((value >>  8) & 0xff) as u8;
		self.data[3] = ((value      ) & 0xff) as u8;
		//bytes.length = TAG_HEADER_BYTE_COUNT + value;
    }

    pub fn get_timestamp(&self) -> u64 {
        ((self.data[7] as u64) << 24) | ((self.data[4] as u64) << 16) | ((self.data[5] as u64) << 8) | (self.data[6] as u64)
    }

    pub fn set_timestamp(&mut self, value: u64) {
        self.data[7] = ((value >> 24) & 0xff) as u8; // extended byte in unusual location
    	self.data[4] = ((value >> 16) & 0xff) as u8;
    	self.data[5] = ((value >> 8 ) & 0xff) as u8;
    	self.data[6] = ((value      ) & 0xff) as u8;
    }

    pub fn read(r: &mut Read) -> Option<FLVTag>{
        let tag_type = match r.read_u8() {
            Ok(n) => n,
            Err(..) => return None
        };
        let data_size = read_u24_be(r);
        let tag_size = TAG_HEADER_BYTE_COUNT + data_size;
        let mut payload: Vec<u8> = Vec::with_capacity(tag_size as usize);

        payload.write_u8(tag_type).unwrap();
        write_u24_be(&mut payload, data_size as u32);

        {
            let mut handle = r.take((tag_size - 1 - 3) as u64);
            let read_len = handle.read_to_end(&mut payload).unwrap();
            assert_eq!((tag_size - 4) as usize, read_len);
        }
        r.read_u32::<BigEndian>().unwrap();

        Some(FLVTag {
            data: payload
        })
    }

    pub fn write(&self, w: &mut Write) {
        w.write(&self.data[..(self.get_tag_size() as usize)]).unwrap();
        w.write_u32::<BigEndian>(self.get_tag_size()).unwrap();
    }
}

impl FLVTag {
    pub fn get_objects(&self) -> Vec<Json> {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);
        let mut v: Vec<Json> = Vec::with_capacity(2);
        let mut handle = Cursor::new(&self.data[(TAG_HEADER_BYTE_COUNT as usize)..]);
        v.push(handle.read_amf0_value());
        v.push(handle.read_amf0_value());
        v
    }

    pub fn set_objects(&mut self, vec: &Vec<Json>) {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);
        
        let mut buf: Vec<u8> = Vec::new();
        buf.write(&self.data[0..(TAG_HEADER_BYTE_COUNT as usize)]);
        buf.write_amf0_value(&vec[0]);
        buf.write_amf0_value(&vec[1]);
        let data_size = buf.len() as u32 - TAG_HEADER_BYTE_COUNT;
        self.data = buf;

        self.set_data_size(data_size);
    }
}

#[derive(Debug, Clone)]
pub struct AvcC {
    pub version: u8,
    pub profile: u8,
    pub compatibility: u8,
    pub level: u8,
    pub nalu_length_size_minus_1: u8,
    pub num_of_sps: u8,
    pub sps: Vec<u8>,
    pub pps_array: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct NalUnitInfo {
    pub nalu_type: u8,
    pub nalu_tag: u8,
    pub nalu_size: u32,
}

impl fmt::Display for NalUnitInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tp  = match self.nalu_type {
            1 => "   ", // IBP
            5 => "IDR", // IDR
            6 => "SEI", // SEI
            7 => "SPS", // SPS
            8 => "PPS", // PPS
            _ => "Unknown Nalu Type",
        };
        let tag = match self.nalu_tag {
            0b11 => "I",
            0b10 => "P",
            0b00 => "B",
            _    => "X", // panic!("unknow nalu type"),
        };
        write!(f, "[{} {} {:>5}]", tp, tag, self.nalu_size)
    }
}

pub struct NalUnitInfos(pub Vec<NalUnitInfo>);

impl fmt::Display for NalUnitInfos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for info in self.0.iter() {
            try!(write!(f, "{} ", info));
        }
        Ok(())
    }
}

impl FLVTag {
    pub fn get_frame_type(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_VIDEO);
        (self.data[TAG_HEADER_BYTE_COUNT as usize + 0] >> 4) & 0x0f
    }

    pub fn get_codec_id(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_VIDEO);
        (self.data[TAG_HEADER_BYTE_COUNT as usize + 0] & 0x0f)
    }

    pub fn get_avc_packet_type(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_VIDEO);
        self.data[TAG_HEADER_BYTE_COUNT as usize + 1]
    }

    pub fn get_avc_composition_time_offset(&self) -> i32 {
        assert!(self.get_codec_id() == 7 || self.get_codec_id() == 12); // CODEC_ID_AVC
        // assert_eq!(self.get_avc_packet_type(), 1); 
        // AVC_PACKET_TYPE_SEQUENCE_HEADER 0
        // AVC_PACKET_TYPE_NALU            1
        // AVC_PACKET_TYPE_END_OF_SEQUENCE 2
        if self.get_avc_packet_type() != 1 {
            return 0;
        }
        let mut value: i32 = (self.data[TAG_HEADER_BYTE_COUNT as usize + 2] as i32) << 16;
        value |= (self.data[TAG_HEADER_BYTE_COUNT as usize + 3] as i32) << 8;
        value |= self.data[TAG_HEADER_BYTE_COUNT as usize + 4] as i32;
        if (value & 0x00800000) != 0 {
            value |= 0xff000000;	// sign-extend the 24-bit read for a 32-bit int
        }
        return value;
    }

    pub fn get_nal_uints_info(&self) -> NalUnitInfos { // see see what in the data
        
        use std::fmt::Write;
        
        assert!(self.get_codec_id() == 7 || self.get_codec_id() == 12); // CODEC_ID_AVC
        let mut handle: Cursor<&[u8]> = Cursor::new(&self.data[(TAG_HEADER_BYTE_COUNT as usize)..]);
        let mut ret: Vec<NalUnitInfo> = Vec::new();

        handle.seek(SeekFrom::Start(5)); // seek to nalus
        let data_size = self.get_data_size() as u64;
        loop {
            let bytes_avaliable = data_size - handle.seek(SeekFrom::Current(0)).unwrap();
            if bytes_avaliable < 4 {
                break;
            }
            let nalu_size = handle.read_u32::<BigEndian>().unwrap();
            if bytes_avaliable < 4 + nalu_size as u64 {
                break;
            }
            let nalu_type = handle.read_u8().unwrap();
            let tp  = nalu_type & 0x1F;
            let tag = (nalu_type >> 5) & 0x3;
            ret.push(NalUnitInfo {
                        nalu_type: tp,
                        nalu_tag: tag,
                        nalu_size,
                    });
            handle.seek(SeekFrom::Current((nalu_size as i64) - 1));
        }
        return NalUnitInfos(ret);
    }

    // for ffmpeg avpacket
    pub fn get_nal_units(&self) -> Vec<Vec<u8>> {
        
        let mut units: Vec<Vec<u8>> = Vec::new();
        let mut handle: Cursor<&[u8]> = Cursor::new(&self.data[(TAG_HEADER_BYTE_COUNT as usize)..]);

        handle.seek(SeekFrom::Start(5)); // seek to nalus
        let data_size = self.get_data_size() as u64;
        loop {
            let bytes_avaliable = data_size - handle.seek(SeekFrom::Current(0)).unwrap();
            if bytes_avaliable < 4 {
                break;
            }
            let nalu_size = handle.read_u32::<BigEndian>().unwrap() as u64;
            if bytes_avaliable < 4 + nalu_size {
                break;
            }
            let mut nalu_data: Vec<u8> = vec![0; nalu_size as usize];
            handle.read_exact(&mut nalu_data).unwrap();
            units.push(nalu_data);
        }
        return units;
    }

    pub fn get_avcc_data(&self) -> &[u8] {
        &self.data[(TAG_HEADER_BYTE_COUNT as usize + 5)..]
    }

    pub fn get_avcc(&self) -> AvcC {
        
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_VIDEO);
        assert_eq!(self.get_codec_id(), 7); // CODEC_ID_AVC
        assert_eq!(self.get_avc_packet_type(), 0); // AVC_PACKET_TYPE_SEQUENCE_HEADER
        
        let mut data: Cursor<&[u8]> = Cursor::new(&self.data);
        data.seek(SeekFrom::Start((TAG_HEADER_BYTE_COUNT + 5) as u64));
        // parse start
        let version: u8 = data.read_u8().unwrap();
        let profile: u8 = data.read_u8().unwrap();
        let compatibility: u8 = data.read_u8().unwrap();
        let level: u8 = data.read_u8().unwrap();
        
        let nalu_length_size_minus_1: u8 = data.read_u8().unwrap() & 0b00000011;
        let num_of_sps: u8 = data.read_u8().unwrap() & 0b00011111;
        assert_eq!(num_of_sps, 1);
        
        let sps_len = data.read_u16::<BigEndian>().unwrap();
        let mut sps: Vec<u8> = vec![0; sps_len as usize];
        data.read_exact(&mut sps).unwrap();

        let num_of_pps = data.read_u8().unwrap();
        let mut pps_array: Vec<Vec<u8>> = Vec::with_capacity(num_of_pps as usize);
        for _ in 0..(num_of_pps as usize) {
            let pps_len = data.read_u16::<BigEndian>().unwrap();
            let mut pps: Vec<u8> = vec![0; pps_len as usize];
            data.read_exact(&mut pps).unwrap();
            pps_array.push(pps);
        }

        AvcC {
            version,
            profile,
            compatibility,
            level,
            nalu_length_size_minus_1,
            num_of_sps,
            sps,
            pps_array,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioSpecificConfig {
    pub original_audio_object_type: u8,
    pub audio_object_type: u8,
    pub sample_index: u8,
    pub channel_config: u8,
}

impl AudioSpecificConfig {
    pub fn get_sample_rate(&self) -> u32 {
        let rate_list: [u32; 13] = [96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350];
        return rate_list[self.sample_index as usize];
    }
}

pub const SOUND_FORMAT_LINEAR: u8 = 0;
pub const SOUND_FORMAT_ADPCM: u8 = 1;
pub const SOUND_FORMAT_MP3: u8 = 2;
pub const SOUND_FORMAT_LINEAR_LE: u8 = 3;
pub const SOUND_FORMAT_NELLYMOSER_16K: u8 = 4;
pub const SOUND_FORMAT_NELLYMOSER_8K: u8 = 5;
pub const SOUND_FORMAT_NELLYMOSER: u8 = 6;
pub const SOUND_FORMAT_G711A: u8 = 7;
pub const SOUND_FORMAT_G711U: u8 = 8;
pub const SOUND_FORMAT_AAC: u8 = 10;
pub const SOUND_FORMAT_SPEEX: u8 = 11;
pub const SOUND_FORMAT_MP3_8K: u8 = 14;
pub const SOUND_FORMAT_DEVICE_SPECIFIC: u8 = 15;

pub const SOUND_RATE_5K : f64 = 5512.5_f64;
pub const SOUND_RATE_11K: f64 = 11025_f64;
pub const SOUND_RATE_22K: f64 = 22050_f64;
pub const SOUND_RATE_44K: f64 = 44100_f64;

pub const SOUND_SIZE_8BITS: u8 = 8;
pub const SOUND_SIZE_16BITS: u8 = 16;

pub const SOUND_CHANNELS_MONO: u8 = 1;
pub const SOUND_CHANNELS_STEREO: u8 = 2;

impl FLVTag {
    pub fn get_sound_format(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        (self.data[TAG_HEADER_BYTE_COUNT as usize + 0] >> 4) & 0x0f
    }

    pub fn get_sound_rate(&self) -> f64 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        match (self.data[TAG_HEADER_BYTE_COUNT as usize + 0] >> 2) & 0b11 {
            0 => SOUND_RATE_5K,
            1 => SOUND_RATE_11K,
            2 => SOUND_RATE_22K,
            3 => SOUND_RATE_44K,
            _ => panic!("get soundRate() a two-bit number wasn't 0, 1, 2, or 3. impossible.")
        }
    }

    /// frame duration = num of samples / sound_rate
    /// for aac, one frame always contains 1024 samples
    /// @return in milliseconds
    pub fn get_sound_frame_duration(&self, asc: &AudioSpecificConfig) -> f64 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        assert_eq!(self.get_sound_format(), SOUND_FORMAT_AAC);
        if self.is_acc_sequence_header() {
            return 0.;
        }
        return 1000. * 1024. / asc.get_sample_rate() as f64;
    }

    pub fn get_sound_size(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        if ((self.data[TAG_HEADER_BYTE_COUNT as usize + 0] >> 1) & 1) == 1 {
            SOUND_SIZE_16BITS
        } else {
            SOUND_SIZE_8BITS
        }
    }

    pub fn get_sound_channels(&self) -> u8 {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        if (self.data[TAG_HEADER_BYTE_COUNT as usize + 0] & 1) == 1 {
            SOUND_CHANNELS_STEREO
        } else {
            SOUND_CHANNELS_MONO
        }
    }

    pub fn get_sound_audio_specific_config(&self) -> AudioSpecificConfig {

        let body: &[u8] = self.get_sound_data();
        let original_audio_object_type: u8 = body[0] >> 3;
        let audio_object_type: u8 = original_audio_object_type;
        let sample_index: u8 = ((body[0] & 0x07) << 1) | (body[1] >> 7);
        let channel_config:u8 = (body[1] & 0x78) >> 3;
        
        AudioSpecificConfig {
            original_audio_object_type,
            audio_object_type,
            sample_index,
            channel_config,
        }
    }

    pub fn get_sound_adts_header_data(asc: &AudioSpecificConfig, frame_len: u32) -> [u8; 7] {

        let &AudioSpecificConfig { original_audio_object_type, audio_object_type, sample_index, channel_config } = asc;
        
        let mut header: [u8; 7] = [0; 7];
        header[0]  = 0xff;         //syncword:0xfff                          高8bits
        header[1]  = 0xf0;         //syncword:0xfff                          低4bits
        header[1] |= (0 << 3);    //MPEG Version:0 for MPEG-4,1 for MPEG-2  1bit
        header[1] |= (0 << 1);    //Layer:0                                 2bits
        header[1] |= 1;           //protection absent:1                     1bit

        header[2]  = (audio_object_type - 1) << 6; //profile:audio_object_type - 1                      2bits
        header[2] |= (sample_index & 0x0f) << 2;   //sampling frequency index:sampling_frequency_index  4bits
        header[2] |= (0 << 1);                     //private bit:0                                      1bit
        header[2] |= (channel_config & 0x04) >> 2; //channel configuration:channel_config               高1bit

        header[3]  = (channel_config & 0x03) << 6;  //channel configuration:channel_config      低2bits
        header[3] |= (0 << 5);                      //original：0                               1bit
        header[3] |= (0 << 4);                      //home：0                                   1bit
        header[3] |= (0 << 3);                      //copyright id bit：0                       1bit
        header[3] |= (0 << 2);                      //copyright id start：0                     1bit

        header[3] |= ((frame_len & 0x1800) >> 11) as u8;  //frame length：value   高2bits
        header[4] = ((frame_len & 0x7f8) >> 3) as u8;     //frame length:value    中间8bits 
        header[5] = ((frame_len & 0x7) << 5) as u8;       //frame length:value    低3bits
        header[5] |= 0x1f;                              //buffer fullness:0x7ff 高5bits 
        header[6] = 0xfc;
        return header;
    }

    pub fn get_sound_data(&self) -> &[u8] {
        self.data[(TAG_HEADER_BYTE_COUNT as usize + 2)..].as_ref()
    }

    pub fn get_sound_data_size(&self) -> u32 {
        self.get_data_size() - 2
    }

    pub fn is_acc_sequence_header(&self) -> bool {
        assert_eq!(self.get_tag_type(), FLVTagType::TAG_TYPE_AUDIO);
        assert_eq!(self.get_sound_format(), 10);
        self.data[TAG_HEADER_BYTE_COUNT as usize + 1] == 0
    }
}

pub struct FLVTagRead<'a, R: Read + 'a> {
    source: &'a mut R,
    pub header: FLVHeader,
    position: u64,
    finished: bool,
}

impl<'a, R: Read> FLVTagRead<'a, R> {
    pub fn new(r: &'a mut R) -> FLVTagRead<'a, R> {
        let header = FLVHeader::read(r);
        FLVTagRead::<'a, R> {
            source: r,
            header: header,
            finished: false,
            position: MIN_FILE_HEADER_BYTE_COUNT as u64 + 4
        }
    }

    pub fn get_position(&self) -> u64 {
        self.position
    }
}

impl<'a, R: Read + 'a> Iterator for FLVTagRead<'a, R> {
    type Item = FLVTag;

    fn next(&mut self) -> Option<FLVTag> {
        if self.finished {
            None
        }
        else {
            let tag = FLVTag::read(self.source);
            if tag.is_none() {
                self.finished = true;
            }
            else {
                self.position += tag.as_ref().unwrap().get_tag_size() as u64 + 4
            }
            tag
        }
    }
}

pub struct FLVTagWrite<W: Write + Seek> {
    stream: W,
    position: u64,
}

impl<W: Write + Seek> FLVTagWrite<W> {
    pub fn new(w: W) -> FLVTagWrite<W> {
        FLVTagWrite::<W> {
            stream: w,
            position: 0
        }
    }

    pub fn write_header(&mut self, header: &FLVHeader) {
        header.write(&mut self.stream);
        self.position += MIN_FILE_HEADER_BYTE_COUNT as u64 + 4;
    }

    pub fn write_tag(&mut self, tag: &FLVTag) {
        tag.write(&mut self.stream);
        // match tag.get_tag_type() {
        //     FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
        //         println!("write scriptdataobject {:?}", tag.get_objects());
        //     },
        //     _ => ()
        // }
        self.position += tag.get_tag_size() as u64 + 4
    }

    pub fn write_meta_tag(&mut self, tag: &FLVTag) {
        use std::io::SeekFrom::{Current, Start};

        let current_pos = self.stream.seek(Current(0)).unwrap();
        self.stream.seek(Start(MIN_FILE_HEADER_BYTE_COUNT as u64 + 4)).unwrap();
        self.write_tag(tag);
        // position fix
        let new_pos = self.stream.seek(Current(0)).unwrap();
        if current_pos > new_pos {
            self.stream.seek(Start(current_pos)).unwrap();
            self.position = current_pos;
        } else {
            self.position = new_pos;
        }
    }

    pub fn get_position(&self) -> u64 {
        self.position
    }
}

//按6分钟切割,计算分割点
//infos timestamp delta position
//return timestamp position keyframe_counts
pub fn split_flv_by_min(infos: &Vec<(u64, u64, u64)>, min: u64, win_seconds: u64) -> Vec<(u64, u64, u64, u64)> {
    let mut vec = Vec::<(u64, u64, u64, u64)>::new();
    let mut acc: usize = 1;
    let mut item_acc: i64 = -1;

    assert_eq!(infos[0].0, 0);
    assert_eq!(infos[1].0, 0);

    let f_time = infos[1].0;
    let f_pos = infos[1].2;
    vec.push((f_time, f_pos, 0, 0));

    for (i, &(t, dt, _)) in infos.iter().enumerate() {
        if t > (acc as u64) * min * 60 * 1000 {
            let mut current_delta = dt;
            let mut current_index = i;
            let mut j = i;
            while j < infos.len() && current_delta != 0 && infos[j].0 - t <= win_seconds * 1000 {
                if infos[j].1 < current_delta {
                    current_delta = infos[j].1;
                    current_index = j;
                }
                j += 1;
            }
            let (t, _, p) = infos[current_index];
            vec.push((t, p, 0, current_delta));
            vec[acc - 1].2 = item_acc as u64 + (current_index - i) as u64;
            acc += 1;
            item_acc = 0 - (current_index - i) as i64;
        }
        item_acc += 1;
    }
    vec[acc - 1].2 = item_acc as u64;

    let last_t = infos[infos.len() - 1].0;
    let last_result_t = vec[vec.len() - 1].0;
    if last_t - last_result_t < min * 60 * 1000 / 2 {
        let (_, _, n, _) = vec.pop().unwrap();
        vec[acc - 2].2 += n;
    }
    vec
}

macro_rules! write_xml {
    ($w:expr, $($rest: tt)*) => {{
        use std::borrow::Borrow;
        use xml::writer::{EmitterConfig, XmlEvent};

        let mut w1 = EmitterConfig::new().perform_indent(true).create_writer($w);
        _write_xml!(w1, $($rest)* );
    }}
}

macro_rules! _write_xml {
    ($w:expr, ) => (());
    ($w:expr, $e:tt) => {
        $w.write::<XmlEvent>(XmlEvent::from(format!("{}", $e).borrow()).into()).unwrap();
    };
    ($w:expr, format!($($e: expr),*)) => {
        $w.write::<XmlEvent>(XmlEvent::from(format!($($e),*).borrow()).into()).unwrap();
    };
    ($w:expr, $tag:ident { $($inner: tt)* } $($rest: tt)* ) => {
        $w.write::<XmlEvent>(XmlEvent::start_element(stringify!($tag)).into()).unwrap();
        _write_xml!($w, $($inner)*);
        $w.write::<XmlEvent>(XmlEvent::end_element().into()).unwrap();
        _write_xml!($w, $($rest)*);
    };
    ($w:expr, for $pat: pat in $expr: expr, { $($inner: tt)* } $($rest: tt)* ) => {
        for $pat in $expr {
            _write_xml!($w, $($inner)*);
        }
        _write_xml!($w, $($rest)*);
    }
}

pub fn write_flv_config<W: Write>(w: &mut W, info_vec: &Vec<(u64, u64)>, flvs: &Vec<String>, timelength: u64, url_prefix: &String) {
    write_xml!(w,
        video {
            timelength { timelength }
            for (path, &(t, s)) in flvs.iter().zip(info_vec.iter()), {
                durl {
                    length { t }
                    size { s }
                    url { format!("{}{}", url_prefix, path) }
                }
            }
        }
    );
}

#[test]
fn test_config() {
    use std::io::stdout;

    let info_vec = vec![(1, 1), (2, 2)];
    let flvs = vec!["1.flv".to_string(), "1.flv".to_string()];
    let timelength = 60;
    let url_prefix = "http://localhost/".to_string();

    let mut w = stdout();
    write_flv_config(&mut w, &info_vec, &flvs, timelength, &url_prefix);
}

#[test]
fn test_xml() {
    use std::io::stdout;

    let w = stdout();
    write_xml!(w,
        video {
            timelength { 1000 }
            for i in 0..2, {
                durl {
                    id { i }
                    length { 20 }
                    size { 30 }
                    url { "http://localhost/a.flv" }
                }
            }
        }
    );
}
