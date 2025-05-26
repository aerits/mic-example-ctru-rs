use std::alloc::{Layout, alloc, dealloc};

use ctru_sys::{
    micExit, micGetLastSampleOffset, micGetSampleDataSize, micInit, MICU_StartSampling, MICU_StopSampling, MICU_ENCODING_PCM16, MICU_ENCODING_PCM16_SIGNED, MICU_SAMPLE_RATE_16360, R_FAILED, R_SUCCEEDED
};

pub struct MicCTRU {
    micbuf_size: usize,
    micbuf_layout: Layout,
    pub micbuf_datasize: u32,
    pub micbuf: *mut u8,
}
impl MicCTRU {
    pub fn new(size: usize) -> Result<MicCTRU, MicError> {
        let micbuf_size: usize = size;

        // not sure why the original example had this alignment, I'm not a low level programming pro, I just copied it.
        let micbuf_layout = Layout::from_size_align(micbuf_size, 0x1000).expect("Invalid layout");
        // allocating a mic buffer on the global allocator
        let micbuf = unsafe { alloc(micbuf_layout) };
        if micbuf.is_null() {
            return Err(MicError::new("Could not allocate micbuf".into()));
        }
        let micbuf_datasize = unsafe {
            if R_FAILED(micInit(micbuf, micbuf_size as u32)) {
                return Err(MicError::new("Could not initialize mic".into()));
            }
            micGetSampleDataSize()
        };
        return Ok(MicCTRU {
            micbuf_size,
            micbuf_layout,
            micbuf_datasize,
            micbuf,
        });
    }
    pub fn start_recording(&mut self) -> Result<(), MicError> {
        if R_SUCCEEDED(unsafe {
            MICU_StartSampling(
                MICU_ENCODING_PCM16_SIGNED,
                MICU_SAMPLE_RATE_16360,
                0,
                self.micbuf_datasize,
                true,
            )
        }) {
            return Ok(());
        } else {
            return Err(MicError::new("Failed to start sampling".into()));
        }
    }
    // The reference should be safe
    pub fn get_mic_buf(&self) -> &[u8] {
        return unsafe { std::slice::from_raw_parts(self.micbuf, micGetLastSampleOffset() as usize) };
    }
    pub fn stop_recording(&mut self) -> Result<(), MicError> {
        if R_FAILED(unsafe { MICU_StopSampling() }) {
            return Err(MicError::new("Failed to stop recording".into()));
        }
        return Ok(())
    }
}
impl Drop for MicCTRU {
    fn drop(&mut self) {
        unsafe {
            micExit();
            dealloc(self.micbuf, self.micbuf_layout);
        }
    }
}

#[derive(Debug)]
pub struct MicError {
    pub error: String,
}
impl MicError {
    fn new(error: String) -> MicError {
        return MicError { error: error };
    }
}
