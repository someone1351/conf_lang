use super::value::ValueContainer;
use super::super::Conf;

#[derive(Default,Clone)]
pub struct ValueStrIter<'a> {
    pub(super) conf_value_start : usize,
    pub(super) conf_value_end : usize,
    pub(super) conf : Option<&'a Conf>,
}

impl<'a> Iterator for ValueStrIter<'a> {
    type Item = &'a str;

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

            Some(v.str())
        }
    }
}

impl<'a> DoubleEndedIterator for ValueStrIter<'a> {
    fn next_back(&mut self) -> Option<&'a str> {
        if self.conf_value_end > self.conf_value_start {
            self.conf_value_end-=1;
            
            let v=ValueContainer {
                conf:self.conf,
                conf_value_ind: self.conf_value_end,
            };

            Some(v.str())
        } else {
            None
        }
    }
}
