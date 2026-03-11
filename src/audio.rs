// Copyright 2026 Piotr Grygorczuk <grygorek@gmail.com>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::io::Cursor;

pub struct AudioController {
    // This must stay alive for the duration of the app to keep the sound device active
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioController {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().expect("Failed to find an audio output device");

        Self { _stream: stream, handle }
    }

    /// Plays a sound from raw bytes (embedded wav/mp3 files)
    pub fn play_sfx(&self, data: &'static [u8]) {
        // Create a cursor for the static bytes
        let cursor = Cursor::new(data);

        // Decode the source
        match Decoder::new(cursor) {
            Ok(source) => {
                // Play the sound on the stream handle
                let _ = self.handle.play_raw(source.convert_samples());
            }
            Err(e) => {
                eprintln!("Error decoding sound effect: {}", e);
            }
        }
    }
}
