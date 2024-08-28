use alloc::sync::Arc;
use crate::mm::UserBuffer;

pub const BUF_LEN: usize = 256;
pub const MAIL_MAX: usize = 16;

pub struct Post {
    content: [u8; BUF_LEN],
    max_len: usize,
}

impl Post {
    pub fn new(buf: UserBuffer) -> Self {
        let mut cur_pos = 0;
        let mut content = [0u8; BUF_LEN];
        for sl in &buf.buffers {
            content[cur_pos..(cur_pos + sl.len())].copy_from_slice(sl);
            cur_pos += sl.len();
        }
        Self {
            content,
            max_len: cur_pos,
        }
    }

    pub fn read(&self, dst: &mut UserBuffer) -> isize {
        let mut num = 0;
        for sl in &mut dst.buffers {
            let n = sl.len();
            let l = n.min(self.max_len - num);
            sl[..l].copy_from_slice(&self.content[num..(num + l)]);
            num += l;
            if num == self.max_len {
                break;
            }
        }
        num as isize
    }
}

#[derive(PartialEq)]
pub enum MailStatus {
    Empty,
    Normal,
    Full,
}

pub struct Mail {
    queue: [Option<Arc<Post>>; MAIL_MAX],
    head: usize,
    tail: usize,
    status: MailStatus,
}

impl Mail {
    pub fn new() -> Self {
        Self {
            queue: [const { None }; MAIL_MAX],
            head: 0,
            tail: 0,
            status: MailStatus::Empty,
        }
    }

    pub fn fetch(&mut self) -> Option<Arc<Post>> {
        if !self.readable() {
            return None;
        }
        let res = self.queue[self.head].clone().unwrap().clone();
        self.head = (self.head + 1) % MAIL_MAX;
        if self.head == self.tail {
            self.status = MailStatus::Empty;
        }
        else if self.status == MailStatus::Full {
            self.status = MailStatus::Normal;
        }
        Some(res)
    }

    pub fn push(&mut self, post: &Arc<Post>) -> isize {
        if !self.writable() {
            return -1;
        }
        self.queue[self.tail] = Some(Arc::clone(post));
        self.tail = (1 + self.tail) % MAIL_MAX;
        if self.status == MailStatus::Empty {
            self.status = MailStatus::Normal;
        }
        else if self.head == self.tail {
            self.status = MailStatus::Full;
        }
        0
    }

    pub fn readable(&self) -> bool {
        self.status != MailStatus::Empty
    }

    pub fn writable(&self) -> bool {
        self.status != MailStatus::Full
    }
}
