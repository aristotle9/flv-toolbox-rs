extern crate rustc_serialize;
extern crate byteorder;

use std::collections::BTreeMap;
use std::io::{Read, Write, Cursor};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use rustc_serialize::json::Json;

fn read_number(r: &mut Read) -> Json {
    Json::F64(r.read_f64::<BigEndian>().unwrap())
}

fn read_boolean(r: &mut Read) -> Json {
    Json::Boolean(r.read_u8().unwrap() != 0)
}

fn read_string(r: &mut Read) -> Json {
    let len = r.read_u16::<BigEndian>().unwrap();
    Json::String(read_raw_string(r, len as usize))
}

fn read_raw_string(r: &mut Read, len: usize) -> String {
    let mut buffer: Vec<u8> = Vec::with_capacity(len);
    let mut handle = r.take(len as u64);
    let read_len = handle.read_to_end(&mut buffer).unwrap();
    assert_eq!(len, read_len);
    String::from_utf8(buffer).unwrap()
}

fn read_ecma_array(r: &mut Read) -> Json {
    let count = r.read_u32::<BigEndian>().unwrap();
    read_object(r)
}

fn read_strict_array(r: &mut Read) -> Json {
    let count = r.read_u32::<BigEndian>().unwrap();
    let mut v: Vec<Json> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        v.push(read_value(r));
    }
    Json::Array(v)
}

fn read_object(r: &mut Read) -> Json {
    let mut obj = BTreeMap::new();
    loop {
        let len = r.read_u16::<BigEndian>().unwrap() as usize;
        if len == 0 {
            let end_mark = r.read_u8().unwrap();
            assert_eq!(0x09, end_mark);
            break;
        }
        else {
            let key = read_raw_string(r, len);
            let val = read_value(r);
            obj.insert(key, val);
        }
    }
    Json::Object(obj)
}

fn read_value(r: &mut Read) -> Json {
    match r.read_u8().unwrap() {
        0x00 => read_number(r),
        0x01 => read_boolean(r),
        0x02 => read_string(r),
        0x03 => read_object(r),
        0x05 => Json::Null,
        0x06 => Json::Null,
        0x08 => read_ecma_array(r),
        0x0A => read_strict_array(r),
        n => panic!(format!("unsupported mark {}", n))
    }
}

#[test]
fn it_works() {
    use std::io::Cursor;
    use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

    return;
    let mut rdr = Cursor::new(vec![2, 5, 3, 0]);

    assert_eq!(517, rdr.read_u16::<BigEndian>().unwrap());
    assert_eq!(768, rdr.read_u16::<BigEndian>().unwrap());
    println!("{:016b}", 517);
    println!("{:016b}", 768);

    //println!("Hello, It works!");
}

fn read_u24_be(r: &mut Read) -> u32 {
    let (b1, b2, b3) = (r.read_u8().unwrap() as u32, r.read_u8().unwrap() as u32, r.read_u8().unwrap() as u32);
    b1 << 16 | b2 << 8 | b3
}

fn write_u24_be(w: &mut Write, n: u32) {
    w.write_u8(((n >> 16) & 0xff) as u8);
    w.write_u8(((n >> 8 ) & 0xff) as u8);
    w.write_u8(((n      ) & 0xff) as u8);
}

const TAG_HEADER_BYTE_COUNT:usize = 11;
const PREV_TAG_BYTE_COUNT:usize = 4;

#[derive(Debug, PartialEq)]
enum FLVTagType {
    TAG_TYPE_AUDIO = 8,
    TAG_TYPE_VIDEO = 9,
    TAG_TYPE_SCRIPTDATAOBJECT = 18,
}

impl From<u8> for FLVTagType {
    fn from(t: u8) -> FLVTagType {
        let v: Vec<u8> = vec![8, 9, 18];
        if !v.contains(&t) {
            panic!(format!("unknown tagType: {}", t));
        }
        unsafe {
            std::mem::transmute(t)
        }
    }
}

struct FLVHeader {
    hasAudioTags: bool,
    hasVideoTags: bool,
}

impl FLVHeader {
    fn read(r: &mut Read) -> FLVHeader {
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

    fn write(&self, w: &mut Write) {
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
        w.write_u32::<BigEndian>(9);
        w.write_u32::<BigEndian>(0);
    }
}

struct FLVTag {
    data: Vec<u8>
}

impl FLVTag {
    fn getTagType(&self) -> FLVTagType {
        FLVTagType::from(self.data[0])
    }

    fn getTagSize(&self) -> u32 {
        11 + self.getDataSize() + 4
    }

    fn getDataSize(&self) -> u32 {
        ((self.data[1] as u32) << 16) | ((self.data[2] as u32) << 8) | (self.data[3] as u32)
    }

    fn getTimestamp(&self) -> u64 {
        ((self.data[7] as u64) << 24) | ((self.data[4] as u64) << 16) | ((self.data[5] as u64) << 8) | (self.data[6] as u64)
    }

    fn read(r: &mut Read) -> Option<FLVTag>{
        let tagType = match r.read_u8() {
            Ok(n) => n,
            Err(..) => return None
        };
        let dataSize = read_u24_be(r) as usize;
        let tagSize = 1 + 3 + 3 + 1 + 3 + dataSize + 4;
        let mut payload: Vec<u8> = Vec::with_capacity(tagSize);

        payload.write_u8(tagType);
        write_u24_be(&mut payload, dataSize as u32);

        let mut handle = r.take((tagSize - 1 - 3) as u64);
        let read_len = handle.read_to_end(&mut payload).unwrap();
        assert_eq!((tagSize - 4) as usize, read_len);

        Some(FLVTag {
            data: payload
        })
    }

    fn write(&self, w: &mut Write) {
        w.write(&self.data);
    }
}

impl FLVTag {
    fn getObjects(&mut self) -> Json {
        assert_eq!(self.getTagType(), FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT);
        let mut v: Vec<Json> = Vec::with_capacity(2);
        let mut handle = Cursor::new(&self.data[TAG_HEADER_BYTE_COUNT..]);
        v.push(read_value(&mut handle));
        v.push(read_value(&mut handle));
        Json::Array(v)
    }
}

impl FLVTag {
    fn getFrameType(&self) -> u8 {
        (self.data[TAG_HEADER_BYTE_COUNT + 0] >> 4) & 0x0f
    }

    fn getCodecID(&self) -> u8 {
        (self.data[TAG_HEADER_BYTE_COUNT + 0] & 0x0f)
    }

    fn getAvcPacketType(&self) -> u8 {
        self.data[TAG_HEADER_BYTE_COUNT + 1]
    }
}

#[test]
fn flv_parse() {
    use std::fs::File;
    use std::path::Path;
    use std::io::SeekFrom;
    use std::io::Seek;
    use rustc_serialize::json::as_pretty_json;

    let path = Path::new("/Users/lanfan/projects/as3-projects/videos/youku-1/0300010800561D2AD49F851468DEFEA585825F-9542-DC16-3713-AC06678EC8EB.flv");
    let mut file = File::open(path).unwrap();
    FLVHeader::read(&mut file);
    loop {
        let tag = FLVTag::read(&mut file);
        if tag.is_none() {
            break;
        }
        let mut tag = tag.unwrap();
        match tag.getTagType() {
            FLVTagType::TAG_TYPE_VIDEO => {
                // println!("{:?}", (tag.getFrameType(), tag.getCodecID(), tag.getAvcPacketType()));
            },
            FLVTagType::TAG_TYPE_SCRIPTDATAOBJECT => {
                println!("{}", as_pretty_json(&tag.getObjects()));
                break;
            }
            _ => ()
        };
        // println!("{:?}", (tag.getTagType(), tag.getDataSize(), tag.getTimestamp()));
    }
}
