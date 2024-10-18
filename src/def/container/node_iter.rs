use std::collections::HashSet;

use super::node::NodeContainer;
// use super::super::grammar::Grammar;
use super::super::Def;

// #[derive (Clone)]
pub struct NodeContainerIter<'a,'t> {
    pub(in super::super) def:&'a Def,
    pub(in super::super) tag_name : Option<&'t str>,

    pub(in super::super) visited_branches:HashSet<Option<&'a str>>,
    pub(in super::super) to_visit_branches:Vec<Option<&'a str>>,

    pub(in super::super) branch_ind:Option<usize>,
    pub(in super::super) branch_node_ind:usize,
}

impl<'a,'t> Iterator for NodeContainerIter<'a,'t> {
    type Item = NodeContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.def.branches.is_empty() {
            return None;
        }

        //if branch name doesn't exist, should just skip over it, instead of error

        loop {
            if let Some(branch_ind)=self.branch_ind {
                let branch=self.def.branches.get(branch_ind).unwrap();

                if let Some(tag_name)=self.tag_name {
                    if let Some(branch_nodes)=branch.tags.get(tag_name) {
                        if self.branch_node_ind==branch_nodes.len() {
                            self.branch_ind=None;
                        } else {
                            let node_ind=branch_nodes.get(self.branch_node_ind).cloned().unwrap();
                            self.branch_node_ind+=1;

                            return Some(NodeContainer {
                                def:self.def,
                                // branch_ind,
                                node_ind,
                            });
                        }
                    } else {
                        self.branch_ind=None;
                    }
                } else {
                    let branch_nodes = &branch.non_tags;

                    if self.branch_node_ind==branch_nodes.len() {
                        self.branch_ind=None;
                    } else {
                        let node_ind=branch_nodes.get(self.branch_node_ind).cloned().unwrap();
                        self.branch_node_ind+=1;

                        return Some(NodeContainer {
                            def:self.def,
                            // branch_ind,
                            node_ind,
                        });
                    }
                }
            } else if let Some(branch_name)=self.to_visit_branches.pop() {
                if !self.visited_branches.contains(&branch_name) {    
                    if let Some(branch_ind)=branch_name.map_or(Some(0), |n|self.def.branch_map.get(n).cloned()){
                        let branch=self.def.branches.get(branch_ind).unwrap();
                        self.to_visit_branches.extend(branch.branch_inserts.iter().map(|x|Some(x.as_str())));
                        self.branch_ind=Some(branch_ind);
                        self.branch_node_ind=0;
                    } else {
                        self.branch_ind=None;
                    }

                    self.visited_branches.insert(branch_name);
                }
            } else {
                return None;
            }
        }
    }
}