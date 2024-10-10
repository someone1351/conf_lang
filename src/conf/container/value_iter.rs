use std::any::Any;

use super::value::ValueContainer;
use super::super::Conf;
use super::value_parsed_iter::ValueParsedIter;
use super::value_str_iter::ValueStrIter;

#[derive(Default,Clone)]
pub struct ValueIter<'a> {
    pub(super) conf_value_start : usize,
    pub(super) conf_value_end : usize,
    pub(super) conf : Option<&'a Conf>,
}

impl<'a> ValueIter<'a> {
    pub fn str(&self) -> ValueStrIter<'a> {
        if self.conf.is_none() {return Default::default();};

        ValueStrIter {
            conf_value_start:self.conf_value_start,
            conf_value_end:self.conf_value_end,
            conf:self.conf,
        }
    }

    pub fn parsed<T:Any>(&self) -> ValueParsedIter<'a,T> {
        if self.conf.is_none() {
            return  ValueParsedIter {
                conf_value_start:0,
                conf_value_end:0,
                conf:None,
                phantom_data:Default::default(),
            };
        };
        
        // println!("hmm {:?} {} for {:?}",std::any::type_name::<T>(),self.record().values.len(),self.node_label());

        ValueParsedIter {
            conf_value_start:self.conf_value_start,
            conf_value_end:self.conf_value_end,
            conf:self.conf,
            phantom_data:Default::default(),
        }
    }
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = ValueContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.conf_value_start==self.conf_value_end {
            None
        } else {
            let value_index=self.conf_value_start;
            self.conf_value_start+=1;

            Some(ValueContainer {
                conf:self.conf,
                conf_value_ind: value_index,
            })
        }
    }
}

impl<'a> DoubleEndedIterator for ValueIter<'a> {
    fn next_back(&mut self) -> Option<ValueContainer<'a>> {
        if self.conf_value_end > self.conf_value_start {
            self.conf_value_end-=1;
            
            Some(ValueContainer {
                conf:self.conf,
                conf_value_ind: self.conf_value_end,
            })
        } else {
            None
        }
    }
}
