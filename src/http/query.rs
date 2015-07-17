use std::collections::HashMap;

#[derive(Debug)]
pub struct Query {
    data: HashMap<String, Vec<String>>,
}

#[allow(dead_code)]
impl Query {
    pub fn new() -> Query {
        Query {
            data: HashMap::<String, Vec<String>>::new(),
        }
    }

    pub fn from_str(query_string: &str) -> Query {
        let mut query = Query::new();

        if query_string.trim().len() == 0 {
            return query;
        }

        for q in query_string.split("&") {
            let key_value: Vec<_> = q.split("=").collect();
            let key = key_value[0];
            let value = key_value[1];

            let mut query_vec = query.data.entry(key.to_string()).or_insert(Vec::new());
            query_vec.push(value.to_string());
        }

        query
    }

    pub fn get(&self, name: &str) -> Option<Vec<String>> {
        match self.data.get(name) {
            Some(values) => {
                if values.is_empty() {
                    None
                }
                else {
                    Some(values.clone())
                }
            },
            None => None
        }
    }
}

fn format_query_param(k: &str, v: &Vec<String>) -> String {
    let mut result = String::new();

    let mut k = k.to_string();
    if v.len() > 1 {
        k.push_str("[]");
    }

    let mut iter = v.iter();
    match iter.next() {
        Some(i) => result.push_str(&format!("{}={}", k, i)),
        None => return result,
    };

    for i in iter {
        result.push_str(&format!("&{}={}", k, i));
    }

    result
}

impl ToString for Query {
    fn to_string(&self) -> String {
        let mut result = String::new();

        let mut iter = self.data.iter();
        match iter.next() {
            Some((k, v)) => result.push_str(&format_query_param(k, v)),
            None => return result,
        }

        for (k, v) in iter {
            result.push_str(&format!("&{}", format_query_param(k, v)));
        }

        result
    }
}
