use std::fmt::Debug;
use std::path::{Path, PathBuf};

// use crate::RecordContainer;

use super::super::lexer::Loc;
use super::super::error_msg;

#[derive(Debug)]
pub enum WalkErrorType<E:Debug> {
    Custom(E),
    RecursiveInclude,
}

#[derive(Debug)]
pub struct WalkError<E:Debug> { //'a,
    // pub src:Option<&'a str>,
    // pub path:Option<&'a Path>,
    pub path:Option<PathBuf>,
    pub loc : Loc,
    pub error_type : WalkErrorType<E>,
}

impl<E:Debug> WalkError<E> {
    pub fn new(p : Option<&Path>, loc:Loc, e:E) -> Self {
        Self { path: p.map(|p|p.to_path_buf()), loc, error_type: WalkErrorType::Custom(e)}
    }
    // pub fn from_record(record : RecordContainer, e:E) -> Self {
    //     Self { path: record.path().map(|p|p.to_path_buf()), loc: record.start_loc(), error_type: WalkErrorType::Custom(e)}
    // }
    pub fn msg(&self,src:Option<&str>) -> String {
        error_msg(&self.error_type, self.loc, src, self.path.as_ref().map(|p|p.as_path()))
    }
}

impl<E:Debug> std::fmt::Display for WalkError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // write!(f,"{}",error_msg(&self.error_type,self.loc,None,self.path.as_ref().map(|p|p.as_path())))
        write!(f,"{:?} at {}",self.error_type,self.loc)
    }
}

impl<E:Debug> std::error::Error for WalkError<E> {
    fn description(&self) -> &str {
        "confdef walk error"
    }
}