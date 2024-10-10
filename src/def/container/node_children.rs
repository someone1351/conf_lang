use super::branch::BranchContainer;

#[derive (Clone,Copy)]

pub enum NodeChildrenContainer<'a> {
    None,
    Body(Option<&'a str>),
    Branch(BranchContainer<'a>),
    BranchMissing(&'a str),
}


impl<'a> NodeChildrenContainer<'a> {
    pub fn is_body(&self) -> bool {
        if let NodeChildrenContainer::Body(_) = self {
            true
        } else {
            false
        }
    }
    pub fn body_node_label(&self) -> Option<&'a str> {
        if let NodeChildrenContainer::Body(x) = self {
            *x
        } else {
            None
        }
    }
}
