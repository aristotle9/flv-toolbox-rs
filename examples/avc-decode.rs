extern crate ffmpeg;
extern crate flv_toolbox_rs;

use std::fs::File;
use std::io::{ Read, Write };

use ffmpeg::ffi::*;

use flv_toolbox_rs::lib::{ FLVTagRead, FLVTagType };

fn main() {

    println!("{:X}", ffmpeg::codec::version());

    ffmpeg::init().unwrap();

    let codec = ffmpeg::codec::decoder::find(ffmpeg::codec::id::Id::H264).unwrap();
    let mut context = ffmpeg::codec::Context::new();
    let opened = context.decoder().open_as(codec).unwrap();
    let mut dc = opened.video().unwrap();

    let mut file = File::open("videos/10167761-16799270/16799270-1.flv").unwrap();
    let mut parser = FLVTagRead::new(&mut file);
    let spliter: [u8; 4] = [0, 0, 0, 1];
    let mut count = 0;
    let mut buf: Vec<u8> = Vec::new();
    let mut avcc_data: Option<Vec<u8>> = None;
    
    loop {
        let nxt = parser.next();
        if nxt.is_none() {
            break;
        }
        let tag = nxt.unwrap();
        if tag.get_tag_type() == FLVTagType::TAG_TYPE_VIDEO {
            
            if tag.get_frame_type() == 1 && tag.get_avc_packet_type() == 0 {
                let avcc = tag.get_avcc();
                buf.write(&spliter);
                buf.write(&avcc.sps);
                buf.write(&spliter);
                buf.write(&avcc.pps_array[0]);
                avcc_data = Some(tag.get_avcc_data().to_vec());

                // println!("{:?}", avcc);
            } else {
                let nalus = tag.get_nal_units();
                for nalu in nalus {
                    buf.write(&spliter);
                    buf.write(&nalu);
                    // println!("{:?}", (nalu.len(), &nalu[0..5]));
                }
            }

            count += 1;
            if count > 1 {
                {
                    let packet = ffmpeg::codec::packet::Packet::borrow(&buf);
                    let mut frame = ffmpeg::util::frame::Video::empty();
                    // frame.set_format(ffmpeg::util::format::Pixel::YUV420P);
                    let result = dc.decode(&packet, &mut frame).unwrap();
                    println!("{:?}", result);
                    println!("{:?}", (frame.width(), frame.height(), frame.planes(), frame.format()));
                    if result { // save picture
                        // let mut scale_ctx = ffmpeg::software::scaling::Context::get(dc.format(), dc.width(), dc.height(), ffmpeg::util::format::Pixel::RGB24, dc.width(), dc.height(), ffmpeg::software::scaling::flag::FAST_BILINEAR).unwrap();
                        let mut pic_frame = ffmpeg::util::frame::Video::new(ffmpeg::util::format::Pixel::RGB32, dc.width(), dc.height());
                        unsafe {
                            let err = avpicture_alloc(pic_frame.as_mut_ptr() as *mut _, ffmpeg::util::format::Pixel::RGB32.into(), dc.width() as i32, dc.height() as i32);
                            if err > 0 {
                                panic!("avpicture_alloc err");
                            }
                        }
                        let mut scale_ctx = frame.converter(ffmpeg::util::format::Pixel::RGB32).unwrap();
                        println!("{:?}", (scale_ctx.output(), pic_frame.format(), pic_frame.width(), pic_frame.height()));
                        // println!("{:?}", dc.time_base());
                        scale_ctx.run(&frame, &mut pic_frame).unwrap();
                        let png_encode = ffmpeg::codec::encoder::find(ffmpeg::codec::id::Id::PNG).unwrap();
                        let png_encode_ctx = ffmpeg::codec::context::Context::new();
                        let mut png_encode_ctx_1 = png_encode_ctx.encoder().video().unwrap();
                        png_encode_ctx_1.set_width(dc.width());
                        png_encode_ctx_1.set_height(dc.height());
                        png_encode_ctx_1.set_format(dc.format());
                        png_encode_ctx_1.set_time_base(dc.time_base());
                        let mut png_opened = png_encode_ctx_1.open_as(png_encode).unwrap();
                        // println!("{:?}", dc.time_base());
                        let mut packet = ffmpeg::codec::packet::Packet::empty();
                        let result = png_opened.encode(&pic_frame, &mut packet).unwrap();
                        println!("encode png: {}", result);
                    }
                }
                buf = Vec::new();
                if count > 6 {
                    break;
                }
            }
        }
    }
}