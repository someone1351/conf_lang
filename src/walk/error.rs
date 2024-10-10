use std::fmt::Debug;
use std::path::PathBuf;

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