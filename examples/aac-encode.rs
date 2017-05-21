extern crate ffmpeg;
extern crate flv_toolbox_rs;
extern crate libc;
extern crate byteorder;
extern crate rand;

use libc::c_int;
use rand::Rng;
use ffmpeg::codec::profile::{AAC, Profile};
use ffmpeg::util::channel_layout::*;
use flv_toolbox_rs::utils::PrettyHex;
use byteorder::{BigEndian, WriteBytesExt};
use std::io::{ Cursor };

static rate_list: [u32; 13] = [96000, 88200, 64000, 48000, 44100, 32000,
                        24000, 22050, 16000, 12000, 11025, 8000, 7350];
/**
 *  Add ADTS header at the beginning of each and every AAC packet.
 *  This is needed as MediaCodec encoder generates a packet of raw
 *  AAC data.
 *
 *  Note the packet_len must count in the ADTS header itself.
 **/
fn adts(rate: u32, profile: &Profile, channels: u16, layout: ChannelLayout, data_len: usize) {

    println!("{:?}", (rate, profile, channels, layout, 1024, data_len));

    let index = rate_list.iter().position(|x| *x == rate).unwrap() as u8;
    let profile: c_int = profile.clone().into();
    let mut packet: [u8; 7] = [0; 7];
    let packet_len = data_len + 7;
    // fill in ADTS data
    packet[0] = 0xFF;
    packet[1] = 0xF9;
    packet[2] = (((profile-1)<<6) as u8 + (index << 2) + (channels >> 2) as u8);
    packet[3] = (((channels & 3) << 6) as u8 + (packet_len>>11) as u8);
    packet[4] = ((packet_len&0x7FF) >> 3) as u8;
    packet[5] = (((packet_len&7)<<5) + 0x1F) as u8;
    packet[6] = 0xFC;

    print!("{}", PrettyHex::new(&packet).with_opts((false, false, false)));
}

fn gen_zeros_frames(rate: u32, channel_layout: ChannelLayout) {

    let codec = ffmpeg::codec::encoder::find(ffmpeg::codec::id::Id::AAC).unwrap();
    let context = ffmpeg::codec::Context::new();
    let mut encoder = context.encoder().audio().unwrap();

    let sample_format = ffmpeg::format::sample::Sample::F32(ffmpeg::format::sample::Type::Planar);

// 0: Defined in AOT Specifc Config
// 1: 1 channel: front-center
// 2: 2 channels: front-left, front-right
// 3: 3 channels: front-center, front-left, front-right
// 4: 4 channels: front-center, front-left, front-right, back-center
// 5: 5 channels: front-center, front-left, front-right, back-left, back-right
// 6: 6 channels: front-center, front-left, front-right, back-left, back-right, LFE-channel
// 7: 8 channels: front-center, front-left, front-right, side-left, side-right, back-left, back-right, LFE-channel
// 8-15: Reserved

    // channel layout 隐含 channels，所以不用再额外设置 channels
    let profile = Profile::AAC(AAC::Main);
    // let rate: u32 = 44100;

    encoder.set_bit_rate(12288);
    encoder.set_format(sample_format);
    encoder.set_rate(rate as i32);
    // encoder.set_channels(2);
    encoder.set_time_base((1, rate as i32));
    encoder.set_channel_layout(channel_layout);
    unsafe {
        (*encoder.as_mut_ptr()).profile = profile.clone().into();
    }
    // encoder.set_parameters()

    let mut output = ffmpeg::codec::packet::Packet::empty();
    let mut frame = ffmpeg::util::frame::audio::Audio::new(sample_format, 1024, channel_layout);
    // println!("{:?}", (frame.planes(), frame.data(0).len()));

    {
        for index in 0..(encoder.channels() as usize) {
            let mut write = Cursor::new(frame.data_mut(index));
            // let mut rng = rand::thread_rng();
            for i in 0..1024 {
                // write.write_f32::<BigEndian>(0.0_f32);
                // write.write_f32::<BigEndian>((rng.gen::<i32>() % (rate as i32 * 2) ) as f32 - rate as f32 * 1f32);
            }
        }
        // {
        //     let mut write = Cursor::new(frame.data_mut(1));
        //     let mut rng = rand::thread_rng();
        //     for i in 0..1024 {
        //         write.write_f32::<BigEndian>((rng.gen::<i32>() % (rate as i32 * 2) ) as f32 - rate as f32 * 1f32);
        //     }
        // }
    }
    // println!("{}", PrettyHex::new(frame.data(0)));

    let mut encoder = encoder.open_as(codec).unwrap();
    let mut result: Result<bool, ffmpeg::Error> = Ok(false);

    for _ in 0..2 {
        output = ffmpeg::codec::packet::Packet::empty();
        result = encoder.encode(&frame, &mut output);
        // println!("{:?}", result);

        if *result.as_ref().unwrap() {
            // adts(encoder.rate(), &profile, frame.channels(), encoder.channel_layout(), output.data().as_ref().unwrap().len());
            // println!("{}", PrettyHex::new(output.data().unwrap()).with_opts((false, false, false)));
        }
    }
    
    for _ in 0..2 {
        output = ffmpeg::codec::packet::Packet::empty();
        result = encoder.flush(&mut output);
        if *result.as_ref().unwrap() {
            // println!("flush:");
            // adts(encoder.rate(), &profile, frame.channels(), encoder.channel_layout(), output.data().as_ref().unwrap().len());
            // println!("{}", PrettyHex::new(output.data().unwrap()).with_opts((false, false, false)));
        }
    }
    
    adts(encoder.rate(), &profile, frame.channels(), encoder.channel_layout(), output.data().as_ref().unwrap().len());
    println!("{}", PrettyHex::new(output.data().unwrap()).with_opts((false, false, false)));
}

fn main() {

    ffmpeg::init().unwrap();

    let rate_list0: [u32; 2] = [22050, 44100];
    let layouts: [ChannelLayout; 7] = [MONO, STEREO, SURROUND, QUAD, _5POINT0, _5POINT1, HEXAGONAL];
    
    for &rate in rate_list0.iter() {
        for &layout in layouts.iter() {
            gen_zeros_frames(rate, layout);
        }
    }
}