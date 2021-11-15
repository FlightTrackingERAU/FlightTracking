use std::mem::MaybeUninit;

pub struct StringFormatter<const N: usize> {
    buf: [u8; N],
    index: usize,
}

impl<const N: usize> Default for StringFormatter<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> StringFormatter<N> {
    pub fn new() -> Self {
        let buf = MaybeUninit::uninit();
        Self {
            // # Safety:
            // 
            // We never read up to self.index bytes out of buf, which are guaranteed to be written
            // to before reading. These are simply bytes which can have any value
            buf: unsafe { buf.assume_init() },
            index: 0,
        }
    }

    pub fn clear(&mut self) {
        self.index = 0;
    }

    pub fn as_str(&self) -> &str {
        // # Safety
        //
        // 1. `self.buf` is guaranteed to be valid for writes in range `0..self.index` by the
        //    implementation of `write_str`
        // 2. `self.buf` is guaranteed to be valid utf-8 because write_str only writes `&str`'s into
        //    the buffer, which are always valid utf-8
        unsafe { std::str::from_utf8_unchecked(&self.buf[0..self.index]) }
    }
}

impl<const N: usize> std::fmt::Write for StringFormatter<N> {
    fn write_str(&mut self, s: &str) -> Result<(), std::fmt::Error> {
        let src = s.as_ptr();
        let dst = &mut self.buf[self.index] as *mut u8;
        let len = s.len();

        if self.index + len > self.buf.len() {
            //Would overflow
            return Err(std::fmt::Error);
        }
        // # Safety
        //
        // 1. `src` is valid for reads length `len`, by the bounds check above
        // 2. `dst` is valid for writes length `len`, by the bounds check above
        //    - We have exclusive access to self so we have exclusive access to self.buf
        // 3. Because we have exclusive access to all of self.buf, then it is impossible for src to
        //    overlap dst
        unsafe { std::ptr::copy_nonoverlapping(src, dst, len) };
        self.index += len;

        Ok(())
    }
}
