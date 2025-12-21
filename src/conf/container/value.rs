use std::any::Any;
use std::path::Path;


use super::super::Conf;
use super::super::super::lexer::Loc;

#[derive(Clone,Copy,Default)]
pub struct ValueContainer<'a> {
    pub(super) conf : Option<&'a Conf>,
    pub(super) conf_value_ind : usize,
}

impl<'a> std::fmt::Debug for ValueContainer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Value({self})")
    }
}
impl<'a> std::fmt::Display for ValueContainer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl<'a> ValueContainer<'a> {
    pub fn start_loc(&self) -> Loc {
        if self.conf.is_none() {return Loc::zero();};
        let val=self.conf.unwrap().values.get(self.conf_value_ind).unwrap();
        val.start_loc
    }

    pub fn end_loc(&self) -> Loc {
        if self.conf.is_none() {return Loc::zero();};
        let val=self.conf.unwrap().values.get(self.conf_value_ind).unwrap();
        val.end_loc
    }

    pub fn path(&self) -> Option<&'a Path> {
        if self.conf.is_none() {return Default::default();};
        self.conf.unwrap().path.as_ref().and_then(|x|Some(x.as_path()))
    }

    // pub fn escaped_string<'s>(&self,func:fn(&'a str)->Option<String>) -> Option<String> {
    //     let Some(s) = self.get_str() else {return  None;};

    //     let mut lexer=Lexer::new(s);
    //     let mut i=0;
    //     let mut out=String::new();

    //     loop {
    //         // let c=lexer.get(0, 2);
    //         if Some('\\')==lexer.getc(i) {
    //             let c=lexer.getc(i+1);

    //             if let Some(r)=func(c) { // what if \ \r\n

    //             } else {

    //             }
    //         }

    //     }

    //     Some(out)
    // }

    pub fn get_str(&self) -> Option<&'a str> {
        if self.conf.is_none() {return None;};
        let val=self.conf.unwrap().values.get(self.conf_value_ind).unwrap();
        let text=self.conf.unwrap().texts.get(val.text_ind).unwrap();
        Some(text.as_str())
    }

    pub fn str(&self) -> &'a str { //if record.get_value(x) use .str() as a value will always have a str
        self.get_str().unwrap_or_default()
    }

    pub fn get_parsed<T:Any+Clone>(&self) -> Option<T> {
        if self.conf.is_none() {return None;};
        let val=self.conf.unwrap().values.get(self.conf_value_ind).unwrap();
        let parsed=val.parsed_ind.map(|parsed_ind|self.conf.unwrap().parsed_values.get(parsed_ind).unwrap());
        parsed.and_then(|x|x.1.downcast_ref::<T>()).map(|x|x.clone())
    }

    pub fn parsed<T:Any+Default+Clone>(&self) -> T {
        self.get_parsed().unwrap_or_default()
    }

    pub fn parsed_or<T:Any+Clone>(&self,d:T) -> T {
        self.get_parsed().unwrap_or(d)
    }
    pub fn parsed_name(&self) -> Option<&'static str> {
        if self.conf.is_none() {return None;};
        let val=self.conf.unwrap().values.get(self.conf_value_ind).unwrap();
        let parsed=val.parsed_ind.map(|parsed_ind|self.conf.unwrap().parsed_values.get(parsed_ind).unwrap());
        parsed.map(|x|x.0)
    }

    pub fn is_empty(&self) -> bool {
        self.conf.is_none()
    }
}
