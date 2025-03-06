
use std::any::{Any, TypeId};

use crate::def::GroupSimilar;

use super::super::Def;



#[derive (Clone,Copy)]
pub struct ParamGroupContainer<'a> {
    pub(in super::super) def:&'a Def,
    pub(in super::super) node_ind:usize,
    pub(in super::super) param_group_ind:usize,
}

impl<'a> ParamGroupContainer<'a> {
    pub fn repeat(&self) -> bool {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.repeat
    }
    pub fn optional(&self) -> bool {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.optional
    }
    pub fn param_optional(&self) -> Option<usize> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.param_optional
    }

    // pub fn param_group(&self) -> bool {
    //     let node=self.def.nodes.get(self.node_ind).unwrap();
    //     let param_group=node.params.get(self.param_ind).unwrap();
    //     param_group.param_group
    // }

    pub fn name(&self) -> Option<&'a str> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.name.as_ref().map(|x|x.as_str())
    }

    pub fn params_num(&self) -> usize {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.params.len()
    }

    pub fn param_run(&self,item_ind:usize,val:&str) -> Option<Box<dyn Any+Send+Sync>> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        let param=param_group.params.get(item_ind);

        // param.and_then(|x|*x).map(|x|x.2).and_then(|func|func(val))
        // param.and_then(|x|x.as_ref()).map(|x|x.2.as_ref()).and_then(|func|func(val))

        //what about the param any, ie None, no func? any's aren't checked from here
        // println!("= {val:?} {} {} {}",
        //     param.and_then(|param_ind|*param_ind).is_some(),
        //     param.and_then(|param_ind|*param_ind).map(|param_ind|self.def.params.get(param_ind).unwrap().2.as_ref()).is_some(),
        //     param.and_then(|param_ind|*param_ind).map(|param_ind|self.def.params.get(param_ind).unwrap().2.as_ref()).and_then(|func|func(val)).is_some(),
        // );

        param
            .and_then(|param_ind|*param_ind)
            .map(|param_ind|self.def.params.get(param_ind).unwrap().2.as_ref())
            .and_then(|func|func(val))
    }

    pub fn param_type_id(&self,item_ind:usize) -> Option<TypeId> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        let param=param_group.params.get(item_ind);

        // param.and_then(|x|*x).map(|x|x.0)
        // param.and_then(|x|x.as_ref()).map(|x|x.0)
        param.and_then(|x|*x).map(|param_ind|self.def.params.get(param_ind).unwrap().0)
    }
    pub fn param_type_name(&self,item_ind:usize) -> Option<&'static str> {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        let param=param_group.params.get(item_ind);

        // param.and_then(|x|*x).map(|x|x.1)
        // param.and_then(|x|x.as_ref()).map(|x|x.1)
        param.and_then(|x|*x).map(|param_ind|self.def.params.get(param_ind).unwrap().1)

    }

    // pub fn params_eq(&self,other:ParamGroupContainer) -> bool {
    //     if self.params_num()!=other.params_num() {
    //         return false;
    //     }

    //     for i in 0 ..self.params_num() {
    //         if !self.param_type_id(i).eq(&other.param_type_id(i)) {
    //             return false;
    //         }
    //     }

    //     true
    // }
    pub fn params_pattern_len(&self) -> usize {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.pattern_len
    }
    pub fn params_patterns_num(&self) -> usize {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.pattern_many
    }
    // pub fn specified(&self) -> bool {
    //     let node=self.def.nodes.get(self.node_ind).unwrap();
    //     let param_group=node.param_groups.get(self.param_group_ind).unwrap();
    //     // param_group.specified
    // }

    pub fn similar(&self) -> GroupSimilar {
        let node=self.def.nodes.get(self.node_ind).unwrap();
        let param_group=node.param_groups.get(self.param_group_ind).unwrap();
        param_group.similar
    }
}
