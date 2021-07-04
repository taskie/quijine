use crate::CFunctionListEntry;
use std::{
    collections::{hash_map::Entry, HashMap},
    ffi::CString,
};

#[derive(Default)]
pub struct CStringArena {
    c_strings: HashMap<String, CString>,
}

impl CStringArena {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn registered(&mut self, s: String) -> &CString {
        match self.c_strings.entry(s.clone()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(CString::new(s).unwrap()),
        }
    }

    pub fn get(&self, s: &str) -> Option<&CString> {
        self.c_strings.get(s)
    }
}

#[derive(Default)]
pub struct DefArena {
    function_lists: HashMap<String, Vec<CFunctionListEntry>>,
}

impl DefArena {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn registered_function_list(
        &mut self,
        key: &str,
        function_list: Vec<CFunctionListEntry>,
    ) -> &Vec<CFunctionListEntry> {
        match self.function_lists.entry(key.to_owned()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(function_list),
        }
    }

    pub fn function_list(&self, key: &str) -> Option<&Vec<CFunctionListEntry>> {
        self.function_lists.get(key)
    }
}
