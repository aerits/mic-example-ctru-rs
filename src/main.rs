#![feature(allocator_api, slice_ptr_get)]
/// Unsafe microphone example for libctru 
/// https://github.com/devkitPro/3ds-examples/tree/master/audio/mic mostly a 1-to-1 copy of this
use ctru::{linear::LinearAllocator, prelude::*};
use ctru_sys::{
    CSND_SetPlayState, CSND_UpdateInfo, GSPGPU_FlushDataCache, MICU_ENCODING_PCM16_SIGNED,
    MICU_SAMPLE_RATE_16360, MICU_StartSampling, MICU_StopSampling, R_FAILED, R_SUCCEEDED,
    SOUND_FORMAT_16BIT, SOUND_ONE_SHOT, csndExit, csndInit, csndPlaySound, micExit,
    micGetLastSampleOffset, micGetSampleDataSize, micInit,
};
use mic::MicCTRU;
use std::{
    alloc::{Allocator, Layout, alloc, dealloc},
    ptr::{NonNull, null, null_mut},
};
mod mic;
fn main() {
    let gfx = Gfx::new().expect("Couldn't obtain GFX controller");
    let mut hid = Hid::new().expect("Couldn't obtain HID controller");
    let apt = Apt::new().expect("Couldn't obtain APT controller");
    let _console = Console::new(gfx.top_screen.borrow_mut());

    let mut initialized = true;
    let mut mic = MicCTRU::new(0x300000).unwrap();
    let mut micbuf_pos: usize = 0;
    unsafe {
        println!("initializing CSND...");
        if R_FAILED(csndInit()) {
            println!("failed to initialize csnd");
            initialized = false;
        }
    };
    const audiobuf_size: usize = 0x100000;
    let mut audiobuf_pos = 0;
    // LinearAllocator in ctru-rs crashes the 3ds when I try to box it, so I'm just using it raw
    let audiobuf = LinearAllocator
        .allocate_zeroed(Layout::new::<[u8; audiobuf_size]>())
        .unwrap()
        .as_mut_ptr();

    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        if !initialized {
            continue;
        }
        if hid.keys_down().contains(KeyPad::A) {
            audiobuf_pos = 0;
            micbuf_pos = 0;
            println!("Stopping audio playback");
            unsafe {
                CSND_SetPlayState(0x8, 0);
                if R_FAILED(CSND_UpdateInfo(false)) {
                    println!("failed to stop audio playback")
                }
                println!("Starting sampling");
                
            };
            mic.start_recording().unwrap();
        }
        if hid.keys_held().contains(KeyPad::A) {
            let mut micbuf_readpos = micbuf_pos;
            micbuf_pos = unsafe { micGetLastSampleOffset() as usize };
            while audiobuf_pos < audiobuf_size && micbuf_readpos != micbuf_pos {
                unsafe {
                    *audiobuf.offset(audiobuf_pos as isize) =
                        *mic.micbuf.offset(micbuf_readpos as isize);
                    audiobuf_pos += 1;
                    micbuf_readpos = (micbuf_readpos + 1) % mic.micbuf_datasize as usize;
                }
            }
        }
        if hid.keys_up().contains(KeyPad::A) {
            println!("Stopping sampling");
            mic.stop_recording().unwrap();
            println!("starting audio playback");
            if unsafe {
                R_SUCCEEDED(GSPGPU_FlushDataCache(audiobuf.cast(), audiobuf_size as u32))
                    && R_SUCCEEDED(csndPlaySound(
                        0x8,
                        (SOUND_ONE_SHOT | SOUND_FORMAT_16BIT).into(),
                        16360,
                        1.0,
                        0.0,
                        audiobuf.cast(),
                        null_mut(),
                        audiobuf_pos as u32,
                    ))
            } {
                println!("now playing");
            } else {
                println!("failed to start playback");
            }
        }
        gfx.wait_for_vblank();
    }
    unsafe {
        
        LinearAllocator.deallocate(
            NonNull::new_unchecked(audiobuf),
            Layout::new::<[u8; audiobuf_size]>(),
        );
        csndExit();
    }
}
