// use std::any::{Any, TypeId};
// use std::ops::{Bound, Range};


use super::branch::BranchContainer;
// use super::super::grammar::Grammar;
use super::super::Def;
use super::super::node::NodeChildren;
use super::node_children::NodeChildrenContainer;
use super::param_group::ParamGroupContainer;


#[derive (Clone,Copy)]
pub struct NodeContainer<'a> {
    pub(in super::super) def:&'a Def,
    // pub branch_ind:usize,
    pub(in super::super) node_ind:usize,
}

impl<'a> NodeContainer<'a> {
    pub fn children(&self) -> NodeChildrenContainer<'a> {
        let node=self.def.nodes.get(self.node_ind).unwrap();

        match &node.children {
            NodeChildren::None => NodeChildrenContainer::None,
            NodeChildren::Body(body_node_label) => NodeChildrenContainer::Body(body_node_label.as_ref().map(|x|x.as_str())),
            NodeChildren::Branch(branch_name) => {
                self.def.branch_map.get(branch_name.as_str())
                    .map(|&children_branch_ind|BranchContainer {
                        def: self.def,
                        branch_ind: children_branch_ind,
                    }).map(|branch|NodeChildrenContainer::Branch(branch))
                    .unwrap_or(NodeChildrenContainer::BranchMissing(branch_name.as_str()))
            },
        }
    }

    pub fn branch_ind(&self) -> usize {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        node.branch_ind
    }

    pub fn branch(&self) -> BranchContainer {
        let branch_ind=self.branch_ind();
        BranchContainer { def: self.def, branch_ind, }
    }

    pub fn node_ind(&self) -> usize {
        self.node_ind
    }
    // pub fn params_repeats_num(&self) -> usize {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params_repeats.len()
    // }
    // pub fn params_repeats(&self, ind:usize) -> Option<Range<usize>> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params_repeats.get(ind).cloned()
    // }

    // pub fn params_repeat(&self) -> Option<Range<usize>> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();

    //     let Some((start,end))=node.params_repeat else {
    //         return None;
    //     };

    //     let start=match start {
    //         Bound::Unbounded => 0,
    //         Bound::Included(x) => x,
    //         Bound::Excluded(_) => panic!(""),
    //     };

    //     let end=match end {
    //         Bound::Unbounded => node.params.len(),
    //         Bound::Included(x) => x+1,
    //         Bound::Excluded(x) => x,
    //     };

    //     Some(start..end)
    // }

    // pub fn params_optional(&self) -> Option<usize> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params_optional
    // }

    pub fn has_tag(&self) -> bool {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        node.has_tag
    }
    pub fn tag_once(&self) -> bool {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        node.tag_once
    }
    // pub fn tag_valstr_key(&self) -> usize {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.tag_valstr_key
    // }
    pub fn label(&self) -> Option<&'a str> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        node.node_label.as_ref().map(|x|x.as_str())
    }

    pub fn param_group(&self, i:usize) -> Option<ParamGroupContainer<'a>> {
        let node=self.def.nodes.get(self.node_ind).unwrap();

        (i<node.param_groups.len()).then(||ParamGroupContainer {
            def:self.def,
            node_ind:self.node_ind,
            param_group_ind: i,
        })
    }

    pub fn param_groups_num(&self) -> usize {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        node.param_groups.len()
    }
    // pub fn get_param(&self,ind:usize) -> Option<(std::any::TypeId,ParamParse)> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params.get(ind).and_then(|x|x.clone())
    // }
    // pub fn param_parse(&self,i:usize,s:&str) -> Option<Box<dyn Any+Send+Sync>> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params.get(i).and_then(|x|*x).map(|x|x.2).and_then(|func|func(s))
    // }
    // pub fn param_parse_type_id(&self,i:usize) -> Option<TypeId> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params.get(i).and_then(|x|*x).map(|x|x.0)
    // }
    // pub fn param_parse_type_name(&self,i:usize) -> Option<&'static str> {
    //     let node=self.grammar.nodes.get(self.node_ind).unwrap();
    //     node.params.get(i).and_then(|x|*x).map(|x|x.1)
    // }
    // pub fn rsimilar(&self) -> bool {
    //     let node=self.def.nodes.get(self.node_ind).unwrap();
    //     node.rsimilar
    // }

}