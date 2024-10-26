
pub mod container;


use std::{any::Any, collections::HashMap, ops::Range, path::{Path, PathBuf}};

// use container::record::RecordContainer;
use super::lexer::Loc;



#[derive(Debug,Clone)]
pub struct Value {
    pub start_loc : Loc,
    pub end_loc : Loc,
    pub text_ind : usize,
    pub parsed_ind : Option<usize>,
}

#[derive(Default, Debug)]
pub struct Record {
    pub parent : Option<usize>,
    pub children : Range<usize>,
    pub children_text:bool,
    pub conf_values : Range<usize>,
    pub param_groups : Range<usize>,
    pub node_label : Option<usize>,
    pub branch_name : Option<usize>,
    pub tag:bool,
}

#[derive(Default, Debug)]
pub struct ParamGroup {
    pub conf_values : Range<usize>,
    pub name : Option<usize>,
    pub params_num:usize,
    pub optional:bool,
    pub repeat:bool,
}

#[derive(Default, Debug)]
pub struct Conf {
    pub(super) records : Vec<Record>,
    pub(super) texts : Vec<String>,
    pub(super) path : Option<PathBuf>,
    pub(super) src : Option<String>,
    pub(super) values : Vec<Value>,
    pub(super) param_groups : Vec<ParamGroup>,
    pub(super) param_group_name_map : HashMap<String,usize>, //[name]=text_ind
    pub(super) param_group_map : HashMap<(usize,usize),usize>, //[(text_ind,record_ind)]=param_group
    pub(super) parsed_values : Vec<(&'static str,Box<dyn Any+Send+Sync>)>,
}

impl Conf {
    pub fn root(&self) -> container::record::RecordContainer {
        container::record::RecordContainer::new_root(self)
    }

    pub fn src(&self) -> Option<&str> {
        self.src.as_ref().map(|x|x.as_str())
    }
    
    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(|x|x.as_path())
    }
}

