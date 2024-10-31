pub mod error;
use std::any::Any;
use std::rc::Rc;
use std::{collections::HashSet, fmt::Debug, path::Path};

use error::{WalkError,WalkErrorType};

use super::conf::container::record::RecordContainer;
// use super::lexer::Loc;

//should replace children and values in record with index range into vecs stored in conf
//walk shouldnt have a single root, rather an array of roots
//  that would be nice, but hhow to provide an iter for that, when only have ChildIter
//  coould replace ChildIter with a general iter, especially if they are all stored in a single array, with siblings adjacent to each other
//  but for now just have a root, and its children starting at depth 1
// probblem with storiing records adjacent to their siblings, is when generating, can't do that initially, since  don't know who are going to be siblings
//    will have to reorganise, but wouldn't that be slow?
//        maybe not too bad if moving completed siblings/families towards start of vec,

//instead of generating a whole new tree for walk, could just have a structure that contains the changes/insssertions done to the conf tree
//  and if the walk tree is also walked, have it document the changes to the walk tree and so on
//  so only need to document insertions
//  also solves problem of storing siblings adjacent in walk
//  handling order will be a pain, depth is ok
//  stored info on modified records children, storing ranges of un modified, and then modified, the modified containing extra info eg of other conf and record ind range
//  in recordcontainer for walk inserrted record, can store parent info
//






#[derive(Clone)]
pub struct WalkFrom<'a> {
    record:RecordContainer<'a>,
    custom:Option<Rc<dyn Any>>,
}

impl<'a> WalkFrom<'a> {
    pub fn record(&self) -> RecordContainer<'a> {
        self.record
    }
    pub fn custom<T:Any+Clone>(&self) -> Option<T> {
        self.custom.as_ref().and_then(|x|x.downcast_ref::<T>().map(|x|x.clone()))
    }
}

#[derive(Clone)]
pub struct WalkFromIter<'b,'a> {
    pub(super) froms : &'b Vec<WalkFrom<'a>>,
    pub(super) start : usize,
    pub(super) end : usize,
}


impl<'b,'a> Iterator for WalkFromIter<'b,'a> {
    type Item = &'b WalkFrom<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start==self.end {
            None
        } else {
            let ind=self.start;
            self.start+=1;
            Some(self.froms.get(ind).unwrap())
        }
    }
}

impl<'b,'a> DoubleEndedIterator for WalkFromIter<'b,'a> {
    fn next_back(&mut self) -> Option<&'b WalkFrom<'a>> {
        if self.end > self.start {
            self.end-=1;
            Some(self.froms.get(self.end).unwrap())
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct WalkAncestorIter<'b,'a> {
    pub(super) ancestors : &'b Vec<RecordContainer<'a>>,
    pub(super) start : usize,
    pub(super) end : usize,
}


impl<'b,'a> Iterator for WalkAncestorIter<'b,'a> {
    type Item = RecordContainer<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start==self.end {
            None
        } else {
            let ind=self.start;
            self.start+=1;
            Some(*self.ancestors.get(ind).unwrap())
        }
    }
}

impl<'b,'a> DoubleEndedIterator for WalkAncestorIter<'b,'a> {
    fn next_back(&mut self) -> Option<RecordContainer<'a>> {
        if self.end > self.start {
            self.end-=1;
            Some(*self.ancestors.get(self.end).unwrap())
        } else {
            None
        }
    }
}

pub struct Walk<'b,'a> {
    record:RecordContainer<'a>,
    depth:usize,
    order:usize,
    exit:bool,
    ancestors : &'b Vec<RecordContainer<'a>>,

    skip_children : &'b mut bool,
    sibling_extends : &'b mut Vec<WalkFrom<'a>>,
    child_extends : &'b mut Vec<WalkFrom<'a>>,
    // skip_exit : &'b mut bool,
    have_exit : &'b mut bool,

    
    froms : &'b Vec<WalkFrom<'a>>,
}

impl<'b,'a> Walk<'b,'a> {
    pub fn error<E:Debug>(&self,e:E) -> WalkError<E> {
        WalkError { path: self.record().path().map(|p|p.to_path_buf()), loc: self.record().start_loc(), error_type: WalkErrorType::Custom(e) }
    }
    pub fn record(&self) -> RecordContainer<'a> {
        self.record
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn is_enter(&self) -> bool {
        !self.exit
    }
    pub fn is_exit(&self) -> bool {
        self.exit
    }
    // pub fn ancestors(&self) -> std::slice::Iter<RecordContainer<'a>> {
    //     self.ancestors.iter().rev()
    // }

    pub fn froms_num(&self) -> usize {
        self.froms.len()
    }

    // pub fn from(&self,ind:usize) -> RecordContainer<'a> {
    //     if self.froms.is_empty() || ind >=self.froms.len() {
    //         Default::default()
    //     } else {
    //         self.froms.get(self.froms.len()-ind-1).cloned().unwrap()
    //     }
    // }
    pub fn get_from(&self,ind:usize) -> Option<&'b WalkFrom<'a>> {
        if self.froms.is_empty() || ind >=self.froms.len() {
            None
        } else {
            Some(self.froms.get(self.froms.len()-ind-1).unwrap()) //.cloned()
        }
    }

    pub fn froms(&self) -> WalkFromIter<'b,'a> {
        WalkFromIter {
            froms: self.froms,
            start: 0,
            end: self.froms.len(),
        }
    }
    
    pub fn ancestors_num(&self) -> usize {
        self.ancestors.len()
    }

    pub fn ancestor(&self,ind:usize) -> RecordContainer<'a> {
        if self.ancestors.is_empty() || ind >=self.ancestors.len() {
            Default::default()
        } else {
            self.ancestors.get(self.ancestors.len()-ind-1).cloned().unwrap()
        }
    }
    pub fn get_ancestor(&self,ind:usize) -> Option<RecordContainer<'a>> {
        if self.ancestors.is_empty() || ind >=self.ancestors.len() {
            None
        } else {
            Some(self.ancestors.get(self.ancestors.len()-ind-1).cloned().unwrap())
        }
    }
    
    pub fn ancestors(&self) -> WalkAncestorIter<'b,'a> {
        WalkAncestorIter {
            ancestors: self.ancestors,
            start: 0,
            end: self.ancestors.len(),
        }
    }
    
    pub fn parent(&self) -> RecordContainer<'a> {
        if self.ancestors.is_empty() {
            Default::default()
        } else {
            self.ancestors.last().cloned().unwrap()
        }
    }
    pub fn get_parent(&self) -> Option<RecordContainer<'a>> {
        if self.ancestors.is_empty() {
            None
        } else {
            Some(self.ancestors.last().cloned().unwrap())
        }
    }
    pub fn has_parent(&self) -> bool {
        !self.ancestors.is_empty()
    }

    pub fn skip_children(&mut self) {
        *self.skip_children=true;
    }

    // pub fn insert(&mut self, record : RecordContainer<'a>) {
    //     self.sibling_inserts.push(record);
    // }

    // pub fn extend<I>(&mut self, records : I) 
    // where
    //     I : IntoIterator<Item=RecordContainer<'a>>
    // {
    //     self.sibling_inserts.extend(records);
    // }

    // pub fn insert_child(&mut self, record : RecordContainer<'a>) {
    //     self.child_inserts.push(record);
    // }
    
    // pub fn extend_children<I>(&mut self, records : I) 
    // where
    //     I : IntoIterator<Item=RecordContainer<'a>>
    // {
    //     self.child_inserts.extend(records);
    // }

    
    pub fn extend(&mut self, from_record : RecordContainer<'a>) {
        self.sibling_extends.push(WalkFrom { record: from_record, custom: None });
    }

    pub fn extend_children(&mut self, from_record : RecordContainer<'a>)  {
        self.child_extends.push(WalkFrom { record: from_record, custom: None });
    }

    pub fn extend_custom<T:Any>(&mut self, from_record : RecordContainer<'a>, custom:T) {
        self.sibling_extends.push(WalkFrom { record: from_record, custom: Some(Rc::new(custom)) });
    }

    pub fn extend_children_custom<T:Any>(&mut self, from_record : RecordContainer<'a>, custom:T)  {
        self.child_extends.push(WalkFrom { record: from_record, custom: Some(Rc::new(custom)) });
    }

    // pub fn skip_exit(&mut self) {
    //     *self.skip_exit=true;
    // }

    pub fn have_exit(&mut self) {
        *self.have_exit=true;
    }
}

struct Work<'a> {
    record:RecordContainer<'a>,
    depth:usize,
    exit:bool,
    exit_order:usize,
    walk_parent:Option<RecordContainer<'a>>,
    visiteds:HashSet<(Option<&'a Path>, usize)>,
    // include_origin:Option<RecordContainer<'a>>,
    
    froms : Vec<WalkFrom<'a>>,
}

pub fn traverse<'a,E:Debug>(
    root_record : RecordContainer<'a>, 
    mut callback : impl for<'b> FnMut(Walk<'b,'a>) -> Result<(),
        // E //(E,Option<Loc>)
        WalkError<E>
        >,
) -> Result<(),WalkError<E>> {

    let mut walk_ancestors=Vec::new();
    
    // let mut walk_history=Vec::new();
    

    // let mut walk_history=Vec::new();
    let mut stk=Vec::new();
    let mut order=0;

    {
        let visiteds=HashSet::from([(root_record.path(),root_record.record_index())]);


        stk.extend(root_record.children().rev().map(|child|{
            Work { 
                record: child,
                depth:0,
                exit:false,
                exit_order:0,
                walk_parent:None,
                visiteds:visiteds.clone(),
                // include_origin:None,
                froms:Vec::new(),
            }
        }));
    }

    //
    while let Some(cur)=stk.pop() {
        //
        let mut new_froms=cur.froms.clone();
        new_froms.push(WalkFrom{ record: cur.record, custom: None });

        //walk ancestors
        if cur.depth>0 {
            walk_ancestors.truncate(cur.depth-1);
            walk_ancestors.push(cur.walk_parent.unwrap());
            // println!("== {:?}",cur.walk_parent.map(|x|x.value_str(0)));
        } else {
            // walk_ancestors.clear();
        }

        //handle circular check here?
        
        //
        let mut walk_skip_children=false;
        let mut walk_sibling_extends=Vec::new();
        let mut walk_child_extends=Vec::new();
        // let mut walk_skip_exit=false;
        let mut walk_have_exit=false;

        //
        callback(Walk { 
            record: cur.record, 
            depth: cur.depth, 
            exit: cur.exit, 
            order:cur.exit.then_some(cur.exit_order).unwrap_or(order),
            ancestors: walk_ancestors.as_ref(),
            skip_children:&mut walk_skip_children,
            sibling_extends:&mut walk_sibling_extends,
            child_extends:&mut walk_child_extends,
            // skip_exit:&mut walk_skip_exit,
            have_exit:&mut walk_have_exit,
            froms:&cur.froms,
        })
        // .or_else(|e //(e,loc)
        //     |Err(WalkError {
        //     // src:cur.record.src(),
        //     path:cur.record.path().map(|p|p.to_path_buf()),
        //     // loc: loc.unwrap_or(cur.record.start_loc()), 
        //     loc: cur.record.start_loc(), 
        //     error_type: WalkErrorType::Custom(e), 
        // }))
        ?;



        //
        for extend_from in walk_sibling_extends.into_iter().rev() {
            //
            let visited_key=(extend_from.record().path(),extend_from.record().record_index());
            
            if cur.visiteds.contains(&visited_key) {
                return Err(WalkError{
                    // src:cur.record.src(),
                    path:cur.record.path().map(|p|p.to_path_buf()),
                    loc:cur.record.start_loc(),
                    error_type:WalkErrorType::RecursiveInclude,
                });
            }

            //
            let mut visiteds=cur.visiteds.clone();
            visiteds.insert(visited_key);

            //
            let mut new_froms=new_froms.clone();
            new_froms.push(extend_from.clone());

            //
            stk.extend(extend_from.record().children().rev().map(|include_record|Work { 
                record: include_record,
                depth:cur.depth,
                exit:false,
                exit_order:0,
                walk_parent:cur.walk_parent,
                visiteds:visiteds.clone(),
                // include_origin:Some(cur.record),
                froms:new_froms.clone(),
            }));
        }

        //
        //allow inserting children on exit?
        //allow inserting on exit?
        
        //on enter add: includes, exit, insert children, children, 
        //on exit add: includes, insert children,

        //
        if !cur.exit { 
            //push exit
            if walk_have_exit // !walk_skip_exit 
            { //skip_exit obviously only works on enter
                stk.push(Work {
                    record: cur.record,
                    depth:cur.depth,
                    exit:true, 
                    exit_order:order,
                    walk_parent:cur.walk_parent, 
                    visiteds:cur.visiteds.clone(),
                    // include_origin:cur.include_origin,
                    froms:cur.froms.clone(),
                });
            }
        }

        //push inserted children, doesn't care about skip_children, and can be inserted on exit
        
        
        //
        for extend_from in walk_child_extends.into_iter().rev() {
            //
            let visited_key=(extend_from.record().path(),extend_from.record().record_index());
            
            if cur.visiteds.contains(&visited_key) {
                return Err(WalkError{
                    // src:cur.record.src(),
                    path:cur.record.path().map(|p|p.to_path_buf()),
                    loc:cur.record.start_loc(),
                    error_type:WalkErrorType::RecursiveInclude,
                });
            }

            //
            let mut visiteds=cur.visiteds.clone();
            visiteds.insert(visited_key);

            //
            let mut new_froms=new_froms.clone();
            new_froms.push(extend_from.clone());

            //
            stk.extend(extend_from.record().children().rev().map(|child|Work { 
                record: child,
                depth:cur.depth+1,
                exit:false,
                exit_order:0,
                walk_parent:Some(cur.record),
                visiteds:cur.visiteds.clone(),
                // include_origin:None,
                froms: new_froms.clone(),
            }));
        }

        //
        if !cur.exit { 
            //push children
            if !walk_skip_children { //only skips on enter, since not visiting children on exit

                let mut new_froms=cur.froms.clone();
                new_froms.push(WalkFrom { record: cur.record, custom: None });

                //
                stk.extend(cur.record.children().rev().map(|child|Work { 
                    record: child,
                    depth:cur.depth+1,
                    exit:false,
                    exit_order:0,
                    walk_parent:Some(cur.record),
                    visiteds:cur.visiteds.clone(),
                    // include_origin:None,
                    froms:new_froms.clone(),
                }));
            }

            //
            order+=1;
        }
    }

    //
    Ok(())
}