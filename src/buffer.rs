use crate::util;
use crate::Result;
use std::io::BufRead;
use std::io::Seek;

#[derive(Debug)]
pub(crate) struct Buffer {
    chunk: Vec<String>,
    reader: Option<std::io::BufReader<std::fs::File>>,
    curline: usize,
    maxline: usize,
}

impl Buffer {
    pub(crate) fn new() -> Result<Self> {
        let mut buffer = Self {
            chunk: Vec::new(),
            reader: None,
            curline: 0,
            maxline: 0,
        };
        assert!(buffer.is_dead());
        buffer.update()?;
        Ok(buffer)
    }

    pub(crate) fn init(&mut self, f: &str) -> std::io::Result<()> {
        assert!(self.reader.is_none());
        let fp = std::fs::File::open(f)?;
        self.reader = Some(std::io::BufReader::new(fp));
        self.update()?;
        Ok(())
    }

    pub(crate) fn get_max_line(&mut self) -> usize {
        self.block_till_ready();
        let ret = self.maxline;
        self.signal_blocked();
        ret
    }

    pub(crate) fn is_dead(&mut self) -> bool {
        self.block_till_ready();
        let ret = self.reader.is_none();
        self.signal_blocked();
        ret
    }

    pub(crate) fn update(&mut self) -> std::io::Result<()> {
        if self.is_dead() {
            return Ok(());
        }
        self.block_till_ready();
        let r = self.reader.as_mut().ok_or_else(util::error)?;
        let tmp = r.stream_position()?;
        r.seek(std::io::SeekFrom::Start(0))?; // affects BufRead::lines
        self.maxline = 0;
        for _ in r./*by_ref().*/lines() {
            self.maxline += 1;
        }
        r.seek(std::io::SeekFrom::Start(tmp))?;
        self.signal_blocked();
        Ok(())
    }

    pub(crate) fn readline(
        &mut self,
        pos: &mut usize,
        s: &mut String,
        standout: &mut bool,
        showlnum: bool,
        blinkline: bool,
    ) -> std::io::Result<()> {
        s.clear();
        if self.reader.as_mut().ok_or_else(util::error)?.read_line(s)? == 0 || s.is_empty() {
            return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
        }

        // rstrip \n and then replace % with %%
        *s = match s.strip_suffix('\n') {
            Some(v) => v,
            None => s,
        }
        .to_string()
        .replace('%', "%%");

        if blinkline {
            if self.curline >= self.chunk.len() {
                self.chunk.resize(self.chunk.len() * 2 + 1, "".to_string());
            }
            *standout =
                !self.chunk[self.curline].is_empty() && self.chunk[self.curline] != s.as_str();
            self.chunk[self.curline] = s.clone();
        } else {
            *standout = false;
        }

        *pos = self.curline;
        self.curline += 1;
        if showlnum {
            *s = format!("{} {}", self.curline, s);
        }
        Ok(())
    }

    // caller needs to test if ready
    pub(crate) fn clear(&mut self) -> std::io::Result<()> {
        self.reader
            .as_mut()
            .ok_or_else(util::error)?
            .seek(std::io::SeekFrom::Start(0))?;
        self.curline = 0;
        Ok(())
    }

    pub(crate) fn block_till_ready(&mut self) {
        // NOP unless fine grained locking
    }

    pub(crate) fn signal_blocked(&mut self) {
        // NOP unless fine grained locking
    }
}
