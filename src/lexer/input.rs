// use std::str::CharIndices;
use std::str::Chars;

use super::loc::*;

#[derive(Debug,Clone)]
pub struct Input<'a> {
    chrs : Chars<'a>,
    // chrs:CharIndices<'a>,
    buf : String,
    // buf_byte_poss:Vec<usize>,
    loc : Loc,
    prev_loc : Loc,
    // line_byte_pos:usize,
    // prev_line_byte_pos:usize,
}

impl<'a> Input<'a> {
    pub fn new(
        // chrs :Chars<'a>,
        // chrs:CharIndices<'a>,
        src:&'a str,
    ) -> Input {
        Self {
            chrs : src.chars(), 
            // chrs:src.char_indices(),
            buf : String::new(), //Vec::new(), 
            // buf_byte_poss:Vec::new(),
            // loc : Loc::default(),
            loc : Loc::one(),
            prev_loc : Loc::one(),
            // line_byte_pos:0,
            // prev_line_byte_pos:0,
        }
    }

    
    pub fn reserve(&mut self, m : usize) {
        while self.buf.len() < m {
            if let Some(c) = self.chrs.next() {
                self.buf.push(c);
                // self.buf_byte_poss.push(c_ind);
            } else {
                break;
            }
        }
    }
    pub fn get(&self, i : usize, n : usize) -> Option<&str> {
    
        let m = i+n;

        if self.buf.len() >= m { //on n is 0, returns Some("") ............why not None? Because consume() needs to?

            //
            let a=self.buf.char_indices().nth(i).map(|x|x.0).unwrap_or_default();
            let b=self.buf.char_indices().nth(m).map(|x|x.0).unwrap_or(self.buf.len());

            Some(&(self.buf[a..b]))
            


    
            // None
            //
            // Some(&(self.buf[i..m]))
        } else {
            None
        }
    }

    pub fn get_reserve(&mut self, i : usize, n : usize) -> Option<&str> {
        self.reserve(i+n);
        self.get(i, n)
    }

    fn calc_loc(&mut self, n : usize) {
        let mut loc = self.loc;

        // let mut line_inds=Vec::new();

        if let Some(v) = self.get_reserve(0,n) {
            for c in v.chars() {
                loc.pos+=1;

                if c=='\n' {
                    loc.row+=1;
                    // loc.col=0;
                    loc.col=1; //starts at 1, not 0
                    // loc.line_pos = loc.pos;
                    // line_inds.push(i);
                } else if c!='\r' {
                    loc.col+=1;
                }
            }
        
            loc.byte_pos+=v.len();
            self.prev_loc=self.loc;
            self.loc = loc;


            // for i in line_inds {
            //     self.prev_line_byte_pos=self.line_byte_pos;
            //     self.line_byte_pos=self.buf_byte_poss.get(i).cloned().unwrap()+1; // plus 1 so after the \n
            // }
        }
    }

    pub fn next(&mut self, n : usize) -> bool {
        let e=self.buf.len()<n;
        self.calc_loc(n);
        let r=0 .. self.buf.len().min(n);
        self.buf.drain(r.clone());
        // self.buf_byte_poss.drain(r);
        e
    }

    pub fn has_buf_left(&self, n : usize) -> bool {
        self.buf.len()>=n
    }

    pub fn loc(&self) -> Loc {
        self.loc
    }
    // pub fn prev_loc(&self) -> Loc {
    //     self.prev_loc
    // }
    // pub fn line_byte_pos(&self) -> usize {
    //     self.line_byte_pos
    // }
    // pub fn prev_line_byte_pos(&self) -> usize {
    //     self.prev_line_byte_pos
    // }
}
