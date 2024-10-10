
#![allow(dead_code)]

mod loc;
mod error;
pub mod input;

pub use loc::Loc;

use error::*;
pub use input::*;

// use std::collections::HashSet;

// #[derive(Debug,Clone,Hash,PartialEq,Eq)]
// pub enum DebugLast {
//     None,
//     Has{i : usize, xs: Vec<String>, loc:Loc,},
//     Get{i : usize, n : usize, loc:Loc,},
// }

#[derive(Debug,Clone)]
pub struct Token {
    pub start_loc : Loc,
    pub end_loc : Loc,
    pub extracted : String,
}

#[derive(Debug,Clone)]
struct Hist<'a> {
    input : Input<'a>,
    token : Option<Token>,
}

pub struct Lexer<'a> {
    stk : Vec<Hist<'a>>,
    error_manager : ErrorManager,
    debug_label_strs:Vec<&'a str>,
    debug_print_enable:bool,
    // debug_break_loop_max:usize,
    // debug_break_loop_count:usize,
    // debug_lasts:HashSet<DebugLast>,
}

impl<'a> Lexer<'a> {
    pub fn new(
        // chrs :Chars<'a>
        src:&'a str,
    ) -> Self {
        Self {
            stk : vec![Hist {
                input : Input::new(src),
                token : None,
            }],
            error_manager : ErrorManager::new(),
            debug_label_strs: Vec::new(),
            debug_print_enable:false,
            // debug_break_loop_max:0,
            // debug_break_loop_count:0,
            // debug_lasts:HashSet::new(),
        } 
    }

    pub fn debug_label_push(&mut self,debug_label_str:&'a str) {
        self.debug_label_strs.push(debug_label_str); 
    }

    pub fn debug_label_pop(&mut self) {
        self.debug_label_strs.pop().unwrap();  
    }


    pub fn debug_print(&mut self,debug_print_enable:bool) {
        self.debug_print_enable=debug_print_enable;
    }
    // pub fn debug_break_loop(&mut self,debug_break_loop_num:usize) {
    //     self.debug_break_loop_max=debug_break_loop_num;
    // }
    
    fn debug_print_cmd(&self,cmd:&str) {
        if self.debug_print_enable {
            print!("\"");

            for &x in self.debug_label_strs.iter() {
                print!("/{x}");
            }

            println!("\" : {} : (pos={}, line={}, col={})",
                // self.debug_label_strs.last().cloned().unwrap_or(""),
                cmd,
                self.loc().pos,
                self.loc().row,
                self.loc().col,
            );
        }
    }

    pub fn loc(&self) -> Loc {
        let cur = self.stk.last().unwrap();
        cur.input.loc()
    }
    // pub fn prev_loc(&self) -> Loc {
    //     let cur = self.stk.last().unwrap();
    //     cur.input.prev_loc()
    // }

    // pub fn line_byte_pos(&self) -> usize {
    //     let cur = self.stk.last().unwrap();
    //     cur.input.line_byte_pos()
    // }
    // pub fn prev_line_byte_pos(&self) -> usize {
    //     let cur = self.stk.last().unwrap();
    //     cur.input.prev_line_byte_pos()
    // }

    pub fn stack_size(&self) -> usize {
        self.stk.len()-1
    }

    pub fn push(&mut self) {
        self.debug_print_cmd("push");

        let cur = self.stk.last().unwrap();

        let x = cur.clone();
        self.stk.push(x);

        self.error_manager.push();
    }

    pub fn pop_discard(&mut self) {
        self.debug_print_cmd("pop_discard");

        if self.stk.len() <= 1 {
            panic!("Lexer stack size 0");
        }

        self.error_manager.on_pop_discard();
        self.stk.pop().unwrap();
    }

    pub fn pop_keep(&mut self) {
        self.debug_print_cmd("pop_keep");

        if self.stk.len() <= 1 {
            panic!("Lexer stack size 0");
        }

        self.error_manager.on_pop_keep();        
        self.stk.remove(self.stk.len()-2);
    }


    pub fn has<'b,const N:usize>(&mut self, i:usize,xs: [&'b str;N]) -> Option<&'b str> {
        let mut res=None;
        
        let cur = self.stk.last_mut().unwrap();

        for &x in xs.iter() {
            if Some(x)==cur.input.get_reserve(i, x.chars().count()) {
                res=Some(x);
                break;
            }
        }

        //
        self.debug_print_cmd(format!("has({i}, {xs:?}) => {res:?}").as_str());
        
        // if self.debug_break_loop_max!=0 {
        //     let loc = self.loc();
        //     let debug_last=DebugLast::Has { i, xs:xs.iter().map(|x|x.to_string()).collect::<Vec<_>>(), loc };

        //     if self.debug_lasts.contains(&debug_last) {
        //         self.debug_break_loop_count+=1;
        //     }

        //     self.debug_lasts.insert(debug_last);

        //     if self.debug_break_loop_count>=self.debug_break_loop_max {
        //         // panic!("debug loop break");
        //     }
        // }

        res
    }
    
    pub fn get(&mut self, i : usize, n : usize) -> Option<&str> {

        //
        let cur = self.stk.last_mut().unwrap();
        cur.input.reserve(i+n);
        
        //
        let cur = self.stk.last().unwrap();
        let res=cur.input.get(i,n);

        self.debug_print_cmd(format!("get({i}, {n}) => {res:?}").as_str());

        // if self.debug_break_loop_max!=0 {
        //     let loc = self.loc();
 
        //     let debug_last=DebugLast::Get { i, n, loc };

        //     if self.debug_lasts.contains(&debug_last) {
        //         self.debug_break_loop_count+=1;
        //     }

        //     self.debug_lasts.insert(debug_last);

        //     if self.debug_break_loop_count>=self.debug_break_loop_max {
        //         panic!("debug loop break");
        //     }
        // }

        res
    }

    pub fn getc(&mut self, i : usize) -> Option<char> {   

        let cur = self.stk.last_mut().unwrap(); 
        let res=cur.input.get_reserve(i,1).and_then(|s|s.chars().last());

        // if let Some(s) = cur.input.get(i,1) {
        //     s.chars().last()
        // } else {
        //     None
        // }
        
        self.debug_print_cmd(format!("getc({i} => {res:?})").as_str());

        // if self.debug_break_loop_max!=0 {
        //     let n=1;
        //     let loc = self.loc();
        //     let debug_last=DebugLast::Get { i, n, loc };

        //     if self.debug_lasts.contains(&debug_last) {
        //         self.debug_break_loop_count+=1;
        //     }

        //     self.debug_lasts.insert(debug_last);

        //     if self.debug_break_loop_count>=self.debug_break_loop_max {
        //         panic!("debug loop break");
        //     }
        // }

        res
    }

    pub fn is_end(&mut self) -> bool {
        let cur = self.stk.last_mut().unwrap(); 
        let res=cur.input.get_reserve(0,1).and_then(|s|s.chars().last()).is_none();

        self.debug_print_cmd(format!("is_end => {res:?}").as_str());
        res
    }

    pub fn skip(&mut self, n : usize) -> Option<()> {

        //
        let cur = self.stk.last_mut().unwrap();

        //
        let res= if !cur.input.has_buf_left(n) {
            return None;
        } else {
            cur.input.next(n);
            self.error_manager.on_next(self.loc());
            Some(())
        };
        
        //
        self.debug_print_cmd(format!("skip({n}) => {res:?}").as_str());
        
        // if self.debug_break_loop_max!=0 && n!=0 && res.is_some() {
        //     self.debug_lasts.clear();
        // }

        res
    }

    pub fn consume(&mut self, n : usize, replace : Option<&str>) -> Option<()> {

        //
        let cur = self.stk.last_mut().unwrap();

        //
        let res=if !cur.input.has_buf_left(n) {
            None
        } else {
            if let None = cur.token {
                cur.token = Some(Token {
                    start_loc : cur.input.loc(),
                    end_loc : cur.input.loc(),
                    extracted : String::new(),
                });
            }

            if let Some(s) = cur.input.get_reserve(0,n) { //on n==0, s=""
                let s = if let Some(replace) = replace { replace } else {s};
                let token =  cur.token.as_mut().unwrap();
                token.extracted.extend(s.chars());
                cur.input.next(n);
                token.end_loc = cur.input.loc();
            }
            
            self.error_manager.on_next(self.loc());

            Some(())
        };

        self.debug_print_cmd(format!("consume({n}, {replace:?}) => {res:?}").as_str());
        
        // if self.debug_break_loop_max!=0 && n!=0 && res.is_some() {
        //     self.debug_lasts.clear();
        // }

        res

    }

    pub fn token(&mut self) -> Option<Token> {

        let cur = self.stk.last_mut().unwrap();
        let res=std::mem::take(&mut cur.token);
        
        // let t = self.cur.token ;
        // self.cur.token = None;
        // println!("{:?}",self.cur.token);
        // t

        self.debug_print_cmd(format!("token => {res:?}").as_str());
        res
    }

    pub fn set_token(&mut self, token : Token) {
        self.debug_print_cmd(format!("set_token({token:?})").as_str());

        let cur = self.stk.last_mut().unwrap();
        cur.token = Some(token);
    }

    // pub fn add_error(&mut self, msg_loc : Loc,msg : &str) {
    //     self.error_manager.add_error(self.loc(), msg_loc, msg);
    // }

    // pub fn get_errors(&self) -> &[Error] {
    //     self.error_manager.get_errors()
    // }

    // pub fn has_errors(&self) -> bool {
    //     self.error_manager.get_errors().len()>0
    // }
}