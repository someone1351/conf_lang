use super::record::RecordContainer;
use super::super::Conf;

#[derive(Default,Clone)]
pub struct AncestorIter<'a> {
    pub(super) conf_record_ind : usize,
    pub(super) conf : Option<&'a Conf>,
    // pub(super) record : Option<RecordContainer<'a>>,
}

impl<'a> Iterator for AncestorIter<'a> {
    type Item = RecordContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(conf)=self.conf else {return None;};
        let record=conf.records.get(self.conf_record_ind).unwrap();
        
        record.parent.map(|parent|RecordContainer {
            conf:Some(conf),
            conf_record_ind:parent,
        })
    }
}