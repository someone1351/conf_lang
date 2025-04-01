//if .group() not used, don't have any stored in record.param_group()

//allow individual params to be marked optional, so if an optional param fails, can move onto the next one
//  but tagless nodes need to atleast have one value match? actually won't be a problem because a record has to have atleast one value, and if that value isn't matched to a tagless node's param, then it will fail


//allow parsed vals to be stolen from prev repeats param, also prev optionals
//  parsed can steal from prev str or parsed
//  str can only steal from prev str
//  only non optionals can steal?
//  if stealing from an optional, can steal everything
//  what if two repeats of same type, should the last one only be able to steal one? steal as many as possible?
//remove repeats_begin/end, replace with group_begin/end, which can be  marked optional and/or repeat

/*
    .param().optional()
    .param().repeat().optional()
    .group_begin("twos").param().param().group_end().repeat()
    .param().group("last")
    .param().param().group("last",2)

    .param().param().group(2,Some("last")).repeat()
    cannot group param that has been already made optional/repeat/grouped

    .group(Some("mygroup")).repeat().optional()
        .param()
        .param()
    .group_end()

    .group_begin(Some("mygroup"))
        .param()
        .param()
    .group_end().repeat().optional()

    .group(Some("mygroup")).repeat().optional()
        .param()
        .param()
    .groupless()
    .param()


    record.group("twos").value(0).unwrap()
    .tagless_nodes()
        .entry()
            .param_parse::<Binding>().repeat().optional().group("modifiers")
            .param_parse::<Binding>().group("primary")
            .param_parse::<f32>().optional().group("scale")

    for x in record.group("modifiers").value_parseds() {
    }
    let p=record.group("primary").value_parsed(0).unwrap();

*/

/*
TODO
* multi line comments
** start comment must have nothing or only whitespace in front of it
** end comment must have nothing or only whitespace after it
** #> comment <# or #! comment !#


*/

/*
* patterns in groups are needed for situations like:
(int int int int)+ (int int)+ => 1 2 3 4 5 6 7 8 => (1 2 3 4) ([4 5] [6 7])


*/

/*
* if in a repeating group, and it has optional params that aren't used, then end, stop more repeating
* if a param_group has optional params, then don't use with patterns? probably
*/

use std::any::{Any, TypeId};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::Path;

pub mod error;

use error::{ParseError, ParseErrorType};



use crate::def::node::GroupSimilar;

use super::conf::{self,Record,Value};
use super::def::container::branch::BranchContainer;
use super::def::container::node_children::NodeChildrenContainer;
// use super::def::container::param_group::ParamGroupContainer;
use super::lexer::{Lexer, Token};
use super::Conf;

pub fn parse_start<'a>(
    root_branch:BranchContainer,
    src : &'a str,
    keep_src:bool,
    path:Option<&'a Path>,
) -> Result<Conf,ParseError> {
    let mut lexer = Lexer::new(src);//cs.clone()
    // lexer.debug_print(true);
    let lexer=&mut lexer;
    lexer.debug_label_push("start");

    // start => (body | ending | ml_cmnt | cmnt | record)*

    // record => indent val (sep val)* (cmnt | ending)
    // body => {has_record_body}: (ending|indent not_eols ending)+
    // cmnt => spc? [#] not_eols ending
    // ml_cmnt => spc? "#!" (^"!#"|"\\!")* "!#" ending

    // val => q_val | s_val
    // s_val => ([\\]([\s\t\n\\]|[\r][\n]|quotes)|[^\s\t\n]|^([\r][\n]))+
    // q_val => quote_start ([\\]([\\]|quote)|not_quote_end)* quote_end

    // indent => [\s\t]*
    // spc => [\s\t]+
    // sep => ([\s\t]|[\\]([\r][\n]|[\n]))+
    // ending => spc? (eol|eof)
    // eol => [\n]|[\r][\n]
    // not_eols => ^eol*


    struct TempParamGroup {
        param_group_ind:usize,
        conf_param_group:conf::ParamGroup,
    }
    //
    // #[derive(Clone)]
    struct TempRecord {
        parent : Option<usize>,
        children_records : Vec<usize>,
        pub children_text:Option<Range<usize>>,

        pub param_groups : Range<usize>,
        pub values : Range<usize>,
        pub node_label : Option<usize>,
        pub branch_name : Option<usize>,
        tag:bool,
    }

    let mut temp_records = vec![TempRecord {
        parent: None,
        children_records: Vec::new(),
        children_text:None,
        param_groups:0..0,
        values:0..0,
        branch_name:None,
        node_label:None,
        tag:false,
    }];

    //
    let mut texts=Vec::<String>::new();
    let mut text_map=HashMap::<String,usize>::new();
    let mut all_param_groups = Vec::<conf::ParamGroup>::new();
    let mut all_values = Vec::<Value>::new();
    let mut all_parsed_values= Vec::new();
    let mut cur_parent = 0;
    let mut last_indent = 0;
    let mut tags_useds = vec![HashSet::<String>::new()]; //tags_useds[node_depth] = parent_tags_used
    let mut node_children_stk=Vec::<NodeChildrenContainer>::new();

    //
    while !lexer.is_end() {
        if !node_children_stk.is_empty() && node_children_stk.last().unwrap().is_body() {
            let tokens=parse_body(lexer,last_indent,path)?;

            if !tokens.is_empty() {
                let last_record_ind=temp_records.len()-1; //records len will always be > 1
                let val_start=all_values.len();

                all_values.extend(tokens.into_iter().map(|token|{
                    let text_ind=texts.len();
                    texts.push(token.extracted);

                    Value {
                        start_loc:token.start_loc,
                        end_loc:token.end_loc,
                        text_ind,
                        parsed_ind : None,
                    }
                }));

                let val_end=all_values.len();

                let parent_record=temp_records.get_mut(last_record_ind).unwrap();
                parent_record.children_text=Some(val_start..val_end)
                // parent_record.children_records.push(record_ind);
            }
        }

        if parse_ending(lexer)
            || parse_ml_cmnt(lexer,"#!","!#")
            || parse_ml_cmnt(lexer,"#>","<#")
            || parse_cmnt(lexer)
            {
            continue;
        }

        //
        if let Some((indent,tokens)) = parse_record(lexer,last_indent,path)? {
            let conf_val_start=all_values.len();

            all_values.extend(tokens.into_iter().map(|token|{
                let text_ind=if let Some(text_ind)=text_map.get(&token.extracted) {
                    *text_ind
                } else {
                    let text_ind=texts.len();
                    text_map.insert(token.extracted.clone(), text_ind);
                    texts.push(token.extracted);
                    text_ind
                };

                Value {
                    start_loc:token.start_loc,
                    end_loc:token.end_loc,
                    text_ind,
                    parsed_ind: None,
                }
            }));

            let all_val_end=all_values.len();
            let record_vals=&all_values[conf_val_start..all_val_end];

            if indent==last_indent+1 { //parent=cur_parent.child.last
                cur_parent=temp_records[cur_parent].children_records.last().cloned().unwrap();
            } else if indent < last_indent {
                for _ in indent .. last_indent { //parent=parent.parent
                    cur_parent=temp_records[cur_parent].parent.unwrap();
                }
            } else if indent == last_indent {

            }

            //
            tags_useds.resize(indent+1, HashSet::new());
            node_children_stk.truncate(indent);

            //
            //how to modify this so that NodeChildrenContainer::Branch can contain multiple branches to be used as children?
            let cur_branch=if indent==0 {
                root_branch
            } else {
                match node_children_stk.last().unwrap().clone() {
                    NodeChildrenContainer::BranchMissing(branch_name)=> {
                        //is this really needed? can quietly ignore missing children branches instead?
                        //  kinda needed on that there was children found, but the branch was missing that specified the children
                        return Err(ParseError{
                            path:path.map(|p|p.to_path_buf()),
                            loc:record_vals.first().unwrap().start_loc,
                            error_type:ParseErrorType::DefChildrenBranchNotFound(branch_name.to_string()),
                        });
                    }
                    NodeChildrenContainer::Branch(branch)=>{
                        branch.clone()
                    }
                    _ =>{
                        return Err(ParseError{
                            path:path.map(|p|p.to_path_buf()),
                            loc:record_vals.first().unwrap().start_loc,
                            error_type:ParseErrorType::DefNoChildren,
                        });
                    }
                }
            };

            //
            last_indent=indent;

            //
            let mut found_node=None;
            let mut record_attempted_parse_vals:HashMap<usize,HashMap<TypeId,Option<(&'static str,Box<dyn Any+Send+Sync>)>>>=HashMap::new(); //[record_val_ind][type_id]=val
            let first_val_text=texts.get(record_vals.first().unwrap().text_ind).unwrap().clone();

            let mut cur_param_groups=Vec::<TempParamGroup>::new();

            //look at nodes
            for node in cur_branch.get_tag_nodes(first_val_text.as_str()).chain(cur_branch.get_tagless_nodes()) {

                cur_param_groups.clear();

                let record_val_start=node.has_tag().then_some(1).unwrap_or(0);
                let record_vals_num=record_vals.len()-record_val_start;

                //
                let mut ok=true;

                if node.param_groups_num()==1 { //handle single group, more efficient then using code below in else, could disable this and only use code in else
                    //
                    let param_group=node.param_group(0).unwrap();
                    let params_num=param_group.params_num();
                    let param_optional=param_group.param_optional().unwrap_or(params_num);

                    //skip on invalid params num
                    if record_vals_num==params_num
                        || (param_group.optional() && record_vals_num==0) // && group.params_num()!=0
                        || (param_group.repeat() && params_num!=0 && record_vals_num!=0 && record_vals_num%params_num==0)

                        || ((param_group.repeat() || record_vals_num<params_num) && record_vals_num%params_num >= param_optional)
                    {} else { continue; }

                    //
                    for record_val_ind in record_val_start .. record_vals.len() {
                        let record_val = record_vals.get(record_val_ind).unwrap();
                        let param_ind= (record_val_ind-record_val_start)%params_num;

                        if let Some(val_type_id) = param_group.param_type_id(param_ind) { //check func, anys not checked
                            let parsed_vals=record_attempted_parse_vals.entry(record_val_ind).or_default(); //-record_val_start

                            let parsed_val=parsed_vals.entry(val_type_id).or_insert_with(||{
                                let text=texts.get(record_val.text_ind).unwrap();

                                //run param func
                                let param_result=param_group.param_parse(param_ind, text.as_str());

                                //
                                param_result.map(|param_result|{
                                    (param_group.param_type_name(param_ind).unwrap(),param_result)
                                })
                            });

                            if parsed_val.is_none() {
                                ok=false;
                                break;
                            }
                        }
                    }

                    //
                    let group_name_text_ind= param_group.name().map(|group_name|{
                        if let Some(text_ind)=text_map.get(group_name) {
                            *text_ind
                        } else {
                            let text_ind=texts.len();
                            text_map.insert(group_name.to_string(), text_ind);
                            texts.push(group_name.to_string());
                            text_ind
                        }
                    });

                    cur_param_groups.push(TempParamGroup {
                        param_group_ind:0,
                        conf_param_group:conf::ParamGroup {
                            conf_values: conf_val_start+record_val_start..all_val_end,
                            name: group_name_text_ind,
                            params_num,
                            optional:param_group.optional(),
                            repeat:param_group.repeat(),
                        }
                    });
                } else
                {
                    //
                    let mut record_val_ind=record_val_start;
                    let mut param_group_ind=0;

                    while record_val_ind<record_vals.len() {
                        //println!("==while g={param_group_ind}: v={record_val_ind}");
                        let param_group=node.param_group(param_group_ind).unwrap();

                        //some might not have any params and are invalid
                        //  should ignore (skip over) if no params?
                        if param_group.params_num()==0 {
                            ok=false; //not really necessary? why not?
                            break;
                        }

                        //handle adjacent param groups that share a pattern
                        //  do not use patterns when using param_optional
                        // println!("gs {:?}",param_group.similar());
                        if param_group.similar()!=GroupSimilar::None && param_group.param_optional().is_none() && (param_group.optional() || param_group.repeat()) {
                            let mut has_repeat=param_group.repeat();
                            let mut param_group_ind2=param_group_ind+1;

                            // let mut total_tuple_count=param_group.params_patterns_num();
                            let mut optional_tuple_count=if param_group.optional(){param_group.params_patterns_num()}else{0};
                            let mut needed_tuple_count=if param_group.optional(){0}else{param_group.params_patterns_num()};

                            //count num of param tuples matching the pattern in param groups
                            while param_group_ind2<node.param_groups_num() {
                                let param_group2=node.param_group(param_group_ind2).unwrap();

                                //do not use patterns when using param_optional
                                if param_group.similar()!=param_group2.similar() || param_group2.param_optional().is_some() {
                                    break;
                                }

                                //check the params groups have same pattern
                                if param_group.params_pattern_len()==param_group2.params_pattern_len() {
                                    let mut i=0;

                                    while i<param_group.params_pattern_len()
                                        && param_group.param_type_id(i)==param_group2.param_type_id(i)
                                    {
                                        i+=1;
                                    }

                                    if i!=param_group.params_pattern_len() { //stop if param groups patterns not the same
                                        break;
                                    }
                                }

                                //
                                if !param_group2.optional() {
                                    needed_tuple_count+=param_group2.params_patterns_num();
                                } else {
                                    optional_tuple_count+=param_group2.params_patterns_num();
                                }

                                // println!("------total_tuple_count={total_tuple_count}");
                                param_group_ind2+=1;
                                has_repeat|=param_group2.repeat();
                            }

                            //
                            let total_tuple_count=optional_tuple_count+needed_tuple_count; //non repeating
                            let adj_param_groups_num=param_group_ind2-param_group_ind;

                            //needed_tuple_count total_tuple_count
                            //println!("\tadj_param_groups_num={adj_param_groups_num}");

                            //on more than one adj param group with same pattern
                            if adj_param_groups_num>1 {
                                let mut record_val_ind2=record_val_ind;

                                //get all record values matching pattern
                                loop {

                                    //can exit early if no repeats and vals count eq to needed tuples count * pattern_len
                                    if !has_repeat && (record_val_ind2-record_val_ind)==total_tuple_count*param_group.params_pattern_len() {
                                        break;
                                    }

                                    //
                                    let mut param_found=false;

                                    //
                                    for pattern_ind in 0..param_group.params_pattern_len() {
                                        //
                                        param_found=param_group.param_type_id(pattern_ind).map(|param_type_id|{
                                            record_attempted_parse_vals
                                                .entry(record_val_ind2).or_default() //-record_val_start
                                                .entry(param_type_id).or_insert_with(||
                                            {
                                                let val = record_vals.get(record_val_ind2).unwrap();
                                                let text=texts.get(val.text_ind).unwrap();
                                                let v=param_group.param_parse(pattern_ind, text.as_str());

                                                v.map(|v|(param_group.param_type_name(pattern_ind).unwrap(),v))
                                            }).is_some()
                                        }).unwrap_or(true);

                                        //
                                        if param_found { //param matches value
                                            record_val_ind2+=1;
                                        } else {
                                            break;
                                        }
                                    }

                                    //
                                    if !param_found || record_val_ind2==record_vals.len() {
                                        break;
                                    }
                                }

                                //
                                let found_vals_num=record_val_ind2-record_val_ind;

                                //
                                // println!("thevals {:?}", (record_val_ind..record_val_ind2).map(|i|{
                                //     let text_ind=record_vals.get(i).unwrap().text_ind;
                                //     texts.get(text_ind).unwrap()
                                // }).collect::<Vec<_>>());

                                //
                                if needed_tuple_count*param_group.params_pattern_len() <= found_vals_num && (
                                    has_repeat
                                    || needed_tuple_count*param_group.params_pattern_len() == found_vals_num
                                    || total_tuple_count*param_group.params_pattern_len() == found_vals_num
                                ) {
                                    //println!("-------tryyyy");

                                    // let from_right=node.rsimilar();
                                    let from_right = param_group.similar()==GroupSimilar::Right;

                                    //int? int? int int*
                                    //int* int? int? int
                                    //int? int* int
                                    //int int* int?

                                    let mut tuple_assigns= Vec::new(); //(0..(param_group_ind2-param_group_ind)).
                                    tuple_assigns.resize(adj_param_groups_num, 0);

                                    let mut remaining_tuple_count= (found_vals_num/param_group.params_pattern_len())-needed_tuple_count;
                                    //println!("remaining_tuple_count={remaining_tuple_count}");
                                    //println!("--hm1 {}-{needed_tuple_count} ={remaining_tuple_count} ",found_vals_num/param_group.params_pattern_len());

                                    //assign tuples to param groups
                                    for i in 0 .. adj_param_groups_num {
                                        let i=if from_right{adj_param_groups_num-i-1}else{i};
                                        let param_group_ind3=param_group_ind+i;
                                        let param_group3=node.param_group(param_group_ind3).unwrap();
                                        let params_patterns_num=param_group3.params_patterns_num();

                                        if !param_group3.optional() {
                                            tuple_assigns[i]=params_patterns_num;
                                            //println!("ggg n {:?} = {}",param_group3.name(),tuple_assigns[i]);
                                        } else if remaining_tuple_count>=params_patterns_num {
                                            tuple_assigns[i]=params_patterns_num;
                                            remaining_tuple_count-=params_patterns_num;
                                            //println!("ggg o {:?} = {}",param_group3.name(),tuple_assigns[i]);
                                        }

                                        if param_group3.repeat() {
                                            while remaining_tuple_count>=params_patterns_num {
                                                tuple_assigns[i]+=params_patterns_num;
                                                //println!("~~~ {remaining_tuple_count}-={params_patterns_num}");
                                                remaining_tuple_count-=params_patterns_num;
                                            }
                                        }

                                    }

                                    //
                                    if remaining_tuple_count==0 {
                                        //println!("--ok {remaining_tuple_count} {tuple_assigns:?}");

                                        for (i,&tuple_assign) in tuple_assigns.iter().enumerate() {
                                            let param_group_ind2=param_group_ind+i;
                                            let param_group2=node.param_group(param_group_ind2).unwrap();
                                            // println
                                            //
                                            let group_name_text_ind= param_group2.name().map(|group_name|{
                                                if let Some(text_ind)=text_map.get(group_name) {
                                                    *text_ind
                                                } else {
                                                    let text_ind=texts.len();
                                                    text_map.insert(group_name.to_string(), text_ind);
                                                    texts.push(group_name.to_string());
                                                    text_ind
                                                }
                                            });

                                            //
                                            //println!("record_val_ind {record_val_ind} + {tuple_assign}*{}",param_group2.params_num());
                                            let record_val_num =tuple_assign*param_group2.params_pattern_len();

                                            //println!("!!! record_val_ind {record_val_ind}, record_val_num {record_val_num}");
                                            //
                                            cur_param_groups.push(TempParamGroup {
                                                param_group_ind:param_group_ind2,
                                                conf_param_group:conf::ParamGroup {
                                                    conf_values: conf_val_start+record_val_ind..conf_val_start+record_val_ind+record_val_num,
                                                    name: group_name_text_ind,
                                                    params_num: param_group2.params_num(),
                                                    optional:param_group2.optional(),
                                                    repeat:param_group2.repeat(),
                                                }
                                            });

                                            record_val_ind+=record_val_num;

                                        }

                                        param_group_ind=param_group_ind2;
                                        continue;
                                    } else {
                                        //println!("---no {remaining_tuple_count} {tuple_assigns:?}");
                                    }
                                }

                                //
                            }
                        }

                        //(else?) handle param group
                        {
                            let mut many_count=0;
                            let mut param_found=false;
                            let mut param_optional_used=false;
                            let mut record_val_ind2=record_val_ind;

                            // let mut ok2=true;

                            loop { //for repeats
                                //println!("==loop repeat {record_val_ind}, {record_val_ind2}");
                                let mut record_val_ind3=record_val_ind2;

                                //get vals for params
                                for param_ind in 0..param_group.params_num() {
                                    // println!("=== p={param_ind}, r={record_val_ind3} rl={}",record_vals.len());

                                    //only used
                                    if record_val_ind3 == record_vals.len() {
                                        // println!("hmm2 p={param_ind} r={record_val_ind3} {:?}",record_vals.get(record_val_ind3).and_then(|val|texts.get(val.text_ind)));
                                        if let Some(param_optional)=param_group.param_optional() {
                                            if param_ind >=param_optional {
                                                param_found=true;
                                                param_optional_used=true;
                                            }
                                        }
                                        break;
                                    }
                                    //
                                    let param_type_id=param_group.param_type_id(param_ind);

                                    if let Some(param_type_id)=param_type_id{
                                        let param_type_name=param_group.param_type_name(param_ind).unwrap();

                                        let param_parsed=record_attempted_parse_vals
                                            .entry(record_val_ind3).or_default() //-record_val_start
                                            .entry(param_type_id).or_insert_with(||
                                        {
                                            let val = record_vals.get(record_val_ind3).unwrap();
                                            let text=texts.get(val.text_ind).unwrap();

                                            //does none repesent param_any? don't think so, just the parse failed
                                            //  storing none just tells it that the parse has already been attempted and failed
                                            let v=param_group.param_parse(param_ind, text.as_str());

                                            v.map(|v|(param_type_name,v))
                                        });
                                        param_found=param_parsed.is_some();
                                    } else { //does it mean any? yes..
                                        param_found=true;
                                    }


                                    //
                                    if param_found { //param matches value
                                        // println!("hmm1 p={param_ind} r={record_val_ind3} {:?}, rlen={}",
                                        //     record_vals.get(record_val_ind3).and_then(|val|texts.get(val.text_ind)),
                                        //     record_vals.len(),
                                        // );
                                        record_val_ind3+=1;
                                        //println!("====x {record_val_ind3}");
                                    // } else if let Some(param_optional)=param_group.param_optional() {
                                    //     if param_ind >=param_optional {
                                    //         param_found=true;
                                    //     } else {
                                    //         break;
                                    //     }
                                    } else {
                                        // println!("hmm0 p={param_ind} r={record_val_ind3} {:?}",record_vals.get(record_val_ind3).and_then(|val|texts.get(val.text_ind)));

                                        //only handles incorrect param type, not too little record vals
                                        if let Some(param_optional)=param_group.param_optional() {
                                            if param_ind >=param_optional {
                                                param_found=true;
                                                param_optional_used=true;
                                            }
                                        }

                                        break;
                                    }
                                } //end for (param_ind)

                                //after looping through params
                                if param_found { //all success
                                    // println!("==a");
                                    record_val_ind2=record_val_ind3;
                                    many_count+=1;

                                    if !param_optional_used && param_group.repeat() && record_val_ind3<record_vals.len() {
                                        continue;
                                    }
                                } else if !param_group.optional() && (!param_group.repeat() || many_count==0) {
                                    // println!("==b");
                                    // ok2=false; //node fail
                                    ok=false; //node fail
                                }

                                //
                                break;
                            } //end loop (repeats)

                            //
                            // println!("group1 is {:?}, many_count={many_count}, ok2={ok}",param_group.name());

                            //
                            // if !ok2 {
                            //     break;
                            // }
                            if !ok {
                                break;
                            }

                            //
                            //after completing a param group, store it
                            // if last_param_group_ind!=param_group_ind
                            {
                                //
                                let group_name_text_ind= param_group.name().map(|group_name|{
                                    if let Some(text_ind)=text_map.get(group_name) {
                                        *text_ind
                                    } else {
                                        let text_ind=texts.len();
                                        text_map.insert(group_name.to_string(), text_ind);
                                        texts.push(group_name.to_string());
                                        text_ind
                                    }
                                });

                                //
                                let conf_values=if many_count!=0 {conf_val_start+record_val_ind..conf_val_start+record_val_ind2} else {0..0};

                                //
                                cur_param_groups.push(TempParamGroup {
                                    param_group_ind,
                                    conf_param_group:conf::ParamGroup {
                                        conf_values,
                                        name: group_name_text_ind,
                                        params_num: param_group.params_num(),
                                        optional:param_group.optional(),
                                        repeat:param_group.repeat(),
                                    }
                                });
                            }

                            if many_count!=0 {
                                record_val_ind=record_val_ind2;
                                //println!("====5 {record_val_ind}={record_val_ind2}");
                            }

                            //
                            param_group_ind+=1;

                            //
                            if param_group_ind==node.param_groups_num() { //is last param group
                                // record_val_ind=record_last_val_ind; //restore record_val_ind to last successful param_group match //why?
                                //println!("====6");
                                break;
                            }
                        } //
                    } //end while (record_val_ind)

                    //
                    if ok {
                        //skip (remaining?) optional param groups
                        while param_group_ind<node.param_groups_num() &&
                            (node.param_group(param_group_ind).unwrap().optional() || node.param_group(param_group_ind).unwrap().param_optional()==Some(0))
                        {
                            let param_group=node.param_group(param_group_ind).unwrap();

                            // println!("group2 is {:?}, ",param_group.name());

                            //
                            let group_name_text_ind= param_group.name().map(|group_name|{
                                if let Some(text_ind)=text_map.get(group_name) {
                                    *text_ind
                                } else {
                                    let text_ind=texts.len();
                                    text_map.insert(group_name.to_string(), text_ind);
                                    texts.push(group_name.to_string());
                                    text_ind
                                }
                            });

                            //
                            cur_param_groups.push(TempParamGroup {
                                param_group_ind,
                                conf_param_group:conf::ParamGroup {
                                    conf_values:0..0,
                                    name: group_name_text_ind,
                                    params_num: param_group.params_num(),
                                    optional:param_group.optional(),
                                    repeat:param_group.repeat(),
                                }
                            });

                            //
                            param_group_ind+=1;
                        }

                        //failed to go through all record vals or failed to go through all param groups
                        if record_val_ind!=record_vals.len() || param_group_ind!=node.param_groups_num() {
                            ok=false;
                            // println!("a3 {} {}, {} {}",
                            //     record_val_ind,record_vals.len(),
                            //     param_group_ind,node.param_groups_num(),
                            // );
                        }
                    }
                } //

                //enforce tag once (only one of the tag allowed)
                if ok && node.has_tag() && node.tag_once() && tags_useds.last().unwrap().contains(&first_val_text) {
                    return Err(ParseError {
                        path:path.map(|p|p.to_path_buf()),
                        loc: record_vals.first().unwrap().start_loc,
                        error_type: ParseErrorType::TagOnce(first_val_text.clone()),
                    });
                }

                //
                if ok {
                    //store used tags
                    if node.has_tag() {
                        tags_useds.last_mut().unwrap().insert(first_val_text.clone());
                    }

                    //store found node
                    found_node=Some(node);
                    break;
                }
            }

            //err if no found_node
            let Some(node)=found_node else {
                return Err(ParseError{
                    path:path.map(|p|p.to_path_buf()),
                    loc:record_vals.first().unwrap().start_loc,
                    error_type:ParseErrorType::NoDefForRecord,
                });
            };

            //store parsed values for record
            {
                let mut record_val_ind=node.has_tag().then_some(1).unwrap_or(0);

                for temp_param_group in cur_param_groups.iter() {
                    let conf_values=temp_param_group.conf_param_group.conf_values.clone();
                    let param_group=node.param_group(temp_param_group.param_group_ind).unwrap();

                    for i in 0..conf_values.len() {
                        // let conf_value_ind=conf_values.start+i;
                        let param_ind=i%param_group.params_num();

                        if let Some(type_id)=param_group.param_type_id(param_ind)  {
                            // println!("hmm {:?}, {:?}",param_group.param_type_name(param_ind), texts.get(all_values.get(conf_values.start+i).unwrap().text_ind).unwrap());

                            // let record_val_start=node.has_tag().then_some(1).unwrap_or(0);
                            let parsed_vals=record_attempted_parse_vals.get_mut(&(record_val_ind)).unwrap(); //-record_val_start
                            // println!("\t\t:{:?}",parsed_vals.iter().map(|x|x.1.is_some()).collect::<Vec<_>>());
                            let parsed_val=parsed_vals.remove(&type_id).unwrap();
                            let value=all_values.get_mut(conf_val_start+record_val_ind).unwrap();

                            value.parsed_ind=Some(all_parsed_values.len());
                            // println!(" parsedind={:?}",value.parsed_ind);
                            all_parsed_values.push(parsed_val.unwrap());
                        }

                        record_val_ind+=1;
                    }
                    // record_val_ind+=conf_values.len();
                }
            }

            //
            let param_groups_start=all_param_groups.len();
            all_param_groups.extend(cur_param_groups.drain(0..).map(|x|x.conf_param_group));
            let param_groups_end=all_param_groups.len();

            //
            node_children_stk.push(node.children());

            //
            let branch_name=cur_branch.name().unwrap_or_default().to_string();
            let node_label=node.label().map(|x|x.to_string());

            let branch_name_text_ind= if let Some(text_ind)=text_map.get(&branch_name) {
                *text_ind
            } else {
                let text_ind=texts.len();
                text_map.insert(branch_name.clone(), text_ind);
                texts.push(branch_name);
                text_ind
            };

            let node_label_text_ind= node_label.map(|node_label|{
                if let Some(text_ind)=text_map.get(&node_label) {
                    *text_ind
                } else {
                    let text_ind=texts.len();
                    text_map.insert(node_label.clone(), text_ind);
                    texts.push(node_label);
                    text_ind
                }
            });

            //
            let record_ind=temp_records.len();

            temp_records.push(TempRecord {
                branch_name:Some(branch_name_text_ind),
                node_label:node_label_text_ind,
                param_groups:param_groups_start..param_groups_end,
                values : conf_val_start..all_val_end,
                parent:Some(cur_parent),
                children_records:Vec::new(),
                children_text:None,
                tag:node.has_tag(),
            });

            temp_records.get_mut(cur_parent).unwrap().children_records.push(record_ind);

            //
            continue;
        }

        //
        if !lexer.is_end() {
            lexer.debug_label_pop();

            return Err(ParseError{
                path:path.map(|p|p.to_path_buf()),
                loc:lexer.loc(),
                error_type:ParseErrorType::Unknown,
            });
        }
    }

    //
    lexer.debug_label_pop();

    //
    let mut records = vec![Record {
        param_groups:0..0,
        conf_values: 0..0,
        parent: None,
        children: 0..0,
        children_text:false,
        node_label: None,
        branch_name: None,
        tag:false,
    }];

    let mut stk=vec![(0,0)]; //[]=(temp_record_ind, record_ind)

    while let Some((temp_record_ind,record_ind))= stk.pop() {
        let temp_record=temp_records.get(temp_record_ind).unwrap();
        let record_children_start = records.len();

        for &child_temp_record_ind in temp_record.children_records.iter() {
            let child_temp_record=temp_records.get(child_temp_record_ind).unwrap();

            records.push(Record {
                parent: Some(record_ind),
                children: child_temp_record.children_text.clone().unwrap_or(0..0),
                children_text:child_temp_record.children_text.is_some(),
                param_groups:child_temp_record.param_groups.clone(),
                conf_values: child_temp_record.values.clone(),
                node_label: child_temp_record.node_label,
                branch_name: child_temp_record.branch_name,
                tag:child_temp_record.tag,
            });
        }

        if !temp_record.children_records.is_empty() {
            let record_children_end = records.len();
            let record=records.get_mut(record_ind).unwrap();
            record.children=record_children_start .. record_children_end;
        }

        stk.extend(temp_record.children_records.iter().enumerate()
            .map(|(i,&child_temp_record_ind)|(child_temp_record_ind,record_children_start+i)));
    }

    //

    let mut param_group_name_map = HashMap::new();

    for param_group in all_param_groups.iter() {
        if let Some(text_ind)=param_group.name {
            let text=texts.get(text_ind).unwrap().clone();
            param_group_name_map.insert(text, text_ind);
        }
    }


    //
    let mut param_group_map = HashMap::new();

    for (record_ind,record) in records.iter().enumerate() {
        for param_group_ind in record.param_groups.clone() {
            let param_group=all_param_groups.get(param_group_ind).unwrap();

            if let Some(text_ind)=param_group.name {
                param_group_map.insert((text_ind,record_ind), param_group_ind);
            }
        }
    }

    //
    Ok(Conf {
        records,
        texts,
        path: path.and_then(|x|Some(x.to_path_buf())),
        // src : src.and_then(|x|Some(x.to_string())),
        src:keep_src.then(||src.to_string()),
        values: all_values,
        param_groups:all_param_groups,
        param_group_map,
        param_group_name_map,
        parsed_values: all_parsed_values,
    })
}


fn parse_record<'a>(lexer : &mut Lexer,last_indent:usize,
    // src:&'a str,
    path:Option<&'a Path>,) -> Result<Option<(usize,Vec<Token>)>,ParseError> {
    // record => indent val (sep val)* (cmnt | ending)

    lexer.debug_label_push("record");

    //
    let mut tokens=Vec::new();

    //
    lexer.push();

    //
    let indent=parse_indent(lexer,last_indent,false,path)?;
    // println!("indent={indent}");

    //
    if !parse_val(lexer,path)? {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return Ok(None);
    }

    //
    tokens.push(lexer.token().unwrap());
    // println!("\n= {} {:?}\n",tokens.len()-1,tokens.last().unwrap());
    lexer.pop_keep();

    //
    loop {
        lexer.push();

        //
        if !parse_sep(lexer) {
            break;
        }

        //
        if parse_cmnt(lexer) {
            lexer.pop_keep();
            lexer.debug_label_pop();
            return Ok(Some((indent,tokens)));
        }

        //
        if !parse_val(lexer,path)? {
            lexer.pop_discard();
            break;
        } else {
            tokens.push(lexer.token().unwrap());
            // println!("\n= {} {:?}\n",tokens.len()-1,tokens.last().unwrap());
            lexer.pop_keep();
        }
    }

    //
    if !parse_ending(lexer) {
        lexer.debug_label_pop();
        // return Err((lexer.loc(),"expected EOL/EOF"));
        return Err(ParseError{
            path:path.map(|p|p.to_path_buf()),
            loc:lexer.loc(),
            error_type:ParseErrorType::ExpectedEOL,
        });
    }

    //
    lexer.debug_label_pop();

    Ok(Some((indent,tokens)))
}

fn parse_body<'a>(lexer : &mut Lexer,last_indent:usize,
    // src:&'a str,
    path:Option<&'a Path>) -> Result<Vec<Token>,ParseError> {
    // body => {has_record_body}: (ending|indent not_eols ending)+

    let mut tokens=Vec::new();

    //
    lexer.debug_label_push("body");

    while !lexer.is_end() {
        //any spcs + eol/eof are ignored
        if parse_ending(lexer) {
            continue;
        }

        //anything after last_ident+1 are stored (including spcs)
        //eols aren't stored from any of these lines

        //
        lexer.push();
        let indent=parse_indent(lexer,last_indent,true,path)?;

        // if indent==0 {
        //     lexer.pop_discard();
        //     break;
        // }

        if indent<=last_indent {
            lexer.pop_discard();
            break;
        }

        lexer.pop_keep();

        //
        if parse_not_eols(lexer,false) {
            tokens.push(lexer.token().unwrap());
        }

        if !parse_ending(lexer) { //should always succeed
            panic!("");
        }
    }

    //
    lexer.debug_label_pop();
    Ok(tokens)
}

fn parse_val<'a>(lexer : &mut Lexer,
    // src:&'a str,
    path:Option<&'a Path>) -> Result<bool,ParseError> {
    // val => q_val | s_val

    // let quotes=['\'','"','`'];
    let quotes=["'","\"","`"];

    lexer.debug_label_push("val");

    //
    for tripple in [true,false] {
        for &quote in quotes.iter() {
            if parse_qval(lexer, quote,tripple,path)? {
                lexer.debug_label_pop();
                return Ok(true);
            }
        }
    }

    //
    let res=parse_sval(lexer,quotes);


    lexer.debug_label_pop();
    return Ok(res);



}


fn parse_sval<const Q:usize>(lexer : &mut Lexer,quotes: [&str;Q]) -> bool {
    // s_val => ([\\]([\s\t\n\\]|[\r][\n]|quotes)|[^\s\t\n]|^([\r][\n]))+

    lexer.debug_label_push("s_val");
    let mut found=false;


    //
    loop {
        //escape
        if lexer.has(0, ["\\"]).is_some() {
            if let Some(x)=lexer.has(1, [" ","\t", "\\"]) {


                // lexer.consume(1+x.len(), Some(x)).unwrap();
                //don't replace \\ with \, only spaces, so user can decide what to do with the other escapes eg \n \a \0 \\ etc
                lexer.consume(1+x.len(), (x!="\\").then_some(x)).unwrap();
                found=true;
                continue;
            } else if let Some(x)=lexer.has(1, ["\n","\r\n"]) {
                lexer.skip(1+x.len()).unwrap();
                found=true;
                continue;
            } else if let Some(x)=lexer.has(1, quotes) {
                lexer.consume(1+x.len(), Some(x)).unwrap();
                found=true;
                continue;
            }
        }

        //char
        if lexer.has(0, [" ","\t","\n","\r\n"]).is_none() && lexer.consume(1, None).is_some() {
            found=true;
            continue;
        }

        //
        break;
    }

    //
    lexer.debug_label_pop();
    found
}

fn parse_qval<'a>(lexer : &mut Lexer,quote:&str,tripple:bool,
    // src:&'a str,
    path:Option<&'a Path>
) -> Result<bool,ParseError> {
    //don't handle quotes differently eg single quote doesn't convert escapes like \n to newline,
    //  instead let the user decide how to handle it themselves
    //  should do so in def?

    // q_val => quote_start ([\\]([\\]|quote)|not_quote_end)* quote_end

    lexer.debug_label_push("q_val");

    //
    // let quote="\"";

    let quote2=if tripple{quote.repeat(3)}else{quote.to_string()};
    let quote2=quote2.as_str();

    //quote start
    let Some(x)=lexer.has(0, [quote2]) else {
        lexer.debug_label_pop();
        return Ok(false);
    };

    lexer.skip(x.len());

    //chars
    // while parse_escaped(lexer) || (lexer.has(0, [q]).is_none() && lexer.consume(1, None).is_some())
    // {
    // }

    //
    loop {
        //escape
        if lexer.has(0, ["\\"]).is_some() {
            if let Some(x)=lexer.has(1, ["\\",quote]) {
                lexer.consume(1+x.len(), (x==quote).then_some(x)).unwrap();
                // lexer.consume(1+x.len(), Some(x)).unwrap();
                continue;
            }
        }

        //char
        if lexer.has(0, [quote]).is_none() && lexer.consume(1, None).is_some() {
            continue;
        }

        //
        break;
    }

    //quote end
    let Some(x)=lexer.has(0, [quote2]) else {
        lexer.debug_label_pop();
        return Err(ParseError{
            // src,
            path:path.map(|p|p.to_path_buf()),
            loc:lexer.loc(),
            error_type:ParseErrorType::NoClosingQuote(quote2.to_string()),
        });

        // return Err((lexer.loc(),"expected closing quote"));
    };

    lexer.skip(x.len());

    //
    lexer.debug_label_pop();
    Ok(true)
}



fn parse_indent<'a>(lexer : &mut Lexer,last_indent:usize, in_body:bool,
    // src:&'a str,
    path:Option<&'a Path>) -> Result<usize,ParseError> {
    // indent => [\s\t]*

    //
    lexer.debug_label_push("indent");

    let mut spcs = 0;

    let mut i=0;

    //
    while let Some(x)=lexer.has(i, [" ","\t"]) {
        if x=="\t" {
            spcs+=4;
        } else {
            spcs+=1;
        }

        // lexer.skip(x.len());
        i+=1;

        //problem is what if 2spcs + tab + 2spcs
        //but body's parent is idented 1(eg 4 spcs or 1tab)
        //it should
        //maybe an indent is only valid for a body if up to last_indent+1 is made of up tabs and spcs%4=0

        if in_body && spcs>=last_indent*4+4 { //indent>=last_indent
            break;
        }
    }

    //
    let indent=spcs/4;


    if in_body && indent <=last_indent {
        //
        lexer.debug_label_pop();

        //
        Ok(0)
    } else {
        lexer.skip(i);

        //
        lexer.debug_label_pop();

        //
        if spcs%4 !=0 {
            Err(ParseError{
                path:path.map(|p|p.to_path_buf()),
                loc:lexer.loc(),
                error_type:ParseErrorType::InvalidIndentIncrement,
            })
            // Err((lexer.loc(),"indents must be increments of 4"))
        } else if indent>last_indent+1 {
            Err(ParseError{
                path:path.map(|p|p.to_path_buf()),
                loc:lexer.loc(),
                error_type:ParseErrorType::InvalidIndent,
            })
            // Err((lexer.loc(),"invalid indent"))
        } else {
            Ok(indent)
        }
    }
}

fn parse_eol(lexer : &mut Lexer,) -> bool {
    // eol => [\n]|[\r][\n]

    //
    lexer.debug_label_push("eol");

    //
    if let Some(x)=lexer.has(0, ["\n","\r\n"]) {
        lexer.skip(x.len());
        lexer.debug_label_pop();
        return true;
    }

    //
    lexer.debug_label_pop();
    false
}

fn parse_not_eols(lexer : &mut Lexer, discard:bool) -> bool {
    // not_eols => ^eof*

    //
    lexer.debug_label_push("not_eol");

    let mut found=false;

    //
    while lexer.has(0, ["\n","\r\n"]).is_none() && !lexer.is_end() {
        if discard {
            lexer.skip(1);
        } else {
            lexer.consume(1, None);
        }

        found=true;
    }

    //
    lexer.debug_label_pop();

    found
}

fn parse_sep(lexer : &mut Lexer) -> bool {
    // sep => ([\s\t]|[\\]([\r][\n]|[\n]))+

    //
    lexer.debug_label_push("sep");

    //
    let mut found = false;

    while let Some(x)=lexer.has(0, [" ","\t","\\\n","\\\r\n"]) {
        lexer.skip(x.len());
        found=true;
    }

    //
    lexer.debug_label_pop();
    found
}

fn parse_spc(lexer : &mut Lexer) -> bool {
    // spc => [\s\t]+

    //
    lexer.debug_label_push("spc");


    //
    let mut found = false;

    while let Some(x)=lexer.has(0, [" ","\t"]) {
        lexer.skip(x.len());
        found=true;
    }

    //
    lexer.debug_label_pop();
    found
}

fn parse_ending(lexer : &mut Lexer) -> bool {
    // ending => spc? (eol|eof)

    //
    lexer.debug_label_push("ending");

    //
    lexer.push();

    //
    parse_spc(lexer);

    if !parse_eol(lexer) && !lexer.is_end() {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    //
    lexer.pop_keep();
    lexer.debug_label_pop();
    true
}


fn parse_cmnt(lexer : &mut Lexer
    // , cmnt_prefix:&str
) -> bool {
    // cmnt => spc? [#] not_eols ending

    //should comments require proper indenting? no since they can be placed anywhere, eg commenting out a bunch of records

    //
    lexer.debug_label_push("cmnt");
    lexer.push();

    //
    parse_spc(lexer);

    //

    if lexer.has(0, ["#"]).is_none() {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    lexer.skip(1);

    //
    while lexer.has(0, ["\r","\n"]).is_none() && lexer.skip(1).is_some() {
    }

    //
    if !parse_ending(lexer) {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    //
    lexer.pop_keep();
    lexer.debug_label_pop();
    true
}




fn parse_ml_cmnt(lexer : &mut Lexer,start:&str,end:&str) -> bool {
    // ml_cmnt => spc? "#!" (^"!#"|"\\!")* "!#" ending

    //
    lexer.debug_label_push("ml_cmnt");
    lexer.push();

    //
    parse_spc(lexer);

    //

    if lexer.has(0, [start]).is_none() {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    lexer.skip(2);


    let end0=end.chars().next().unwrap().to_string();

    //
    while (lexer.has(0, [end]).is_none() && lexer.skip(1).is_some())
        // || (lexer.has(0, ["\\!"]).is_some() && lexer.skip(2).is_some())
        || (lexer.has(0, ["\\"]).is_some() && lexer.has(1, [end0.as_str()]).is_some() && lexer.skip(2).is_some())
    {
    }

    //
    if lexer.has(0, [end]).is_none() {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    lexer.skip(2);

    //
    if !parse_ending(lexer) {
        lexer.pop_discard();
        lexer.debug_label_pop();
        return false;
    }

    //
    lexer.pop_keep();
    lexer.debug_label_pop();

    true
}