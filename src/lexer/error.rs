

use super::loc::Loc;

#[derive(Debug,Clone)]
pub struct Error {
    pub loc : Loc,
    pub msg_loc : Loc,
    pub msg : String,
}

#[derive(Debug,Clone)]
struct Hist {
    owned_ind : usize,
    start_ind : usize,
}

pub struct ErrorManager {
    errors : Vec<Error>,
    hists : Vec<Hist>,
}

impl ErrorManager {
    pub fn new() -> Self {
        Self {
            errors : Vec::new(),
            hists : vec![Hist {
                owned_ind : 0,
                start_ind : 0,
            }],
        }
    }

    pub fn on_next(&mut self, cur_loc : Loc) {
        let cur_hist = self.hists.last_mut().unwrap();
        
        // self.errors.drain_filter(|x|x.loc<cur_loc);
        // self.errors.drain(cur_hist.owned_ind .. self.errors.len());

        let mut i = cur_hist.owned_ind;
        while i < self.errors.len() {
            if self.errors[i].loc<cur_loc {
                self.errors.remove(i);
            } else {
                i += 1;
            }
        }

        cur_hist.start_ind=self.errors.len();
        
        //cur.error_own_ind;//
    }

    pub fn push(&mut self) {
        let cur = self.hists.last().unwrap();
        let cur_start_ind = cur.start_ind;

        self.hists.push(Hist {
            start_ind : cur_start_ind,
            owned_ind : self.errors.len()
        });
    }

    pub fn on_pop_discard(&mut self) {
        self.hists.pop();

        //keep errors
    }

    pub fn on_pop_keep(&mut self) {
        //0 <= cur.start <= cur.owned

        let prev = self.hists.get(self.hists.len()-2).unwrap();
        let prev_own_ind = prev.owned_ind;
        let prev_start_ind = prev.start_ind;
        let cur = self.hists.last_mut().unwrap();
       
        self.errors.drain(prev_own_ind .. cur.start_ind);
        // for i in prev_own_ind .. cur.start_ind {
        //     self.errors.remove(prev_own_ind);
        // }
        
        cur.start_ind=cur.start_ind.min(prev_start_ind);
        cur.owned_ind=prev_own_ind;

        self.hists.remove(self.hists.len()-2);
    }

    pub fn add_error(&mut self, loc : Loc, msg_loc : Loc,msg : &str) {
        //println!("add err {:?} {:?}",self.loc(),msg);
        self.errors.push(Error {
            loc,
            msg_loc,
            msg:msg.to_string(),
        });
    }

    pub fn get_errors(&self) -> &[Error] {
        let cur = self.hists.last().unwrap();

        let start = cur.start_ind;
        let end = self.errors.len();

        self.errors[start..end].as_ref()
    }
}