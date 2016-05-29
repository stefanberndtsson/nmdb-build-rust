use std::collections::HashMap;

#[derive(Debug)]
pub struct IdHandler {
    movie_ids: HashMap<String, i32>,
    movie_id_max: i32
}

impl IdHandler {
    pub fn new() -> IdHandler {
        let mut handler = IdHandler { movie_ids: HashMap::new(), movie_id_max: 0 };
        handler.generate();
        return handler;
    }

    pub fn generate(&mut self) {
        self.movie_ids = HashMap::new();
        self.movie_id_max = 10;
        self.new_movie_id(String::from("\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}"));
    }

    pub fn new_movie_id(&mut self, key: String) -> i32 {
        let new_id = self.movie_id_max + 1;
        self.movie_ids.insert(key, new_id);
        self.movie_id_max = new_id;
        return new_id;
    }
    
    pub fn find_or_generate_movie_id(&mut self, title: &String) -> i32 {
        let mut new_id = 0;
        let mut set_new_id = false;
        match self.movie_ids.get(title) {
            Some(id) => {
                new_id = *id;
            },
            None => {
                set_new_id = true;
            }
        }
        if set_new_id {
            new_id = self.new_movie_id(title.clone());
        }
        return new_id;
    }
}
