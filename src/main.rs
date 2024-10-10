

use std::{collections::HashMap, path::PathBuf};

use conf_def::{ Conf, RecordContainer, Walk};

// mod conf_def;

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

fn create_def() -> conf_def::Def {
    conf_def::Def::new()
        .branch("root_branch")
            .tag_nodes(["text"]).entry().text()
            .tag_nodes(["include"]).entry().any()
            .tag_nodes(["hello"])
                .rentry().children("root_branch")
                    .group().name("ints").repeat() .optional()
                        .parse::<i32>()
                        .parse::<i32>()
                    .group().name("the opt int").optional()
                        .parse::<i32>()
                    .group().name("the int")
                        .parse::<i32>()
                        .parse::<i32>()
                    .group().name("the any").any()
            .tagless_nodes()
                .entry().label("somevals").repeat()
                    .parse::<i32>()

        //     .insert_nodes("rest_branch")
        // .branch("rest_branch").tagless_nodes().entry().children("rest").label("rest").repeat().any()
}

fn load_confs(def:conf_def::Def) -> HashMap<PathBuf, (Conf,String)> {
    let root_branch=def.get_branch("root_branch").unwrap();

    let file_paths = Vec::from_iter(std::fs::read_dir("examples").unwrap().filter_map(Result::ok)
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

fn walk_test() {
    let def = create_def();
    let confs=load_confs(def);

    let Some(test_conf)=confs.get(&PathBuf::from("examples/test.conf")) else {
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
            _ =>{}
        }

        Ok(None)
    });

    if let Err(e)=res {
        println!("{}",e.msg(e.path.as_ref().and_then(|p|confs.get(p)).map(|x|x.1.as_str())));
    }
}


fn main() {
    walk_test();
}
