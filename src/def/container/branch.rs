
use std::collections::HashSet;
use std::path::Path;

use super::super::super::Conf;
use super::super::super::ParseError;
use super::super::super::parser::parse_start;
use super::super::Def;


// use super::super::grammar::Grammar;
use super::node_iter::NodeContainerIter;

#[derive (Clone,Copy)]
pub struct BranchContainer<'a> {
    pub(in super::super) def:&'a Def,
    pub(in super::super) branch_ind:usize,
}


impl<'a> BranchContainer<'a> {
    pub fn get_tag_nodes<'t>(&self,tag_name:&'t str) -> NodeContainerIter::<'a,'t> {
        let branch=self.def.branches.get(self.branch_ind).unwrap();
        
        NodeContainerIter {
            def:&self.def,
            tag_name:Some(tag_name),
            visited_branches:HashSet::new(),
            to_visit_branches:vec![branch.branch_name.as_str()],
            branch_ind:None,
            branch_node_ind:0,
        }
    }

    pub fn get_tagless_nodes(&self) -> NodeContainerIter::<'a,'_> {
        let branch=self.def.branches.get(self.branch_ind).unwrap();

        NodeContainerIter {
            def:&self.def,
            tag_name:None,
            visited_branches:HashSet::new(),
            to_visit_branches:vec![branch.branch_name.as_str()],
            branch_ind:None,
            branch_node_ind:0,
        }
    }

    pub fn name(&self) -> &'a str {
        let branch=self.def.branches.get(self.branch_ind).unwrap();
        branch.branch_name.as_str()
    }
    pub fn branch_ind(&self) -> usize {
        self.branch_ind
    }
    pub fn parse<'b>(
        &self,
        // walk_branch:BranchContainer,
        src : &'b str, 
        keep_src:bool,
        path:Option<&'b Path>,
    ) -> Result<Conf,ParseError> {
    
        parse_start(*self,src,keep_src,path)
    }
}
