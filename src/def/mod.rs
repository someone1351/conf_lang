pub mod container;
pub mod node;
pub mod branch;

use std::any::{Any, TypeId};
use std::collections::HashMap;

// use std::ops::{Bound, Range, RangeBounds};
use std::str::FromStr;

use branch::*;
use node::*;
use container::branch::*;


pub struct Def {
    branches : Vec<Branch>,
    branch_map : HashMap<String,usize>,
    nodes : Vec<Node>,

    for_tag_names : Vec<String>,
    cur_nodes_start : usize,
    tags_once:bool,


    // cur_repeat_start:Option<usize>,
    // cur_params_num:usize,
    // cur_optional:Option<usize>,
    // cur_group_param_start:Option<usize>,
}

impl Def {
    pub fn new() -> Self {
        Self {
            branches : Vec::new(),
            branch_map : HashMap::new(),
            nodes : Vec::new(),
            for_tag_names : Vec::new(),
            cur_nodes_start : 0,
            tags_once:false,
            // cur_repeat_start:None,
            // cur_params_num:0,
            // cur_group_param_start:None,
        }
    }


    pub fn branch(mut self, branch_name : &str) -> Self {
        if self.branch_map.contains_key(branch_name) { //panic or overwrite?
            panic!("Branch label already used.")
        }
        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, group not ended");
        // }
        //
        self.cur_nodes_start = self.nodes.len();
        let branch_index = self.branches.len();
        self.branches.push(Branch::new(branch_name.to_string()));
        self.branch_map.insert(branch_name.to_string(), branch_index);

        //
        self
    }

    pub fn insert_nodes(mut self, branch_name : &str) -> Self { //from branch_nodes_from
        // let cur_branch = self.cur_branch_mut();
        
        //
        if self.branches.len()==0 { //panic or do nothing?
            panic!("confdef, no branch available!");
        }
        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, group not ended");
        // }
        // if let Some(branch_rest_ind)=self.branch_rest_ind {
        //     if branch_rest_ind+1==self.branches.len() {
        //         panic!("confdef, cannot insert nodes into rest branch");
        //     }
        // }

        //
        let cur_branch = self.branches.last_mut().unwrap();
        cur_branch.branch_inserts.push(branch_name.to_string());
        self.cur_nodes_start = self.nodes.len();
        self.tags_once=false;
        self
    }
    
    pub fn tagless_nodes(mut self, ) -> Self {

        //
        if self.branches.len()==0 { //panic or do nothing?
            panic!("confdef, no branch available!");
        }
        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, group not ended");
        // }
        // if let Some(branch_rest_ind)=self.branch_rest_ind {
        //     if branch_rest_ind+1==self.branches.len() {
        //         panic!("confdef, cannot add nodes into rest branch");
        //     }
        // }

        //
        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();
        // self.for_tag_valstr_key=0;
        self.tags_once=false;

        // self.cur_params_num=0;
        // self.cur_repeat_start=None;

        self
    }

    fn inner_tag_nodes<'t,T>(mut self, tag_names: T, once:bool) -> Self 
    where
        T:IntoIterator<Item = &'t str>,
    {
        //
        if self.branches.len()==0 { //panic or do nothing?
            panic!("confdef, no branch available!");
        }

        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, group not ended");
        // }
        // if let Some(branch_rest_ind)=self.branch_rest_ind {
        //     if branch_rest_ind+1==self.branches.len() {
        //         panic!("confdef, cannot add nodes into rest branch");
        //     }
        // }

        //
        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();
        
        self.for_tag_names.extend(tag_names.into_iter().map(|x|x.to_string()));
        // self.for_tag_valstr_key = val_type_ind;

        self.tags_once=once;

        // self.cur_params_num=0;
        // self.cur_repeat_start=None;

        self
    }
    
    pub fn tag_nodes<'t,T>(self, tag_names: T) -> Self 
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names,false)
    }
    pub fn tag_nodes_once<'t,T>(self, tag_names: T) -> Self 
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names,true)
    }

    fn inner_entry(&mut self) {
        //adds a bunch of nodes for each tagname or if tagless a single node

        if self.branches.len()==0 { //panic or do nothing?
            panic!("confdef, no branch available!");
        }

        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, group not ended");
        // }

        //
        let branch_ind=self.branches.len()-1;
        let cur_branch = self.branches.last_mut().unwrap();


        //
        self.cur_nodes_start = self.nodes.len();
        // self.cur_params_num=0;
        // self.cur_repeat_start=None;
        // self.cur_group_param_start=None;

        if self.for_tag_names.len()==0 {
            let node_index = self.nodes.len();
            cur_branch.non_tags.push(node_index);

            self.nodes.push(Node{
                branch_ind,
                has_tag:false,
                tag_once:self.tags_once,
                param_groups:vec![ParamGroup::default()],
                .. Default::default()
            });
        } else {
            for tag_name in self.for_tag_names.iter() {
                let node_index = self.nodes.len();
                cur_branch.tags.entry(tag_name.clone()).or_default().push(node_index);

                self.nodes.push(Node{
                    branch_ind,has_tag:true,tag_once:self.tags_once,
                    param_groups:vec![ParamGroup::default()],
                    .. Default::default()
                });
            }
        }
    }

    pub fn entry(mut self) -> Self {
        self.inner_entry();
        self
    }
    pub fn rentry(mut self) -> Self {
        self.inner_entry();
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.rsimilar=true;
        }

        self
    }
    // pub fn priority(mut self,priority:i32) -> Self {
    //     if self.cur_nodes_start==self.nodes.len() {
    //         panic!("confdef, no nodes");
    //     }

    //     for node_index in self.cur_nodes_start .. self.nodes.len() {
    //         let node=self.nodes.get_mut(node_index).unwrap();
    //         let last_param_group=node.param_groups.last_mut().unwrap();

    //         // if last_param_group.greedy {
    //         //     panic!("confdef, greedy already set");
    //         // }

    //         last_param_group.priority=priority;
    //     }
        
    //     self
    // }

    pub fn optional(mut self) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }

        // if self.cur_params_num==0 {
        //     panic!("confdef, no param to make optional");
        // }

        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, cannot use optional inside group");
        // }
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            
            let node=self.nodes.get_mut(node_index).unwrap();

            // if node.params.is_empty() {
            //     panic!("confdef, no param to make optional");
            // }
            
            // Self::inner_init_param(node);

            let last_param_group=node.param_groups.last_mut().unwrap();

            if last_param_group.optional {
                panic!("confdef, optional already set");
            }

            last_param_group.optional=true;
        }

        self
    }

    pub fn repeat(mut self) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }

        // if self.cur_params_num==0 {
        //     panic!("confdef, no param to make repeat");
        // }
        
        // if self.cur_group_param_start.is_some() {
        //     panic!("confdef, cannot use repeat inside group");
        // }
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            
            let node=self.nodes.get_mut(node_index).unwrap();

            // Self::inner_init_param(node);
            // if node.params.is_empty() {
            //     panic!("confdef, no param to make repeat");
            // }

            let last_param_group=node.param_groups.last_mut().unwrap();

            if last_param_group.repeat {
                panic!("confdef, repeat already set");
            }

            last_param_group.repeat=true;
        }

        self
    }
    
    pub fn name(mut self, param_group_name:&str) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }

        // if self.cur_params_num==0 {
        //     panic!("confdef, no params");
        // }
                
        for node_index in self.cur_nodes_start .. self.nodes.len() {            
            let node=self.nodes.get_mut(node_index).unwrap();

            if node.param_groups.is_empty() {
                panic!("confdef, no param to name");
            }

            let last_param=node.param_groups.last_mut().unwrap();

            if last_param.name.is_some() {
                panic!("confdef, param already named");
            }

            last_param.name=Some(param_group_name.to_string());
        }

        self
    }

    pub fn group(mut self) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            
            if !node.param_groups.last().unwrap().specified { //.params.is_empty()
                // if !node.param_groups.last().unwrap().params.is_empty() { //not really needed, just force user to either use groups or not at all
                //     panic!("confdef, cannot add group when params added previously without group");
                // }

                //
                node.param_groups.pop().unwrap();
            }

            node.param_groups.push(ParamGroup{specified:true,..Default::default()});
        }

        // self.cur_params_num+=1;

        self
    }


    pub fn children(mut self, children_branch :&str) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();

            if node.children != NodeChildren::None {
                panic!("confdef, children already set");
            }
            
            node.children = NodeChildren::Branch(children_branch.to_string());
        }

        self
    }

    pub fn text(mut self, ) -> Self { //text_node_label : Option<&str>
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();

            if node.children != NodeChildren::None {
                panic!("confdef, children already set");
            }

            node.children = NodeChildren::Body(None); //text_node_label.map(|x|x.to_string())
        }

        self
    }

    pub fn label(mut self, node_label : &str) -> Self {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no nodes");
        }
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();

            if node.node_label.is_some() {
                panic!("confdef, label already set");
            }
            node.node_label=Some(node_label.to_string());
        }

        self
    }

    // pub fn rsimilar(mut self) -> Self {
    //     if self.cur_nodes_start==self.nodes.len() {
    //         panic!("confdef, no nodes");
    //     }

    //     for node_index in self.cur_nodes_start .. self.nodes.len() {
    //         let node=self.nodes.get_mut(node_index).unwrap();

    //         if node.rsimilar {
    //             panic!("confdef, rsimilar already set");
    //         }

    //         node.rsimilar=true;
    //     }

    //     self
    // }

    // fn inner_init_param(node:&mut Node) {
    //     if node.param_groups.is_empty() {
    //         node.param_groups.push(ParamGroup { optional: false, repeat: false, name: None, params: Vec::new() });
    //     }
    // }

    fn inner_add_param_item(&mut self,param_item:Param) {
        if self.cur_nodes_start==self.nodes.len() {
            panic!("confdef, no node to add param item to!");
        }
        
        //calc len of any repeating patterns in the param group eg (int bool int bool) => (int bool) => 2
        //bit inefficent to recalc everytime a param is added, should do it once they are all added
        let (pattern_len,patterns_num)={
            let mut params=self.nodes.last().unwrap()
                .param_groups.last().unwrap()
                .params.iter().map(|x|x.map(|x|x.0))
                .collect::<Vec<_>>();

            params.push(param_item.map(|x|x.0));

            //
            let mut ok=false;
            let mut pattern = vec![params.get(0).cloned().unwrap()];

            for param_ind in 1 .. params.len() {
                ok=true;

                if params.len()%pattern.len() !=0 {
                    pattern.push(params.get(param_ind).cloned().unwrap());
                    continue;
                }
        
                for x in 1 .. params.len()/pattern.len() {
                    let y=x*pattern.len();
                    let against_range=y..y+pattern.len();
                    let against=params.get(against_range).unwrap();
        
                    if !pattern.eq(against) {
                        ok=false;
                        break;
                    }
                }
        
                if ok {
                    break;
                }
        
                pattern.push(params.get(param_ind).cloned().unwrap());
            }
        
            if ok {
                (pattern.len(),params.len()/pattern.len())
            } else {
                (params.len(),1)
            }
        };

        //add param to nodes
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let param_group=node.param_groups.last_mut().unwrap();
            
            param_group.params.push(param_item);
            param_group.pattern_len=pattern_len;
            param_group.patterns_num=patterns_num;
        }
    }

    //if no param given to a tagless node, it will just have a node with no params, that will be skipped over during parsing
    
    
    pub fn any(mut self) -> Self { //str, any
        self.inner_add_param_item(None);
        self
    }

    pub fn parse<T>(mut self) -> Self
    where
        T:FromStr+Any+Send+Sync,
    {
        //std::any::type_name::<T>(),
        let func2=|s:&str|T::from_str(s).ok().map(|p|Box::new(p) as Box<dyn Any+Send+Sync>);
        let param_item:Param=Some((TypeId::of::<T>(),std::any::type_name::<T>(),func2));
        self.inner_add_param_item(param_item);
        
        self
    }
    
    pub fn get_branch(&self, branch_name : &str) -> Option<BranchContainer> {
        self.branch_map.get(branch_name).map(|&branch_ind|BranchContainer {
            def:self,
            branch_ind,
        })
    }

}

