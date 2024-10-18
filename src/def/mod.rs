pub mod container;
pub mod node;
pub mod branch;

use std::any::{Any, TypeId};
use std::collections::HashMap;

use std::str::FromStr;

use branch::*;
use node::*;
use container::branch::*;

pub struct Def {
    branches : Vec<Branch>,
    branch_map : HashMap<String,usize>, //Option<String>
    nodes : Vec<Node>,
    
    params : Vec<Param>,

    cur_branch_ind : usize,
    for_tag_names : Vec<String>,
    cur_nodes_start : usize,
    tags_once:bool,
}

impl Def {
    pub fn new() -> Self {
        Self {
            branches : Vec::new(),
            branch_map : HashMap::new(),
            nodes : Vec::new(),
            params : Vec::new(),

            cur_branch_ind:0,
            for_tag_names : Vec::new(),
            cur_nodes_start : 0,
            tags_once:false,
        }
    }

    pub fn get_root_branch(&self) -> BranchContainer {
        BranchContainer {
            def:self,
            branch_ind:0,
        }
    }

    // pub fn get_branch(&self, branch_name : &str) -> Option<BranchContainer> {
    //     self.branch_map.get(branch_name).map(|&branch_ind|BranchContainer {
    //         def:self,
    //         branch_ind,
    //     })
    // }

    pub fn get_branch(&self, branch_name : &str) -> BranchContainer {
        BranchContainer {
            def:self,
            branch_ind:self.branch_map.get(branch_name).cloned().unwrap_or(self.branches.len()),
        }

    }
    pub fn branch(mut self, branch_name : &str) -> Self {
        if let Some(branch_ind)=self.branch_map.get(branch_name).cloned() {
            self.cur_branch_ind=branch_ind;
        } else {
            // let branch_index = self.branches.len();
            self.cur_branch_ind=self.branches.len();
            self.branches.push(Branch::new(Some(branch_name.to_string())));
            self.branch_map.insert(branch_name.to_string(), self.cur_branch_ind);
        }

        //
        self.cur_nodes_start = self.nodes.len();
        self.for_tag_names.clear();
        self.tags_once=false;

        //
        self
    }

    pub fn insert_nodes(mut self, branch_name : &str) -> Self { //from branch_nodes_from
        //
        if self.branches.is_empty() {
            self.branches.push(Default::default());
        }

        //
        // let cur_branch = self.branches.last_mut().unwrap();
        let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();        

        cur_branch.branch_inserts.push(branch_name.to_string());
        self.cur_nodes_start = self.nodes.len();
        self.tags_once=false;

        //
        self
    }
    
    pub fn tagless_nodes(mut self, ) -> Self {
        //
        if self.branches.is_empty() {
            self.branches.push(Default::default());
        }

        //
        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();
        self.tags_once=false;

        //
        self
    }

    fn inner_tag_nodes<'t,T>(&mut self, tag_names: T, once:bool)
    where
        T:IntoIterator<Item = &'t str>,
    {
        //
        if self.branches.is_empty() {
            self.branches.push(Default::default());
        }
        
        //
        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();        
        self.for_tag_names.extend(tag_names.into_iter().map(|x|x.to_string()));
        self.tags_once=once;

    }

    pub fn tag_nodes<'t,T>(mut self, tag_names: T) -> Self 
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names,false);
        self
    }

    pub fn tag_nodes_once<'t,T>(mut self, tag_names: T) -> Self 
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names,true);
        self
    }

    fn inner_entry(&mut self) {
        //adds a bunch of nodes for each tagname or if tagless a single node

        //
        if self.branches.is_empty() {
            self.branches.push(Default::default());
        }

        //
        // let branch_ind=self.branches.len()-1;
        // let cur_branch = self.branches.last_mut().unwrap();
        let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();     

        //
        self.cur_nodes_start = self.nodes.len();

        //
        if self.for_tag_names.len()==0 {
            let node_index = self.nodes.len();
            cur_branch.non_tags.push(node_index);

            self.nodes.push(Node{
                branch_ind:self.cur_branch_ind,
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
                    branch_ind:self.cur_branch_ind,
                    has_tag:true,
                    tag_once:self.tags_once,
                    param_groups:vec![ParamGroup::default()],
                    .. Default::default()
                });
            }
        }
    }

    pub fn entry(mut self,
        label : Option<&str>,
    ) -> Self {
        self.inner_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.node_label=label.map(|x|x.to_string());
        }

        self
    }

    pub fn entry_children(mut self,
        label : Option<&str>,
        children : &str,
    ) -> Self {
        self.inner_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.node_label=label.map(|x|x.to_string());

            node.children = NodeChildren::Branch(children.to_string());
        }

        self
    }
    pub fn entry_text(mut self,
        label : Option<&str>,
    ) -> Self {
        self.inner_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.node_label=label.map(|x|x.to_string());
            node.children = NodeChildren::Body(None);
        }

        self
    }
    
    pub fn rentry(mut self,
        label : Option<&str>,
    ) -> Self {
        self.inner_entry();
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.rsimilar=true;
            node.node_label=label.map(|x|x.to_string());
        }

        self
    }

    pub fn rentry_children(mut self,
        label : Option<&str>,
        children : &str,
    ) -> Self {
        self.inner_entry();
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.rsimilar=true;
            node.node_label=label.map(|x|x.to_string());
            node.children = NodeChildren::Branch(children.to_string());
        }

        self
    }

    pub fn rentry_text(mut self,
        label : Option<&str>,
    ) -> Self {
        self.inner_entry();
        
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.rsimilar=true;
            node.node_label=label.map(|x|x.to_string());
            node.children = NodeChildren::Body(None);
        }

        self
    }

    pub fn group(mut self,
        name:Option<&str>,
        optional:bool,
        repeat:bool,    
    ) -> Self {
        //add node if there are none set
        if self.cur_nodes_start==self.nodes.len() {
            self.inner_entry();
        }

        //
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            
            if !node.param_groups.last().unwrap().specified {
                node.param_groups.pop().unwrap();
            }

            node.param_groups.push(ParamGroup{specified:true,..Default::default()});

            let last_param_group=node.param_groups.last_mut().unwrap();

            last_param_group.repeat=repeat;
            last_param_group.optional=optional;
            last_param_group.name=name.map(|x|x.to_string());
        }

        self
    }

    fn inner_add_param_item(&mut self,param_item:Option<Param>) {        
        //add node if there are none set
        if self.cur_nodes_start==self.nodes.len() {
            self.inner_entry();
        }

        //
        let param_item_ind= param_item.map(|x|{
            let param_ind=self.params.len();
            self.params.push(x);
            param_ind
        });

        let param_item_type=param_item_ind.map(|param_ind|self.params.get(param_ind).unwrap().0);

        //calc len of any repeating patterns in the param group eg (int bool int bool) => (int bool) => 2
        //bit inefficent to recalc patterns everytime a param is added, should instead do it once they are all added
        let (pattern_len,patterns_num)={
            let mut param_types=self.nodes.last().unwrap()
                .param_groups.last().unwrap()
                // .params.iter().map(|x|x.map(|x|x.0))
                // .params.iter().map(|x|x.as_ref().map(|x|x.0))
                .params.iter().map(|&x|x.map(|param_ind|self.params.get(param_ind).unwrap().0))
                
                .collect::<Vec<_>>();

            // param_types.push(param_item.as_ref().map(|x|x.0));
            param_types.push(param_item_type);
            
            //
            let mut ok=false;
            let mut pattern = vec![param_types.get(0).cloned().unwrap()];

            for param_ind in 1 .. param_types.len() {
                ok=true;

                if param_types.len()%pattern.len() !=0 {
                    pattern.push(param_types.get(param_ind).cloned().unwrap());
                    continue;
                }
        
                for x in 1 .. param_types.len()/pattern.len() {
                    let y=x*pattern.len();
                    let against_range=y..y+pattern.len();
                    let against=param_types.get(against_range).unwrap();
        
                    if !pattern.eq(against) {
                        ok=false;
                        break;
                    }
                }
        
                if ok {
                    break;
                }
        
                pattern.push(param_types.get(param_ind).cloned().unwrap());
            }
        
            if ok {
                (pattern.len(),param_types.len()/pattern.len())
            } else {
                (param_types.len(),1)
            }
        };

        //add param to nodes
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let param_group=node.param_groups.last_mut().unwrap();
            
            // param_group.params.push(param_item);
            param_group.params.push(param_item_ind);
            param_group.pattern_len=pattern_len;
            param_group.patterns_num=patterns_num;
        }
    }

    pub fn param_any(mut self) -> Self { //str, any
        self.inner_add_param_item(None);
        self
    }

    pub fn param_parse<T>(mut self) -> Self
    where
        T:FromStr+Any+Send+Sync,
    {
        let func2=|s:&str|T::from_str(s).ok().map(|p|Box::new(p) as Box<dyn Any+Send+Sync>);
        let param_item:Option<Param>=Some((TypeId::of::<T>(),std::any::type_name::<T>(),Box::new(func2)));
        self.inner_add_param_item(param_item);
        
        self
    }
    
    pub fn param_func<T,F>(mut self, func:F) -> Self
    where
        T:Any+Send+Sync,
        F:Fn(&str)->Option<T>+'static

    {
        let func2=move|s:&str|func(s).map(|p|Box::new(p) as Box<dyn Any+Send+Sync>);
        let param_item:Option<Param>=Some((TypeId::of::<T>(),std::any::type_name::<T>(),Box::new(func2)));
        self.inner_add_param_item(param_item);
        
        self
    }
}