extern crate flv_toolbox_rs;

use std::mem::size_of;
use flv_toolbox_rs::faad::*;
use std::fs::File;
use std::io::Read;

fn main() {
    unsafe {
        let cap = NeAACDecGetCapabilities();
        let h_aac = NeAACDecOpen();
        let conf = NeAACDecGetCurrentConfiguration(h_aac);
        NeAACDecSetConfiguration(h_aac, conf);

        let mut sample_rate: ::std::os::raw::c_ulong = 0;
        let mut channels: ::std::os::raw::c_uchar = 0;

        let mut file = File::open("./audio-b.aac").unwrap();
        let mut buf = [0; 4096];
        file.read(&mut buf);
        // println!("{:?}", buf.as_ref());
        let mut data: Vec<u8> =vec![
                255, 241, 92, 128, 2, 31, 252,
                0x21, 0x00, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80,
                255, 241, 92, 128, 2, 31, 252,
                0x21, 0x00, 0x49, 0x90, 0x02, 0x19, 0x00, 0x23, 0x80,
                255, 241, 92, 128, 5, 191, 252,
                33, 25, 211, 64, 125, 11, 109, 68, 174, 129, 8, 0, 137, 160, 62, 133, 182, 146, 87, 4, 128, 0, 91, 183, 120, 0, 132, 0, 0, 0, 0, 0, 56, 48, 0, 6, 0, 56,
                255, 241, 92, 128, 5, 191, 252,
                33, 25, 211, 64, 125, 11, 109, 68, 174, 129, 8, 0, 137, 160, 62, 133, 182, 146, 87, 4, 128, 0, 91, 183, 120, 0, 132, 0, 0, 0, 0, 0, 56, 48, 0, 6, 0, 56,
                255, 241, 92, 128, 5, 191, 252,
                33, 25, 211, 64, 125, 11, 109, 68, 174, 129, 8, 0, 137, 160, 62, 133, 182, 146, 87, 4, 128, 0, 91, 183, 120, 0, 132, 0, 0, 0, 0, 0, 56, 48, 0, 6, 0, 56,
                255, 241, 92, 128, 5, 191, 252,
                33, 25, 211, 64, 125, 11, 109, 68, 174, 129, 8, 0, 137, 160, 62, 133, 182, 146, 87, 4, 128, 0, 91, 183, 120, 0, 132, 0, 0, 0, 0, 0, 56, 48, 0, 6, 0, 56,
        ];
        let mut info_raw: Vec<u8> = vec![0; size_of::<NeAACDecFrameInfo>()];
        let mut buf_p = data.as_mut_ptr();
        let mut buf_len = data.len() as ::std::os::raw::c_ulong;
        // let mut buf_p = buf.as_mut_ptr();
        // let mut buf_len = buf.len() as ::std::os::raw::c_ulong;
        // println!("{:?}", &data);
        let err = NeAACDecInit(h_aac, buf_p, buf_len, &mut sample_rate, &mut channels);
        println!("{:?}", (err, sample_rate, channels));
        buf_len -= err as u64;
        buf_p = buf_p.offset(err as isize);

        loop {
            let mut output = NeAACDecDecode(h_aac, info_raw.as_mut_ptr() as *mut _, buf_p, buf_len);
            let mut info: Box<NeAACDecFrameInfo> = Box::from_raw(info_raw.as_mut_ptr() as *mut _);
            println!("{:?}", (info.samples, info.channels, info.samplerate, info.error, info.bytesconsumed, buf_len));
            if info.error > 0 || info.bytesconsumed == 0 {
                break;
            }
            buf_p = buf_p.offset(info.bytesconsumed as isize);
            buf_len -= info.bytesconsumed;

            Box::into_raw(info);
        }
    }
}