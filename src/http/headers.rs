use std::collections::HashMap;

#[derive(Debug)]
pub struct Headers {
    data: HashMap<String, Vec<String>>,
}

impl Headers {
    pub fn new() -> Headers {
        Headers {
            data: HashMap::<String, Vec<String>>::new()
        }
    }

    pub fn parse(&mut self, header: String) -> &Self {
        let header: Vec<_> = header.split(": ").collect();
        let name = header[0];

        for value in header[1].split(",") {
            let mut vec = self.data.entry(name.to_string()).or_insert(Vec::<String>::new());
            vec.push(value.to_string());
        }

        self
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    None
                }
                else {
                    Some(&values[0])
                }
            },
            None => None
        }
    }

    pub fn get_default(&self, name: &str, default: String) -> String {
        let default = default.to_string();
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    default
                }
                else {
                    values[0].clone()
                }
            },
            None => default
        }
    }

    pub fn insert(&mut self, name: &str, value: &str) {
        let mut vec = self.data.entry(name.to_string()).or_insert(Vec::<String>::new());
        vec.push(value.to_string());
    }
}
