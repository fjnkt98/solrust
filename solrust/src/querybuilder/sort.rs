pub struct SortOrderBuilder {
    order: Vec<String>,
}

impl SortOrderBuilder {
    pub fn new() -> Self {
        Self { order: Vec::new() }
    }

    pub fn build(&self) -> String {
        self.order.join(",")
    }

    pub fn asc(mut self, field: &str) -> Self {
        self.order.push(format!("{} asc", field));
        self
    }

    pub fn desc(mut self, field: &str) -> Self {
        self.order.push(format!("{} desc", field));
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_build_sort_order() {
        let sort = SortOrderBuilder::new().desc("score").asc("name").build();

        assert_eq!(String::from("score desc,name asc"), sort);
    }
}
