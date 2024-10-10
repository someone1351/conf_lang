

use std::path::PathBuf;


use super::super::error_msg;

use super::super::lexer::Loc;



#[derive(Debug,Clone)]
pub enum ParseErrorType {
    Unknown,
    DefChildrenBranchNotFound(String),
    DefNoChildren,
    NoDefForRecord,
    TagOnce(String),

    NoClosingQuote(String),
    InvalidIndent,
    InvalidIndentIncrement,
    ExpectedEOL,
}


// impl std::fmt::Display for ParseErrorType {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         write!(f,"{self:?}")
//     }
// }

#[derive(Debug,Clone)]
pub struct ParseError { //<'a>
    // pub src:Option<&'a str>,
    // pub src:&'a str,
    // pub path:Option<&'a Path>,
    pub path:Option<PathBuf>,
    pub loc : Loc,
    pub error_type : ParseErrorType,
}

impl ParseError {
    pub fn msg(&self,src:Option<&str>) -> String {
        error_msg(&self.error_type, self.loc, src, self.path.as_ref().map(|p|p.as_path()))
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // write!(f,"{}",error_msg(&self.error_type,self.loc,None,self.path.as_ref().map(|p|p.as_path())))
        write!(f,"{:?} at {}",self.error_type,self.loc)
    }
}

impl std::error::Error for ParseError {
    fn description(&self) -> &str {
        "confdef error"
    }
}
