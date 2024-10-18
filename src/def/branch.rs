use std::collections::HashMap;


#[derive (Default)]
pub struct Branch {
    pub tags : HashMap<String,Vec<usize>>, //[tag][tag_node_ind]=node_index
    pub non_tags : Vec<usize>, //[no_tag_node_ind]=node_index
    pub branch_inserts : Vec<String>,
    pub branch_name : Option<String>,
    // ignore_other_errors : bool,
    // tag_onces : HashMap<String,HashSet<String>>,
    // tag_onces : HashSet<String>,
}

impl Branch {
    pub fn new(branch_name : Option<String>) -> Self {
        Self {
            tags : HashMap::new(),
            non_tags : Vec::new(),
            branch_inserts : Vec::new(),
            // ignore_other_errors : false,
            branch_name,
            // tag_onces : HashMap::new(),
            // tag_onces : HashSet::new(),
        }
    }
}
