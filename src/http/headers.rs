use std::collections::HashMap;
use std::clone;

#[derive(Debug)]
pub struct Headers {
    data: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl Headers {
    pub fn new() -> Headers {
        Headers {
            data: HashMap::<String, Vec<String>>::new()
        }
    }

    pub fn parse(&mut self, header: &str) -> &Self {
        let header: Vec<_> = header.split(": ").collect();
        let name = header[0];

        for value in header[1].split(',') {
            let mut vec = self.data.entry(name.trim().to_owned()).or_insert(Vec::<String>::new());
            vec.push(value.trim().to_owned());
        }

        self
    }

    pub fn insert(&mut self, name: &str, value: &str) {
        let mut vec = self.data.entry(name.to_owned()).or_insert(Vec::<String>::new());
        vec.push(value.to_string());
    }

    fn get(&self, key: &str) -> Option<Vec<String>> {
        match self.data.get(key) {
            Some(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    let vec = vec.clone();
                    Some(vec)
                }
            }
            _ => None
        }
    }

    fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    fn all(&self) -> Vec<(String, Vec<String>)> {
        let mut vec = Vec::<(String, Vec<String>)>::new();
        let data = self.data.clone();

        for (key, val) in data {
            vec.push((key, val));
        }

        vec
    }
}

impl ToString for Headers {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for (key, vec) in &self.data {
            let mut iter = vec.iter();
            match iter.next() {
                Some(i) => result.push_str(&format!("{}: {}", key, i)),
                None => return result,
            }

            for i in vec {
                format!(", {}", i);
            }
        }
        result
    }
}

impl clone::Clone for Headers {
    fn clone(&self) -> Self {
        Headers {
            data: self.data.clone()
        }
    }
}
