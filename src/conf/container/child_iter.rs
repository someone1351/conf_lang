
use super::super::Conf;
use super::record::RecordContainer;

#[derive(Default,Clone)]
pub struct ChildIter<'a> {
    pub(super) conf_record_start : usize,
    pub(super) conf_record_end : usize, 
    pub(super) conf :Option<&'a Conf>,
}

impl<'a> Iterator for ChildIter<'a> {
    type Item = RecordContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.conf_record_start < self.conf_record_end {
            let i=self.conf_record_start;
            self.conf_record_start+=1;

            Some(RecordContainer {
                conf_record_ind: i,
                conf: self.conf,
            })
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for ChildIter<'a> {
    fn next_back(&mut self) -> Option<RecordContainer<'a>> {
        if self.conf_record_end > self.conf_record_start {
            self.conf_record_end-=1;
            
            Some(RecordContainer {
                conf_record_ind: self.conf_record_end,
                conf: self.conf,
            })
        } else {
            None
        }
    }
}
