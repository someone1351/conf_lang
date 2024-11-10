 


mod def;
mod lexer;
mod conf;
mod parser;
mod walk;


use std::path::Path;

// use parser::parse_start;

pub use conf::Conf;
pub use conf::container::record::RecordContainer;
pub use conf::container::value::ValueContainer;

pub use conf::container::ancestor_iter::AncestorIter;
pub use conf::container::child_iter::ChildIter;
pub use conf::container::value_iter::ValueIter;

pub use def::Def;
pub use def::container::branch::BranchContainer;
pub use def::container::node::NodeContainer;
pub use def::container::node_children::NodeChildrenContainer;

pub use def::container::node_iter::NodeContainerIter;

pub use lexer::Loc;
pub use parser::error::{ParseError,ParseErrorType};
pub use walk::{Walk,error::WalkError};

// fn parse<'a>(
//     walk_branch:BranchContainer,
//     src : &'a str, 
//     keep_src:bool,
//     path:Option<&'a Path>,
// ) -> Result<Conf,ParseError> {
//     parse_start(walk_branch,src,keep_src,path)
// }

fn error_line_src(src : &str, loc : Loc) -> String {
    if src.is_empty() {
        String::new()
    } else {
        let start=src[0..loc.byte_pos].chars().rev().position(|x|x=='\n').map(|x|loc.byte_pos-x).unwrap_or_default();
        let p=loc.byte_pos+(loc.byte_pos==src.len()).then_some(0).unwrap_or(1);
        let end=src[p..].chars().position(|x|"\r\n".contains(x)).map(|x|x+loc.byte_pos+1).unwrap_or(src.len());
        let spcs=" ".repeat(src[start..loc.byte_pos].chars().filter(|x|*x!='\t').count());
        let tbs="\t".repeat(src[start..loc.byte_pos].chars().filter(|x|*x=='\t').count());
        let line=src.get(start..end).unwrap();
        format!("`{line}`\n {spcs}{tbs}^")
    }
}

fn error_msg<T:std::fmt::Debug>(error_type:T,loc:Loc,src : Option<&str>, path:Option<&Path> ) -> String {
    use std::fmt::Write;

    let mut output=String::new();

    write!(output,"Error {error_type:?} ").unwrap();

    if let Some(path)=path {
        write!(output,"in {path:?}, ").unwrap();
    }
    
    write!(output,"at {loc}").unwrap();

    if let Some(src)=src {
        write!(output,":\n{}",error_line_src(src, loc)).unwrap();
    } else {
        write!(output,".").unwrap();
    }

    output
}
