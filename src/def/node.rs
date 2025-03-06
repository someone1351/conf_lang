use std::any::Any;


// pub enum NodeParamType {
//     Param
//     Group
// }

// pub type Param=Option<(std::any::TypeId,&'static str,ParamParse)>;
pub type Param=(std::any::TypeId,&'static str,ParamParse);

#[derive (Copy,Clone,Default,PartialEq,Eq)]
pub enum GroupSimilar {
    #[default]
    None,
    Left,
    Right,
}

#[derive (Default)]
pub struct ParamGroup {
    // pub priority:i32,
    pub optional:bool, //group optional
    pub param_optional : Option<usize>, //
    pub repeat:bool,
    // pub first:bool,
    pub name:Option<String>,
    // pub params : Vec<Param>,
    pub params : Vec<Option<usize>>,
    pub pattern_len:usize,
    pub patterns_num:usize,

    // pub specified:bool, //actually added via .group()
    pub similar:GroupSimilar,
}

#[derive (Clone,PartialEq,Eq,Default)]
pub enum NodeChildren {
    #[default]
    None,
    Branch(String),
    // BranchIndex(usize),
    Body(Option<String>),
}

// pub type ParamParse = fn(&str)->Option<Box<dyn Any+Send+Sync>>;

pub type ParamParse = Box<dyn Fn(&str)->Option<Box<dyn Any+Send+Sync>>+'static>;

#[derive (Default)]
pub struct Node {
    // pub vals : Vec<Option<(std::any::TypeId,Box<dyn Fn(&str)->Option<Box<dyn Any>>>)>>,
    // pub params : Vec<Option<(std::any::TypeId,&'static str,ParamParse)>>,
    pub param_groups : Vec<ParamGroup>,
    pub has_tag : bool,
    // pub params_repeat : bool,
    // pub params_repeat:Option<(Bound<usize>,Bound<usize>)>,
    // pub params_repeats:Vec<Range<usize>>,
    // // pub params_optional : bool, //doesnt make sense for tagless nodes, since they need atleast one param
    // pub params_optional : Option<usize>,
    pub node_label : Option<String>,
    pub children : NodeChildren,
    pub branch_ind:usize,
    pub tag_once:bool,
    // pub params
    // pub tag_valstr_key:usize,
    // ignore_children : bool,
    // pub rsimilar:bool,
}

// impl<> Node {
//     pub fn new(has_tag : bool,branch_ind:usize) -> Self {
//         Self {
//             has_tag,
//             branch_ind,
//             params : Vec::new(),
//             params_repeat:None,
//             // params_repeat:false,
//             // params_optional:false,
//             params_optional:None,
//             node_label : None,
//             children : NodeChildren::None,
//             // ignore_children:false,
//             tag_once:false,
//             // tag_valstr_key:0,
//         }
//     }
// }