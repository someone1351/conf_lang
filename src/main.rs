use std::{collections::HashMap, path::{Path, PathBuf}};

use conf_def::{ Conf, RecordContainer, Walk};

fn walk_test1_def() -> conf_def::Def {
    conf_def::Def::new()
        .branch("root_branch")
            .tag_nodes(["text"])
                .entry_text(None) 
            .tag_nodes(["include"])
                .entry(None)
                    .param_any()
            .tag_nodes(["hello"])
                .rentry_children(None,"root_branch")
                    .group(Some("ints"),true,true) 
                        .param_parse::<i32>()
                        .param_parse::<i32>()
                    .group(Some("the opt int"),true,false)
                        .param_parse::<i32>()
                    .group(Some("the int"),false,false)
                        .param_parse::<i32>()
                        .param_parse::<i32>()
                    .group(Some("the any"),false,false)
                        .param_any()
            .tag_nodes(["functest"])
                .entry(None)
                    .group(None, false, true)
                        .param_func(|x|match x {"a"=>Some(111),"b"=>Some(222),"c"=>Some(333),_=>None})
            .tagless_nodes()
                .entry(Some("somevals"))
                    .group(None,false,true)
                        .param_parse::<i32>()
        //     .insert_nodes("rest_branch")
        // .branch("rest_branch").tagless_nodes().entry_children(Some("rest"),"rest_branch").group(None, false, true).any()
}

fn walk_test1() {
    let def = walk_test1_def();
    let confs=load_confs(def,"examples/test1");

    let Some(test_conf)=confs.get(&PathBuf::from("examples/test1/test.conf")) else {
        return;
    };

    let res=test_conf.0.root().walk_ext::<&str>( |walk|{
        let record=walk.record();
        println!("{}",get_record_info(&walk));

        match record.tag() {
            Some("include") if walk.is_enter() => { //include records from another file
                let mut include_path=record.path().unwrap().to_path_buf();
                include_path.pop();
                include_path.push(record.value(0).as_str());
    
                return if let Some(conf_data)=confs.get(&include_path) {
                    Ok(Some(conf_data.0.root()))
                } else {
                    Err(("include file not found",Some(record.value(0).start_loc())))
                };
            }
            Some("hello") if walk.is_enter() => {
                println!("    {}",get_group_vals_info(record));
                println!("    the int values are: {}",record.param_group("ints").values().parsed::<i32>().map(|x|format!("{x:?}")).collect::<Vec<_>>().join(", "));
                println!("    any val is {:?}",record.param_group("the any").value(0).as_str());
            }
            Some("functest") if walk.is_enter() => {
                println!("    functest: {:?}",record.values().parsed().collect::<Vec<i32>>());
            }
            _ =>{}
        }

        Ok(None)
    });

    if let Err(e)=res {
        println!("{}",e.msg(e.path.as_ref().and_then(|p|confs.get(p)).map(|x|x.1.as_str())));
    }
}

fn walk_test2() {
    let def = conf_def::Def::new()
        .param_parse::<i32>()
        .param_parse::<i32>()
        ;

    // let def = conf_def::Def::new()
    //     .group(None,false,false)
    //         .parse::<i32>()
    //         .parse::<i32>()
    //     ;

    // let def = conf_def::Def::new()
    //     .entry(None)
    //         .parse::<i32>()
    //         .parse::<i32>()
    //     ;

    // let def = conf_def::Def::new()
    //     .branch("a")
    //         .parse::<f32>()
    //     .branch("b")
    //         .parse::<bool>()
    //     .branch("a")
    //         .entry(None)
    //             .parse::<i32>()
    //             .parse::<i32>()
    //     ;

    let confs=load_confs(def,"examples/test2");

    let Some(test_conf)=confs.get(&PathBuf::from("examples/test2/test.conf")) else {
        return;
    };

    let res=test_conf.0.root().walk_ext::<&str>( |walk|{
        println!("{}",get_record_info(&walk));
        Ok(None)
    });

    if let Err(e)=res {
        println!("{}",e.msg(e.path.as_ref().and_then(|p|confs.get(p)).map(|x|x.1.as_str())));
    }
}

fn main() {
    walk_test1();
    println!("===");
    walk_test2();
}

fn load_confs<P: AsRef<Path>>(def:conf_def::Def,dir:P) -> HashMap<PathBuf, (Conf,String)> {
    let root_branch=def.get_root_branch(); //.get_branch("root_branch");

    let file_paths = Vec::from_iter(std::fs::read_dir(dir).unwrap().filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension() == Some(std::ffi::OsStr::new("conf"))),
    );

    let mut confs=HashMap::new();

    for file_path in file_paths {
        let src = std::fs::read_to_string(file_path.as_path()).unwrap();

        match root_branch.parse(src.as_str(), true,Some(file_path.as_path())) {
            Ok(conf)=>{
                confs.insert(file_path, (conf,src));
            }
            Err(e)=>{
                println!("{}",e.msg(Some(src.as_str())));
            }
        }
    }

    confs
}

fn get_record_info(walk:&Walk) -> String {
    let record=walk.record();
    format!("{} {}{}: {}[{:}]{} @ ({}:{}:{})",
        walk.is_exit().then_some("<=").unwrap_or("=>"),
        "   ".repeat(walk.depth()),
        walk.order(),
        record.tag().map(|x|format!("{x:?} : ")).unwrap_or_default(),
        record.values().map(|x|format!("{:?}",x.as_str())).collect::<Vec<_>>().join(", "),
        record.has_text().then(||format!(" : {:?} :",record.text_values().map(|x|x.as_str()).collect::<Vec<_>>().join("\n"))).unwrap_or_default(),
        record.branch_name().map(|x|format!("{x}")).unwrap_or("_".to_string()),
        record.node_label().map(|x|format!("{x}")).unwrap_or("_".to_string()),
        record.path().map(|x|format!("{x:?}")).unwrap_or("_".to_string()),
    )
}

fn get_group_vals_info(record:RecordContainer) -> String {
    format!("{}",
        (0..record.param_groups_num()).map(|param_group_ind|{
            let param_group=record.param_group(param_group_ind);
            format!("{}:{}",
                param_group.name().map(|x|format!("{x:?}")).unwrap_or("_".to_string()), {
                let x=(0..param_group.many_num()).map(|many_ind|{
                    let y=(0..param_group.params_num())
                        .map(|param_ind|param_group.value(many_ind*param_group.params_num()+param_ind).as_str())
                        .collect::<Vec<_>>().join(", ");
                    if param_group.params_num()==1 {format!("{y}",)} else {format!("({y})",)}
                }).collect::<Vec<_>>().join(", ");
                if !param_group.is_repeat() {format!("{x}",)} else {format!("[{x}]",)}
            })
        }).collect::<Vec<_>>().join(", ")
    )
}
