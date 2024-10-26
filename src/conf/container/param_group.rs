// use std::any::Any;

use std::ops::{Bound, RangeBounds};

use super::super::super::conf::ParamGroup;

use super::super::Conf;
use super::value::ValueContainer;
use super::value_iter::ValueIter;
// use super::value_parsed_iter::ValueParsedIter;
// use super::value_str_iter::ValueStrIter;

#[derive(Clone,Copy,Default)]
pub struct ParamGroupContainer<'a> {
    pub(super) conf : Option<&'a Conf>,
    pub(super) conf_param_group_ind : usize,
}

impl<'a> ParamGroupContainer<'a> {
    fn param_group(&self) -> &'a ParamGroup {
        &self.conf.unwrap().param_groups[self.conf_param_group_ind]
    }

    pub fn params_num(&self) -> usize {
        if self.conf.is_none(){return 0;}
        self.param_group().params_num
    }

    // pub fn repeats_num(&self) -> usize {
    //     (self.values_num()/self.params_num())-1
    // }
    
    pub fn many_num(&self) -> usize {
        if self.conf.is_none(){return 0;}
        self.values_num()/self.params_num()
    }

    // pub fn tuple_values(&self, tuple_ind:usize) -> ValueIter<'a> {
    //     let (value_start,value_end)=if tuple_ind<self.many_num() {
    //         let value_start=self.param_group().values.start+tuple_ind*self.params_num();
    //         let value_end=value_start+self.params_num();
    //         (value_start,value_end)
    //     } else {
    //         (0,0)
    //     };
        
    //     ValueIter {
    //         value_start,
    //         value_end,
    //         conf:self.conf
    //     }
    // }

    pub fn get_first(&self) -> Option<ValueContainer<'a>> {
        self.get_value(0)
    }
    pub fn get_last(&self) -> Option<ValueContainer<'a>> {
        self.get_value(if self.values_num()==0 {0}else{self.values_num()-1})
    }


    pub fn first(&self) -> ValueContainer<'a> {
        self.value(0)
    }
    pub fn last(&self) -> ValueContainer<'a> {
        self.value(if self.values_num()==0 {0}else{self.values_num()-1})
    }

    pub fn value(&self, ind : usize) -> ValueContainer<'a> {
        if self.conf.is_none() {return Default::default();};

        if ind >= self.values_num() {
            return Default::default();
        }

        ValueContainer {
            conf:self.conf,
            conf_value_ind: self.param_group().conf_values.start+ind,
        }
    }

    pub fn get_value(&self, ind : usize) -> Option<ValueContainer<'a>> {
        if self.conf.is_none() {return None;};

        if ind >= self.values_num() {
            return None;
        }

        Some(ValueContainer {
            conf:self.conf,
            conf_value_ind: self.param_group().conf_values.start+ind,
        })
    }
    
    pub fn values_num(&self) -> usize {
        if self.conf.is_none(){return 0;}
        self.param_group().conf_values.len()
    }
    
    pub fn values(&self) -> ValueIter<'a> {
        if self.conf.is_none() {return Default::default();};

        ValueIter {
            conf_value_start:self.param_group().conf_values.start,
            conf_value_end:self.param_group().conf_values.end,
            conf:self.conf,
        }
    }
    pub fn get_values<R:RangeBounds<usize>>(&self,r:R) -> Option<ValueIter<'a>> {
        if self.conf.is_none() {return None;}

        let range_start=match r.start_bound() {
            Bound::Included(x)=>*x,
            Bound::Excluded(_)=>panic!(""),
            Bound:: Unbounded=>0,
        };

        let range_end=match r.start_bound() {
            Bound::Included(x)=>*x+1,
            Bound::Excluded(x)=>*x,
            Bound:: Unbounded=>self.values_num(),
        };

        if range_start>range_end {return None;} //if range start==end will return some empty iter

        let x_len=range_end-range_start;

        if x_len>self.values_num() {return None;}

        let x_start=self.param_group().conf_values.start+range_start;
        let x_end = x_start+x_len;

        Some(ValueIter {
            conf_value_start:x_start,
            conf_value_end:x_end,
            conf:self.conf,
        })
    }
    pub fn get_parsed_array<T:Copy+'static, const COUNT: usize>(&self,init:T) -> Option<[T;COUNT]> {
        let mut array=[init;COUNT];

        for i in 0..COUNT {
            if let Some(value)=self.get_value(i) {
                if let Some(parsed)=value.get_parsed::<T>() {
                    array[i]=parsed;
                } else { //if parsed fails, then return nothing
                    return None; 
                }
            } else { //if not enough values, return what was already gotten, and the rest uses the init
                break;
            }
        }
        
        Some(array)
    }

    // pub fn strs(&self) -> ValueStrIter<'a> {
    //     if self.conf.is_none() {return Default::default();};
    //     ValueStrIter {
    //         conf_value_start:self.param_group().conf_values.start,
    //         conf_value_end:self.param_group().conf_values.end,
    //         conf:self.conf,
    //     }
    // }

    // pub fn parseds<T:Any+Copy>(&self) -> ValueParsedIter<'a,T> {
    //     if self.conf.is_none() {return Default::default();};
    //     ValueParsedIter {
    //         conf_value_start:self.param_group().conf_values.start,
    //         conf_value_end:self.param_group().conf_values.end,
    //         conf:self.conf,
    //         phantom_data:Default::default(),
    //     }
    // }
    // pub fn parsed<T:Any+Copy>(&self, param_group_value_ind : usize) -> Option<T> {
    //     self.value(param_group_value_ind).and_then(|x|x.parsed())
    // }
    
    // pub fn str(&self, param_group_value_ind : usize) -> Option<&'a str> {
    //     self.value(param_group_value_ind).map(|x|x.str())
    // }
    pub fn name(&self) -> Option<&'a str> {
        if self.conf.is_none() {return Default::default();};
        let text_ind=self.param_group().name;
        let text=text_ind.map(|text_ind|self.conf.unwrap().texts.get(text_ind).unwrap().as_str());
        text
    }
    
    pub fn is_empty(&self) -> bool {
        self.conf.is_none()
    }

    pub fn is_optional(&self) -> bool {
        if self.conf.is_none() {return false;};
        self.param_group().optional
    }
    pub fn is_repeat(&self) -> bool {
        if self.conf.is_none() {return false;};
        self.param_group().repeat
    }
}