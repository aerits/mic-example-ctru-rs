#![feature(allocator_api, slice_ptr_get)]
/// Unsafe microphone example for libctru
/// https://github.com/devkitPro/3ds-examples/tree/master/audio/mic mostly a 1-to-1 copy of this
use ctru::{
    linear::LinearAllocator,
    prelude::*,
    services::ndsp::{AudioFormat, AudioMix, InterpolationType, Ndsp, OutputMode, wave::Wave},
};
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
    let mut mic = MicCTRU::new(0x30000).unwrap();
    let mut micbuf_pos: usize = 0;
    println!("Initializing mic");

    let mut ndsp = Ndsp::new().expect("Couldn't obtain NDSP controller");
    println!("initializing ndsp");
    ndsp.set_output_mode(OutputMode::Mono);
    let mut channel_zero = ndsp.channel(0).unwrap();
    channel_zero.set_interpolation(InterpolationType::Linear);
    channel_zero.set_sample_rate(16360 as f32);
    channel_zero.set_format(AudioFormat::PCM16Mono);
    let mix = AudioMix::default();
    channel_zero.set_mix(&mix);

    // wave needs to stay alive
    let mut wave_info1;

    while apt.main_loop() {
        hid.scan_input();
        if hid.keys_down().contains(KeyPad::START) {
            break;
        }
        if !initialized {
            continue;
        }
        if hid.keys_down().contains(KeyPad::A) {
            // audiobuf_pos = 0;
            // micbuf_pos = 0;
            println!("Stopping audio playback");
            channel_zero.clear_queue();
            mic.start_recording().unwrap();
        }
        if hid.keys_held().contains(KeyPad::A) {
            // let mut micbuf_readpos = micbuf_pos;
            // micbuf_pos = unsafe { micGetLastSampleOffset() as usize };
            // while audiobuf_pos < audiobuf_size && micbuf_readpos != micbuf_pos {
            //     unsafe {
            //         *audiobuf.offset(audiobuf_pos as isize) =
            //             *mic.micbuf.offset(micbuf_readpos as isize);
            //         audiobuf_pos += 1;
            //         micbuf_readpos = (micbuf_readpos + 1) % mic.micbuf_datasize as usize;
            //     }
            // }
        }
        if hid.keys_up().contains(KeyPad::A) {
            println!("Stopping sampling");
            mic.stop_recording().unwrap();
            println!("starting audio playback");
            let mic_data = mic.get_mic_buf();
            // Box::new_in(...) doesn't work for me
            let heap_data = LinearAllocator
                .allocate_zeroed(Layout::from_size_align(mic_data.len(), 1).unwrap())
                .unwrap();
            let mut audio_data1 = unsafe { Box::from_non_null_in(heap_data, LinearAllocator) };

            audio_data1.copy_from_slice(mic_data);
            wave_info1 = Wave::new(audio_data1, AudioFormat::PCM16Mono, false);
            if let Err(e) = channel_zero.queue_wave(&mut wave_info1) {
                println!("Failed to start playback")
            } else {
                println!("Now Playing");
            }
        }
        gfx.wait_for_vblank();
    }
    // unsafe {

    //     LinearAllocator.deallocate(
    //         NonNull::new_unchecked(audiobuf),
    //         Layout::new::<[u8; audiobuf_size]>(),
    //     );
    //     csndExit();
    // }
}
