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
    is_group_similar : bool,
}

impl Def {
    pub fn new() -> Self {
        Self {
            branches : Vec::new(),
            // branches:vec![Branch{ tags: todo!(), non_tags: todo!(), branch_inserts: todo!(), branch_name: todo!() }]
            branch_map : HashMap::new(),
            nodes : Vec::new(),
            params : Vec::new(),

            cur_branch_ind:0,
            for_tag_names : Vec::new(),
            cur_nodes_start : 0,
            tags_once:false,
            is_group_similar:false,
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

    fn init_branch(&mut self) {
        if self.branches.is_empty() {
            self.branches.push(Default::default());
        }
    }

    pub fn include<'a,B>(mut self, branch_names : B) -> Self
    where
        B :IntoIterator<Item = &'a str>
    { //from branch_nodes_from
        //
        self.init_branch();

        //
        // let cur_branch = self.branches.last_mut().unwrap();
        let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();

        for branch_name in branch_names.into_iter() {
            cur_branch.branch_inserts.push(branch_name.to_string());
        }

        // cur_branch.branch_inserts.push(branch_name.to_string());
        self.cur_nodes_start = self.nodes.len(); //why?
        self.tags_once=false; //why?

        //
        self
    }

    fn inner_tagless_nodes(&mut self) {
        self.init_branch();

        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();
        self.tags_once=false;
    }

    pub fn tagless(mut self, ) -> Self {
        //
        self.inner_tagless_nodes();

        //
        self
    }

    fn inner_tag_nodes<'t,T>(&mut self, tag_names: T,
        once:bool
    )
    where
        T:IntoIterator<Item = &'t str>,
    {
        //
        self.init_branch();

        //
        self.cur_nodes_start = self.nodes.len(); //makes modifying a node an error if each hasnt been called?
        self.for_tag_names.clear();
        self.for_tag_names.extend(tag_names.into_iter().map(|x|x.to_string()));
        self.tags_once=once;

    }

    pub fn tags<'t,T>(mut self, tag_names: T) -> Self
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names, false );
        self
    }

    pub fn tags_once<'t,T>(mut self, tag_names: T) -> Self
    where
        T:IntoIterator<Item = &'t str>,
    {
        self.inner_tag_nodes(tag_names,true);
        self
    }

    // pub fn tags_once(mut self) -> Self {
    //     self.tags_once=true;
    //     self
    // }

    fn inner_entry_tagless(&mut self) {
        self.init_branch();
        let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();
        self.cur_nodes_start = self.nodes.len();

        let node_index = self.nodes.len();
        cur_branch.non_tags.push(node_index);

        self.nodes.push(Node{
            branch_ind:self.cur_branch_ind,
            has_tag:false,
            tag_once:self.tags_once,
            // param_groups:vec![ParamGroup::default()],
            param_groups:Vec::new(),
            .. Default::default()
        });
    }

    fn inner_entry_tags(&mut self) {
        self.init_branch();
        let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();
        self.cur_nodes_start = self.nodes.len();

        for tag_name in self.for_tag_names.iter() {
            let node_index = self.nodes.len();
            cur_branch.tags.entry(tag_name.clone()).or_default().push(node_index);

            self.nodes.push(Node{
                branch_ind:self.cur_branch_ind,
                has_tag:true,
                tag_once:self.tags_once,
                // param_groups:vec![ParamGroup::default()],
                param_groups:Vec::new(),
                .. Default::default()
            });
        }
    }
    fn inner_entry(&mut self) {
        //adds a bunch of nodes for each tagname or if tagless a single node

        //
        // self.init_branch();

        //
        // // let branch_ind=self.branches.len()-1;
        // // let cur_branch = self.branches.last_mut().unwrap();
        // let cur_branch = self.branches.get_mut(self.cur_branch_ind).unwrap();

        //
        // self.cur_nodes_start = self.nodes.len();

        //
        if self.for_tag_names.len()==0 {
            self.inner_entry_tagless();
        } else {
            self.inner_entry_tags();
        }
    }

    pub fn entry(mut self) -> Self { //entry
        self.inner_entry();
        self
    }

    //would like to make this an array of branches that can be used as children, but no idea on how to modify code in the parser to handle that
    pub fn entry_children(mut self,children : &str) -> Self { //centry
        self.inner_entry();
        // self.init_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.children = NodeChildren::Branch(children.to_string());
        }

        self
    }
    pub fn entry_text(mut self) -> Self { //tentry
        self.inner_entry();
        // self.init_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.children = NodeChildren::Body(None);
        }

        self
    }

    fn init_entry(&mut self) {
        self.init_branch();

        if self.cur_nodes_start==self.nodes.len() {
            self.inner_entry_tagless();
        }
    }

    pub fn elabel(mut self,label : &str) -> Self {
        self.init_entry();

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.node_label=Some(label.to_string());
        }

        self
    }

    fn inner_group(&mut self, group_similar:GroupSimilar) {
        self.init_entry();

        self.is_group_similar=group_similar!=GroupSimilar::None;

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            node.param_groups.push(ParamGroup{similar: group_similar,..Default::default()});
        }
    }

    pub fn group(mut self) -> Self {
        self.inner_group(GroupSimilar::None);
        self
    }

    pub fn group_left(mut self) -> Self {
        self.inner_group(GroupSimilar::Left);
        self
    }

    pub fn group_right(mut self) -> Self {
        self.inner_group(GroupSimilar::Right);
        self
    }

    fn init_group(&mut self) {
        self.init_entry();

        if self.cur_nodes_start == self.nodes.len() {
            self.is_group_similar=false;
        }

        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();

            if node.param_groups.is_empty() {
                node.param_groups.push(ParamGroup{similar:GroupSimilar::None,..Default::default()});
            }
        }
    }

    pub fn glabel(mut self,name:&str) -> Self {
        //add node if there are none set
        // if self.cur_nodes_start==self.nodes.len() {
        //     self.inner_entry();
        // }
        self.init_group();

        //
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let last_group=node.param_groups.last_mut().unwrap();
            // last_group.specified =true;
            last_group.name=Some(name.to_string());
        }

        self
    }
    pub fn grepeat(mut self) -> Self {
        //add node if there are none set
        // if self.cur_nodes_start==self.nodes.len() {
        //     self.inner_entry();
        // }
        self.init_group();

        //
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let last_group=node.param_groups.last_mut().unwrap();
            // last_group.specified =true;
            last_group.repeat=true;
        }

        self
    }
    pub fn goptional(mut self,
        // from:Option<usize>,
    ) -> Self {
        //add node if there are none set
        // if self.cur_nodes_start==self.nodes.len() {
        //     self.inner_entry();
        // }
        self.init_group();

        //
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let last_group=node.param_groups.last_mut().unwrap();
            // last_group.specified =true;

            // if let Some(from)=from {
            //     last_group.param_optional = Some(from);
            // } else {
                last_group.optional=true;
            // }


        }

        self
    }

    fn inner_param_pattern(&mut self, param_item_ind: Option<usize>) -> (usize,usize) {
        //also should be when last_group.param_optional.is_some(),
        //  but is done in parser, maybe can impl group_similar for param_optional cases,
        //  though not needed
        if !self.is_group_similar {
            return (0,0);
        }

        let param_item_type=param_item_ind.map(|param_ind|self.params.get(param_ind).unwrap().0);

        //calc len of any repeating patterns in the param group eg (int bool int bool) => (int bool) => 2
        //bit inefficent to recalc patterns everytime a param is added, should instead do it once they are all added

        //need to break pattern if there is a param_optional? or have no pattern at all?

        let (pattern_len,pattern_many)={ //pattern_len aka pattern_ssstuple_len
            let last_group = self.nodes.last().unwrap().param_groups.last().unwrap();

            // if last_group.param_optional.is_some() { //if theres a optional param, don't use patterns
            //     //should set patterns_num to 0? so in parser is skipped over?
            //     //  check in parser if pattern_num is 0 instead of params_optional.is_none()?
            //     (last_group.params.len()+1,1)
            // } else
            // {

            // }
            let mut param_types=last_group
                // .params.iter().map(|x|x.map(|x|x.0))
                // .params.iter().map(|x|x.as_ref().map(|x|x.0))
                .params.iter().map(|&x|x.map(|param_ind|self.params.get(param_ind).unwrap().0)).collect::<Vec<_>>();

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

        (pattern_len,pattern_many)
    }

    fn inner_add_param_item(&mut self,param_item:Option<Param>) {
        //add node if there are none set
        // if self.cur_nodes_start==self.nodes.len() {
        //     self.inner_entry();
        // }
        self.init_group();

        //
        //any is param_item==None, so here it doesn't get added to def.params?
        //  no later the None gets added to param_group.params
        let param_item_ind: Option<usize>= param_item.map(|x|{
            let param_ind=self.params.len();
            self.params.push(x);
            param_ind
        });

        //
        let (pattern_len,pattern_many)=self.inner_param_pattern(param_item_ind);


        //add param to nodes
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let param_group=node.param_groups.last_mut().unwrap();

            // println!("ggg {} {:?}",self.is_group_similar,param_group.similar);
            // param_group.params.push(param_item);
            param_group.params.push(param_item_ind);
            param_group.pattern_len=pattern_len;
            param_group.pattern_many=pattern_many;
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

    pub fn param_optional(mut self) -> Self {
        //add node if there are none set
        // if self.cur_nodes_start==self.nodes.len() {
        //     self.inner_entry();
        // }
        self.init_group();

        //
        for node_index in self.cur_nodes_start .. self.nodes.len() {
            let node=self.nodes.get_mut(node_index).unwrap();
            let param_group=node.param_groups.last_mut().unwrap();

            if param_group.param_optional.is_none() {
                param_group.param_optional = Some(param_group.params.len());
            }
        }

        self
    }
}