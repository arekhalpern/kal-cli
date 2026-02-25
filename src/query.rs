use std::collections::BTreeMap;

pub struct QueryParams {
    params: BTreeMap<String, String>,
}

impl QueryParams {
    pub fn new() -> Self {
        Self {
            params: BTreeMap::new(),
        }
    }

    pub fn insert(mut self, key: &str, value: impl ToString) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn limit(self, n: usize) -> Self {
        self.insert("limit", n)
    }

    pub fn optional<T>(self, key: &str, value: Option<T>) -> Self
    where
        T: ToString,
    {
        match value {
            Some(value) => self.insert(key, value),
            None => self,
        }
    }

    pub fn build(self) -> Option<BTreeMap<String, String>> {
        if self.params.is_empty() {
            None
        } else {
            Some(self.params)
        }
    }

    pub fn build_always(self) -> BTreeMap<String, String> {
        self.params
    }
}
