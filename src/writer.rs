

//todo implement some kind of write trait/func that writes the values using double quotes
//  with any double quote chars escaped
//  and any escape chars before a quote also escaped
pub struct Writer {
    buffer :String,
    // record_indent: Option<usize>,
    record_indent: usize,
    param_used:bool,
    // cur_params:Vec<String>,
    last_newline:bool,
}

impl std::fmt::Display for Writer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}",self.buffer)
    }
}

impl Writer {
    pub fn new() -> Self {
        Self {
            buffer:String::new(),
            // record_indent:None,
            record_indent:0,
            param_used:false,
            last_newline:true,
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.record_indent=0;
        self.param_used=false;
        self.last_newline=true;
    }

    pub fn comment<T:ToString>(&mut self,indent:usize,val:T) -> &mut Self {
        if !self.last_newline {
            self.buffer.push('\n');
        }

        self.record_indent=0;
        self.param_used=false;
        self.last_newline=true;

        let indent="    ".repeat(indent);
        let val=val.to_string();

        self.buffer.push_str(indent.as_str());
        self.buffer.push_str("#");


        for c in val.chars() {
            if let Some(x)=match c {
                '\n' => Some("\\n"),
                '\r' => Some("\\r"),
                _=>None,
            } {
                self.buffer.push_str(x);
            } else {
                self.buffer.push(c);
            }
        }

        self.buffer.push('\n');
        self
    }

    pub fn newline(&mut self,indent:usize) -> &mut Self {
        if !self.last_newline {
            self.buffer.push('\n');
        }

        self.record_indent=0;
        self.param_used=false;
        self.last_newline=true;

        let indent="    ".repeat(indent);

        self.buffer.push_str(indent.as_str());
        self.buffer.push_str("\n");

        self
    }

    pub fn text<T:ToString>(&mut self,indent:usize,val:T) -> &mut Self {
        if !self.last_newline {
            self.buffer.push('\n');
        }

        self.record_indent=0;
        self.param_used=false;
        self.last_newline=true;

        let indent="    ".repeat(indent);
        let val=val.to_string();

        let mut lines: Vec<String>=vec![indent.clone()];

        for c in val.chars() {
            match c {
                '\n' => {
                    lines.last_mut().unwrap().push('\n');
                    lines.push(indent.clone());
                }
                '\r' => {

                }
                _ => {
                    lines.last_mut().unwrap().push(c);
                }
            }
        }

        for line in lines {
            self.buffer.push_str(line.as_str());
        }

        self.buffer.push('\n');

        self
    }

    pub fn record(&mut self,indent:usize) -> &mut Self {
        self.param_used=false;
        self.record_indent=indent;
        // self.record_indent=Some(indent);
        self
    }

    fn inner_param<T:ToString>(&mut self,quote:&str,val:T) {
        let quote_chars: Vec<char>=quote.chars().collect();
        let val=val.to_string();

        //
        if self.param_used {
            self.buffer.push(' ');
        } else {
            if !self.last_newline {
                self.buffer.push('\n');
            }

            let indent="    ".repeat(self.record_indent);
            self.buffer.push_str(&indent);
        }

        //
        if !quote_chars.is_empty() {
            self.buffer.push_str(quote);
        }

        //
        if quote_chars.is_empty() {
            for c in val.chars() {
                let x=match c {
                    '\n' => Some("\\n"),
                    '\r' => Some("\\r"),
                    ' ' => Some("\\ "),
                    '\t'=> Some("\\t"),
                    '"'=> Some("\\\""),
                    '\''=> Some("\\'"),
                    '`'=> Some("\\`"),
                    _ => None,
                };

                if let Some(x)=x {
                    self.buffer.push_str(x);
                } else {
                    self.buffer.push(c);
                }
            }
        } else {
            let val: Vec<char>=val.chars().collect();
            for i in 0..val.len() {
                let c = val[i];
                let check=val.get(i..(i+quote_chars.len())).map(|x|x.to_vec()).unwrap_or_default();

                if quote_chars==check {
                    self.buffer.push('\\');
                    self.buffer.push(c);
                } else {
                    self.buffer.push(c);
                }
            }
        }


        //
        if !quote_chars.is_empty() {
            self.buffer.push_str(quote);
        }

        //
        self.param_used=true;
        self.record_indent=0;
        self.last_newline=false;

    }

    pub fn param<T:ToString>(&mut self,val:T) -> &mut Self {
        self.inner_param("", val);
        self
    }

    pub fn params<T:IntoIterator<Item=impl ToString>>(&mut self,vals:T) -> &mut Self {
        for val in vals.into_iter() {
            self.param(val);
        }

        self
    }
    pub fn param_squote<T:ToString>(&mut self,tripple:bool,val:T) -> &mut Self {
        self.inner_param("'".repeat(if tripple {3}else{1}).as_str(), val);
        self
    }

    pub fn param_squotes<T:IntoIterator<Item=impl ToString>>(&mut self,tripple:bool,vals:T) -> &mut Self {
        for val in vals.into_iter() {
            self.param_squote(tripple,val);
        }

        self
    }

    pub fn param_dquote<T:ToString>(&mut self,tripple:bool,val:T) -> &mut Self {
        self.inner_param("\"".repeat(if tripple {3}else{1}).as_str(), val);
        self
    }

    pub fn param_dquotes<T:IntoIterator<Item=impl ToString>>(&mut self,tripple:bool,vals:T) -> &mut Self {
        for val in vals.into_iter() {
            self.param_dquote(tripple,val);
        }

        self
    }

    pub fn param_bquote<T:ToString>(&mut self,tripple:bool,val:T) -> &mut Self {
        self.inner_param("`".repeat(if tripple {3}else{1}).as_str(), val);
        self
    }


    pub fn param_bquotes<T:IntoIterator<Item=impl ToString>>(&mut self,tripple:bool,vals:T) -> &mut Self {
        for val in vals.into_iter() {
            self.param_bquote(tripple,val);
        }

        self
    }
}
