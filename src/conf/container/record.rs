
// use std::any::Any;
use std::fmt::Debug;
use std::path::Path;

use super::super::{Conf,Record};
use super::ancestor_iter::AncestorIter;
use super::child_iter::ChildIter;
use super::param_group::ParamGroupContainer;
use super::value::ValueContainer;
use super::value_iter::ValueIter;
// use super::value_str_iter::ValueStrIter;
// use super::value_parsed_iter::ValueParsedIter;
use super::super::super::lexer::Loc;

use super::super::super::walk::{traverse,Walk,error::WalkError};
// use super::MyIndex;


pub enum ParamGroupIndex<'a> {
    Name(&'a str),
    Index(usize),
}
// impl<'a> From<usize> for MyIndex<'a> {
//     fn from(item: usize) -> Self {
//         MyIndex::Index(item)
//     }
// }

// impl<'a> From<&'a str> for MyIndex<'a> {
//     fn from(item: &'a str) -> Self {
//         MyIndex::Name(item)
//     }
// }

impl<'a> Into<ParamGroupIndex<'a>> for &'a str {
    fn into(self) -> ParamGroupIndex<'a> {
        ParamGroupIndex::<'a>::Name(self)
    }
}
impl<'a> Into<ParamGroupIndex<'a>> for usize {
    fn into(self) -> ParamGroupIndex<'a> {
        ParamGroupIndex::<'a>::Index(self)
    }
}


#[derive(Clone,Copy,Default)]
pub struct RecordContainer<'a> {
    pub(super) conf_record_ind : usize,
    pub(super) conf : Option<&'a Conf>,
}

impl<'a> std::fmt::Debug for RecordContainer<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.conf.is_none() {return core::fmt::Result::Ok(())};
        self.conf.unwrap().records.get(self.conf_record_ind).unwrap().fmt(f)
    }
}

impl<'a> RecordContainer<'a> {
    pub fn new_root(conf : &'a Conf) -> Self { 
        Self { conf:Some(conf), conf_record_ind:0, }
    }

    // pub fn conf(&self) -> &'a Conf {
    //     self.conf
    // }

    fn record(&self) -> &'a Record {
        &self.conf.unwrap().records[self.conf_record_ind]
    }

    fn record_value_start_offset(&self) -> usize {
        self.record().tag.then_some(1).unwrap_or_default()
    }

    pub fn record_index(&self) -> usize {
        self.conf_record_ind
    }

    pub fn first(&self) -> ValueContainer<'a> {
        self.value(0)
    }
    pub fn last(&self) -> ValueContainer<'a> {
        self.value(if self.values_num()==0 {0}else{self.values_num()-1})
    }

    pub fn value(&self, ind : usize) -> ValueContainer<'a> {
        if self.conf.is_none() {return Default::default();};
        if ind >= self.values_num() {return Default::default();}

        ValueContainer {
            conf:self.conf,
            conf_value_ind: self.record().values.start+ind+self.record_value_start_offset(),
        }
    }

    // pub fn parsed<T:Any+Copy>(&self, record_value_ind : usize) -> Option<T> {
    //     self.value(record_value_ind).and_then(|x|x.parsed())
    // }
    
    // pub fn str(&self, record_value_ind : usize) -> Option<&'a str> {
    //     self.value(record_value_ind).map(|x|x.str())
    // }

    pub fn values_num(&self) -> usize {
        if self.conf.is_none() {return 0;};
        self.record().values.len()-self.record_value_start_offset()
    }

    // pub fn child_text_value(&self, i : usize) -> Option<ValueContainer<'a>> {
    //     if self.record().children_text {
    //         let value_index=self.record().children.start+i;
            
    //         (value_index < self.record().children.end).then(||ValueContainer {
    //             conf:self.conf,
    //             conf_value_ind: value_index,
    //         })
    //     } else {
    //         None
    //     }
    // }

    // pub fn child_text_str(&self, i : usize) -> Option<&'a str> {
    //     self.child_text_value(i).map(|x|x.str())
    // }

    // pub fn child_text_values_num(&self) -> usize {
    //     if self.record().children_text {
    //         self.record().children.len()
    //     } else {
    //         0
    //     }
    // }

    pub fn param_group<'b>(&self, ind : impl Into<ParamGroupIndex<'b>>) -> ParamGroupContainer<'a> {
        if self.conf.is_none() {return Default::default();};

        match ind.into() {
            ParamGroupIndex::Index(record_param_group_ind)=> {
                if record_param_group_ind < self.param_groups_num() {
                    ParamGroupContainer{
                        conf:self.conf,
                        conf_param_group_ind: self.record().param_groups.start+record_param_group_ind,
                    }
                } else {
                    Default::default()
                }
            },
            ParamGroupIndex::Name(param_group_name)=>{
                let Some(&text_ind)=self.conf.unwrap().param_group_name_map.get(param_group_name) else {return Default::default();};
                let Some(&param_group_ind)=self.conf.unwrap().param_group_map.get(&(text_ind,self.conf_record_ind)) else {return Default::default();};
                
                ParamGroupContainer { 
                    conf:self.conf,
                    conf_param_group_ind: param_group_ind,
                }
            }
        }
    }

    pub fn param_groups_num(&self) -> usize {
        if self.conf.is_none() {return 0;};
        self.record().param_groups.len()
    }

    pub fn child(&self, ind : usize) -> RecordContainer<'a>  {
        if self.conf.is_none() {return Default::default();};
        if self.record().children_text {return Default::default();}
        if ind>=self.children_num() {return Default::default();}

        Self {
            conf:self.conf,
            conf_record_ind :self.record().children.start+ind,
        }
     
    }

    pub fn children_num(&self) -> usize {
        if self.conf.is_none() {return 0;};

        if self.record().children_text {
            0
        } else {
            self.record().children.len()
        }
    }

    pub fn parent(&self) -> RecordContainer<'a> {
        if self.conf.is_none() {return Default::default();};

        let Some(parent_record_index)= self.conf.unwrap().records[self.conf_record_ind].parent else {
            return Default::default();
        };

        Self{
            conf:self.conf, 
            conf_record_ind :parent_record_index,
        }
    }

    pub fn has_parent(&self) -> bool {
        if self.conf.is_none() {return false;};
        self.record().parent.is_some()
    }

    pub fn ancestor(&self, ind : usize) -> RecordContainer<'a> {
        if self.conf.is_none() {return Default::default();};
        if !self.has_parent() {return Default::default();};

        let mut cur = self.parent();
        let mut j = 0;

        while cur.has_parent() {
            if j==ind {
                return cur;
            } else {
                cur = cur.parent();
                j+=1;
            }
        }

        Default::default()
    }

    pub fn ancestors(&self) -> AncestorIter<'a> {
        if self.conf.is_none() {return Default::default();};
        let Some(parent)=self.record().parent else {return Default::default()};

        AncestorIter { 
            // record: Some(*self) 
            conf:self.conf,
            conf_record_ind:parent,
        }
    }
    
    pub fn children(&self) -> ChildIter<'a> {
        if self.conf.is_none() {return Default::default();};

        let (child_record_start,child_record_end)=if self.record().children_text {
            (0,0)
        } else {
            (self.record().children.start,self.record().children.end)
        };
        
        ChildIter::<'a> {
            conf:self.conf,
            conf_record_start: child_record_start,
            conf_record_end: child_record_end,
        }
    }

    pub fn values(&self) -> ValueIter<'a> {
        if self.conf.is_none() {return Default::default();};

        ValueIter {
            conf_value_start:self.record().values.start+self.record_value_start_offset(),
            conf_value_end:self.record().values.end,
            conf:self.conf,
        }
    }

    // pub fn strs(&self) -> ValueStrIter<'a> {
    //     if self.conf.is_none() {return Default::default();};

    //     ValueStrIter {
    //         conf_value_start:self.record().values.start+self.record_value_start_offset(),
    //         conf_value_end:self.record().values.end,
    //         conf:self.conf,
    //     }
    // }

    // pub fn parseds<T:Any+Copy>(&self) -> ValueParsedIter<'a,T> {
    //     if self.conf.is_none() {return Default::default();};
        
    //     // println!("hmm {:?} {} for {:?}",std::any::type_name::<T>(),self.record().values.len(),self.node_label());

    //     ValueParsedIter {
    //         conf_value_start:self.record().values.start+self.record_value_start_offset(),
    //         conf_value_end:self.record().values.end,
    //         conf:self.conf,
    //         phantom_data:Default::default(),
    //     }
    // }

    pub fn text_values(&self) -> ValueIter<'a> {
        if self.conf.is_none() {return Default::default();};

        let (value_start,value_end)=if self.record().children_text {
            (self.record().children.start,self.record().children.end)
        } else {
            (0,0)
        };
        
        ValueIter {
            conf_value_start: value_start,
            conf_value_end: value_end,
            conf:self.conf,
        }
    }
    
    // pub fn text_strs(&self) -> ValueStrIter<'a> {
    //     if self.conf.is_none() {return Default::default();};

    //     let (value_start,value_end)=if self.record().children_text {
    //         (self.record().children.start,self.record().children.end)
    //     } else {
    //         (0,0)
    //     };
        
    //     ValueStrIter {
    //         conf_value_start: value_start,
    //         conf_value_end: value_end,
    //         conf:self.conf,
    //     }
    // }

    pub fn has_text(&self) -> bool {
        if self.conf.is_none() {return Default::default();};
        self.record().children_text && self.record().children.len()!=0
    }

    pub fn has_children(&self) -> bool {
        if self.conf.is_none() {return Default::default();};
        !self.record().children_text && self.record().children.len()!=0
    }
    
    pub fn is_children_text(&self) -> bool {
        if self.conf.is_none() {return Default::default();};
        self.record().children_text
    }
    
    pub fn path(&self) -> Option<&'a Path> {
        if self.conf.is_none() {return Default::default();};
        self.conf.unwrap().path.as_ref().and_then(|x|Some(x.as_path()))
    }

    pub fn src(&self) -> Option<&'a str> {
        if self.conf.is_none() {return Default::default();};
        self.conf.unwrap().src.as_ref().and_then(|x|Some(x.as_str()))
    }
        
    pub fn branch_name(&self) -> Option<&'a str> {
        if self.conf.is_none() {return Default::default();};
        self.record().branch_name.map(|text_ind|self.conf.unwrap().texts.get(text_ind).unwrap().as_str())
    }

    pub fn node_label(&self) -> Option<&'a str> {
        if self.conf.is_none() {return Default::default();};
        self.record().node_label.map(|text_ind|self.conf.unwrap().texts.get(text_ind).unwrap().as_str())
    }

    pub fn walk(&self,mut callback : impl for<'b> FnMut(Walk<'b,'a>) -> Option<RecordContainer<'a>>) -> Result<(),WalkError<()>> { //'a,
        if self.conf.is_none() {return Ok(());};
        traverse(*self, |w|Ok(callback(w)))
    }

    pub fn walk_ext<E:Debug>(&self,callback : impl for<'b> FnMut(Walk<'b,'a>) -> Result<Option<RecordContainer<'a>>,(E,Option<Loc>)>) -> Result<(),WalkError<E>> { //'a,
        if self.conf.is_none() {return Ok(());};
        traverse(*self, callback)
    }

    pub fn tag(&self) -> Option<&'a str> {
        if self.conf.is_none() {return Default::default();};

        self.record().tag.then(||{
            let text_ind=self.conf.unwrap().values.get(self.record().values.start).unwrap().text_ind;
            self.conf.unwrap().texts.get(text_ind).unwrap().as_str()
        })
    }

    pub fn has_tag(&self) -> bool {
        if self.conf.is_none() {return Default::default();};
        self.record().tag
    }

    pub fn start_loc(&self) -> Loc {
        if self.conf.is_none() {return Default::default();};

        let val_ind=self.record().values.start;
        self.conf.unwrap().values.get(val_ind).unwrap().start_loc
    }

    pub fn end_loc(&self) -> Loc {
        if self.conf.is_none() {return Default::default();};

        let val_ind=self.record().values.end-1;
        self.conf.unwrap().values.get(val_ind).unwrap().end_loc
    }

    pub fn is_empty(&self) -> bool {
        self.conf.is_none()
    }
}