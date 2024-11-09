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




#[derive(Clone,Default)]
pub struct WalkAncestor<'a> {
    record:RecordContainer<'a>,
    note:Option<Rc<dyn Any>>,
    depth:usize,
    order:usize,
    breadth:usize,
}

impl<'a> WalkAncestor<'a> {
    pub fn record(&self) -> RecordContainer<'a> {
        self.record
    }
    pub fn get_note<T:Any+Clone>(&self) -> Option<T> {
        self.note.as_ref().and_then(|x|x.downcast_ref::<T>().map(|x|x.clone()))
    }
    pub fn depth(&self) -> usize {
        self.depth
    }
    pub fn order(&self) -> usize {
        self.order
    }
    pub fn breadth(&self) -> usize {
        self.breadth
    }
}

#[derive(Clone)]
pub struct WalkAncestorIter<'b,'a> {
    pub(super) ancestors : &'b Vec<WalkAncestor<'a>>,
    pub(super) start : usize,
    pub(super) end : usize,
}

impl<'b,'a> Iterator for WalkAncestorIter<'b,'a> {
    type Item = &'b WalkAncestor<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start>=self.end {
            None
        } else {
            let ind=self.ancestors.len()-self.start-1;
            self.start+=1;
            Some(self.ancestors.get(ind).unwrap())
        }
    }
}

impl<'b,'a> DoubleEndedIterator for WalkAncestorIter<'b,'a> {
    fn next_back(&mut self) -> Option<&'b WalkAncestor<'a>> {
        if self.start>=self.end {
            None
        } else {
            self.end-=1;
            let ind=self.ancestors.len()-self.end-1;
            Some(self.ancestors.get(ind).unwrap())
        }
    }
}

pub struct Walk<'b,'a> {
    record:RecordContainer<'a>,
    depth:usize,
    order:usize,
    breadth:usize,
    exit:bool,
    ancestors : &'b Vec<WalkAncestor<'a>>,
    skip_children : &'b mut bool,
    // sibling_inserts : &'b mut Vec<Bla<'a>>,
    // child_inserts : &'b mut Vec<Bla<'a>>,
    sibling_inserts : &'b mut Vec<(RecordContainer<'a>,Option<Rc<dyn Any>>)>,
    child_inserts : &'b mut Vec<(RecordContainer<'a>,Option<Rc<dyn Any>>)>,
    have_exit : &'b mut bool,
    cur_note : &'b mut Option<Rc<dyn Any>>,
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
    pub fn breadth(&self) -> usize {
        self.breadth
    }

    pub fn is_enter(&self) -> bool {
        !self.exit
    }

    pub fn is_exit(&self) -> bool {
        self.exit
    }

    pub fn ancestors_num(&self) -> usize {
        self.ancestors.len()
    }

    pub fn ancestor(&self,ind:usize) -> WalkAncestor<'a> {
        if self.ancestors.is_empty() || ind >=self.ancestors.len() {
            Default::default()
        } else {
            self.ancestors.get(self.ancestors.len()-ind-1).cloned().unwrap()
        }
    }
    pub fn get_ancestor(&self,ind:usize) -> Option<WalkAncestor<'a>> {
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
    
    pub fn parent(&self) -> WalkAncestor<'a> {
        self.ancestors.last().cloned().unwrap_or_default()
    }

    pub fn get_parent(&self) -> Option<WalkAncestor<'a>> {
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

    pub fn extend<I>(&mut self, records : I) 
    where
        I : IntoIterator<Item=RecordContainer<'a>>
    {
        self.sibling_inserts.extend(records.into_iter().map(|x|(x,None)));
    }

    pub fn extend_children<I>(&mut self, records : I) 
    where
        I : IntoIterator<Item=RecordContainer<'a>>
    {
        self.child_inserts.extend(records.into_iter().map(|x|(x,None)));
    }

    pub fn extend_note<I,T:Any>(&mut self, records : I, note:T) 
    where
        I : IntoIterator<Item=RecordContainer<'a>>
    {
        let note:Option<Rc<(dyn Any)>>=Some(Rc::new(note));
        self.sibling_inserts.extend(records.into_iter().map(|x|(x, note.clone())));
    }

    pub fn extend_children_note<I,T:Any>(&mut self, records : I, note:T) 
    where
        I : IntoIterator<Item=RecordContainer<'a>>
    {
        let note:Option<Rc<(dyn Any)>>=Some(Rc::new(note));
        self.child_inserts.extend(records.into_iter().map(|x|(x,note.clone())));
    }

    pub fn set_note<T:Any>(&mut self, note:T) {
        *self.cur_note=Some(Rc::new(note));
    }
    pub fn get_note<T:Any+Clone>(&self) -> Option<T> {
        self.cur_note.as_ref().and_then(|x|x.downcast_ref::<T>().map(|x|x.clone()))
    }
    pub fn have_exit(&mut self) {
        *self.have_exit=true;
    }
}

struct Work<'a> {
    record:RecordContainer<'a>,
    depth:usize,
    exit:bool,
    exit_order:usize,
    visiteds:HashSet<(Option<&'a Path>, usize)>,
    note : Option<Rc<dyn Any>>,
}

pub fn traverse<'a,E:Debug>(
    root_record : RecordContainer<'a>, 
    mut callback : impl for<'b> FnMut(Walk<'b,'a>) -> Result<(),WalkError<E>>,
) -> Result<(),WalkError<E>> {

    let mut walk_ancestors=Vec::new();
    let mut breadths=vec![0];
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
                visiteds:visiteds.clone(),
                note : None,
            }
        }));
    }

    //
    while let Some(cur_work)=stk.pop() {
        //walk ancestors
        //  the +1 makes it not remove cur node inserted during enter
        //  should note be kept from enter for the exit? 
        // walk_ancestors.truncate(cur_work.depth+cur_work.exit.then_some(1).unwrap_or(0));

        walk_ancestors.truncate(cur_work.depth);
        breadths.resize(cur_work.depth+1,0);
        
        if !cur_work.exit {
            *breadths.last_mut().unwrap()+=1;
        }

        let cur_breadth=breadths.last().cloned().unwrap()-1;

        //handle circular check here?
        
        //
        let mut walk_skip_children=false;
        let mut walk_sibling_inserts=Vec::new();
        let mut walk_child_inserts=Vec::new();
        let mut walk_have_exit=false;
        let mut walk_cur_note=cur_work.note.clone();

        //
        let cur_order=cur_work.exit.then_some(cur_work.exit_order).unwrap_or(order);

        //
        callback(Walk { 
            record: cur_work.record, 
            depth: cur_work.depth, 
            exit: cur_work.exit, 
            order:cur_order,
            breadth :cur_breadth,
            ancestors: walk_ancestors.as_ref(),
            skip_children:&mut walk_skip_children,            
            sibling_inserts:&mut walk_sibling_inserts,
            child_inserts:&mut walk_child_inserts,
            have_exit:&mut walk_have_exit,
            cur_note: &mut walk_cur_note,
        })?;

        //
        //note set on entry is lost on exit? yes
        // if !cur_work.exit {
        walk_ancestors.push(WalkAncestor { 
            record: cur_work.record, 
            note: walk_cur_note.clone(), 
            depth: cur_work.depth, 
            order:cur_order, 
            breadth:cur_breadth,
        });
        // }

        //
        for (include_record, include_note) in walk_sibling_inserts.into_iter().rev() {
            //
            let visited_key=(include_record.path(),include_record.record_index());
            
            if cur_work.visiteds.contains(&visited_key) {
                return Err(WalkError{
                    path:cur_work.record.path().map(|p|p.to_path_buf()),
                    loc:cur_work.record.start_loc(),
                    error_type:WalkErrorType::RecursiveInclude,
                });
            }

            //
            let mut visiteds=cur_work.visiteds.clone();
            visiteds.insert(visited_key);

            //
            stk.push(Work { 
                record: include_record,
                depth:cur_work.depth,
                exit:false,
                exit_order:0,
                visiteds:visiteds.clone(),
                note : include_note.clone(),
            });
        }

        //
        //allow inserting children on exit?
        //allow inserting on exit?
        
        //on enter add: includes, exit, insert children, children, 
        //on exit add: includes, insert children,

        //
        if !cur_work.exit { 
            //push exit
            if walk_have_exit // !walk_skip_exit 
            { //skip_exit obviously only works on enter
                stk.push(Work {
                    record: cur_work.record,
                    depth:cur_work.depth,
                    exit:true, 
                    exit_order:order,
                    visiteds:cur_work.visiteds.clone(),
                    note : walk_cur_note.clone(),
                });
            }
        }

        //
        for (child_record,child_note) in walk_child_inserts.into_iter().rev() {
            let visited_key=(child_record.path(),child_record.record_index());
            
            if cur_work.visiteds.contains(&visited_key) {
                return Err(WalkError{
                    path:cur_work.record.path().map(|p|p.to_path_buf()),
                    loc:cur_work.record.start_loc(),
                    error_type:WalkErrorType::RecursiveInclude,
                });
            }

            //
            let mut visiteds=cur_work.visiteds.clone();
            visiteds.insert(visited_key);

            //
            stk.push(Work { 
                record: child_record,
                depth:cur_work.depth+1,
                exit:false,
                exit_order:0,
                visiteds:cur_work.visiteds.clone(),
                note : child_note.clone(),
            });
        }

        //
        if !cur_work.exit { 
            //push children
            if !walk_skip_children { //only skips on enter, since not visiting children on exit
                stk.extend(cur_work.record.children().rev().map(|child|Work { 
                    record: child,
                    depth:cur_work.depth+1,
                    exit:false,
                    exit_order:0,
                    visiteds:cur_work.visiteds.clone(),
                    note : None,
                }));
            }

            //
            order+=1;
        }
    }

    //
    Ok(())
}