extern crate core;

use hound;
use libmp3lame_sys::*;
use num_rational::Rational64;
use std::ffi::CString;
use std::fs::File;
use std::i16;
use std::io::prelude::*;
use std::io::{self};
use std::os::raw::c_char;
use std::ptr::NonNull;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub const SAMPLE_RATE: u32 = 48_000;

#[derive(Clone)]
pub struct RawFrame {
    pub timestamp: Rational64,
    pub data: Vec<i16>,
}

impl RawFrame {
    fn duration(&self, num_channels: usize) -> Rational64 {
        Rational64::new((self.data.len() / num_channels) as i64, 48_000)
    }
}

pub struct Encoder {
    lame: NonNull<lame_global_flags>,
    data: Vec<u8>,
}

#[no_mangle]
pub extern "C" fn encode(data: *mut u8, length: u32) -> *const c_char {
    unsafe {
        let buf: &mut [u8] = core::slice::from_raw_parts_mut(data, length as usize);
        let mut ptr = io::Cursor::new(buf);
        let mut wav_reader = hound::WavReader::new(&mut ptr).unwrap();
        let mut data = Vec::new();
        for sample in wav_reader.samples() {
            match sample {
                Ok(smpl) => data.push(smpl),
                _ => panic!("failed"),
            }
        }
        let timestamp = Rational64::from_integer(0);
        let raw_frame = RawFrame { timestamp, data };
        let mut encoder = Encoder::new();
        let output = encoder.encode(raw_frame);
        return CString::new(output)
            .expect("failed to encode mp3")
            .into_raw();
    }
}

impl Encoder {
    fn new() -> Encoder {
        let lame = unsafe {
            let lame = NonNull::new(lame_init()).expect("Failed to allocate lame global flags");

            lame_set_in_samplerate(lame.as_ptr(), SAMPLE_RATE as i32);
            lame_set_brate(lame.as_ptr(), 128);
            lame_set_quality(lame.as_ptr(), 2); // 2=high 5=medium 7=low
            lame_set_num_channels(lame.as_ptr(), 2);
            lame_set_write_id3tag_automatic(lame.as_ptr(), 0);

            let ret = lame_init_params(lame.as_ptr());
            if ret < 0 {
                panic!("failed to lame_init_params(): {}", ret);
            }

            lame
        };

        Encoder {
            lame,
            data: Vec::new(),
        }
    }

    fn encode(&mut self, mut raw_frame: RawFrame) -> String {
        let estimated_encoded_bytes = (raw_frame.duration(2) * SAMPLE_RATE as i64 * 5 / 4 + 7200)
            .ceil()
            .to_integer() as usize;
        let mut encoded: Vec<u8> = Vec::new();
        encoded.resize(estimated_encoded_bytes, 0);
        let encoded_bytes = unsafe {
            lame_encode_buffer_interleaved(
                self.lame.as_ptr(),
                raw_frame.data.as_mut_ptr(),
                raw_frame.data.len() as i32 / 2,
                encoded.as_mut_ptr(),
                encoded.len() as i32,
            )
        };
        if encoded_bytes < 0 {
            panic!(
                "Failed to lame_encode_buffer_interleaved: error={}",
                encoded_bytes
            );
        }
        encoded.resize(encoded_bytes as usize, 0);
        self.data.append(&mut encoded);
        let data = self.data.clone();
        let now = SystemTime::now();
        let unixtime = now.duration_since(UNIX_EPOCH).expect("back to the future");
        let timestamp = unixtime.as_secs();
        let tempfile = format!("{}.mp3", timestamp);
        File::create(tempfile.clone())
            .expect("oops")
            .write_all(&data)
            .expect("oops");

        return tempfile;
    }
}

impl Drop for Encoder {
    fn drop(&mut self) {
        unsafe { lame_close(self.lame.as_ptr()) };
    }
}
