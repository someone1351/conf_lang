pub mod error;
use std::{collections::HashSet, fmt::Debug, path::Path};

use error::{WalkError,WalkErrorType};

use super::conf::container::record::RecordContainer;
use super::lexer::Loc;

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




pub struct Walk<'b,'a> {
    record:RecordContainer<'a>,
    depth:usize,
    order:usize,
    exit:bool,
    ancestors : &'b Vec<RecordContainer<'a>>,

    skip_children : &'b mut bool,
    traverse_inserts : &'b mut Vec<RecordContainer<'a>>,
    skip_exit : &'b mut bool,
}

impl<'b,'a> Walk<'b,'a> {
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
    pub fn ancestor(&self,ind:usize) -> RecordContainer<'a> {
        if self.ancestors.is_empty() {
            Default::default()
        } else {
            self.ancestors.get(self.ancestors.len()-ind-1).cloned().unwrap()
        }
    }
    pub fn parent(&self) -> RecordContainer<'a> {
        if self.ancestors.is_empty() {
            Default::default()
        } else {
            self.ancestors.last().cloned().unwrap()
        }
    }
    pub fn has_parent(&self) -> bool {
        !self.ancestors.is_empty()
    }

    pub fn skip_children(&mut self) {
        *self.skip_children=true;
    }

    pub fn insert(&mut self, record : RecordContainer<'a>) {
        self.traverse_inserts.push(record);
    }
    pub fn skip_exit(&mut self) {
        *self.skip_exit=true;
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
}

pub fn traverse<'a,E:Debug>(
    root_record : RecordContainer<'a>, 
    mut callback : impl for<'b> FnMut(Walk<'b,'a>) -> Result<(),(E,Option<Loc>)>) -> Result<(),WalkError<E>> {

    let mut walk_ancestors=Vec::new();
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
            }
        }));
    }

    //
    while let Some(cur)=stk.pop() {
        //walk ancestors
        if cur.depth>0 {
            walk_ancestors.truncate(cur.depth-1);
            walk_ancestors.push(cur.walk_parent.unwrap());
            // println!("== {:?}",cur.walk_parent.map(|x|x.value_str(0)));
        } else {
            walk_ancestors.clear();
        }
        
        //
        let mut walk_skip_children=false;
        let mut walk_traverse_inserts=Vec::new();
        let mut walk_skip_exit=false;

        //
        callback(Walk { 
            record: cur.record, 
            depth: cur.depth, 
            exit: cur.exit, 
            order:cur.exit.then_some(cur.exit_order).unwrap_or(order),
            ancestors: walk_ancestors.as_ref(),
            skip_children:&mut walk_skip_children,
            traverse_inserts:&mut walk_traverse_inserts,
            skip_exit:&mut walk_skip_exit,
        }).or_else(|(e,loc)|Err(WalkError {
            // src:cur.record.src(),
            path:cur.record.path().map(|p|p.to_path_buf()),
            loc: loc.unwrap_or(cur.record.start_loc()), 
            error_type: WalkErrorType::Custom(e), 
        }))?;

        //
        for include_record in walk_traverse_inserts.into_iter().rev() {
            let x=(include_record.path(),include_record.record_index());
            
            if cur.visiteds.contains(&x) {
                return Err(WalkError{
                    // src:cur.record.src(),
                    path:cur.record.path().map(|p|p.to_path_buf()),
                    loc:cur.record.start_loc(),
                    error_type:WalkErrorType::RecursiveInclude,
                });
            }

            let mut visiteds=cur.visiteds.clone();
            visiteds.insert(x);

            //push includes
            // stk.extend(include_record.children().rev().map(|child|Work { 
            //     record: child,
            //     depth:cur.depth,
            //     exit:false,
            //     exit_order:0,
            //     walk_parent:cur.walk_parent,
            //     visiteds:visiteds.clone(),
            //     // include_origin:Some(cur.record),
            // }));

            //
            stk.push(Work { 
                record: include_record,
                depth:cur.depth,
                exit:false,
                exit_order:0,
                walk_parent:cur.walk_parent,
                visiteds:visiteds.clone(),
                // include_origin:Some(cur.record),
            });
        }

        //
        if !cur.exit && !walk_skip_exit { //skip_exit obviously only works on enter
            //push exit
            stk.push(Work { 
                record: cur.record,
                depth:cur.depth,
                exit:true, 
                exit_order:order,
                walk_parent:cur.walk_parent, 
                visiteds:cur.visiteds.clone(),
                // include_origin:cur.include_origin,
            });

            //
            if !walk_skip_children { //only skips on enter, since not visiting children on exit
                //push children
                stk.extend(cur.record.children().rev().map(|child|Work { 
                    record: child,
                    depth:cur.depth+1,
                    exit:false,
                    exit_order:0,
                    walk_parent:Some(cur.record),
                    visiteds:cur.visiteds.clone(),
                    // include_origin:None,
                }));
            }

            //
            order+=1;
        }
    }

    //
    Ok(())
}