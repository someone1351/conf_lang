use std::any::Any;
use std::marker::PhantomData;

use super::value::ValueContainer;
use super::super::Conf;

#[derive(Clone)]
pub struct ValueParsedIter<'a,T> {
    pub(super) conf_value_start : usize,
    pub(super) conf_value_end : usize,
    pub(super) conf : Option<&'a Conf>,
    pub(super) phantom_data: PhantomData<T>,
}

impl<'a,T:Any> Default for ValueParsedIter<'a,T> {
    fn default() -> Self {
        Self {
            conf_value_start:0,
            conf_value_end:0,
            conf:None,
            phantom_data:Default::default(),
        }
    }
}

impl<'a,T:Any+Clone> Iterator for ValueParsedIter<'a,T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.conf_value_start==self.conf_value_end {
            None
        } else {
            let value_index=self.conf_value_start;
            self.conf_value_start+=1;

            let v=ValueContainer {
                conf:self.conf,
                conf_value_ind: value_index,
            };

            v.get_parsed()
        }
    }
}

impl<'a,T:Any+Clone> DoubleEndedIterator for ValueParsedIter<'a,T> {
    fn next_back(&mut self) -> Option<T> {
        if self.conf_value_end > self.conf_value_start {
            self.conf_value_end-=1;
            
            let v=ValueContainer {
                conf:self.conf,
                conf_value_ind: self.conf_value_end,
            };

            v.get_parsed()
        } else {
            None
        }
    }
}
