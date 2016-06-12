use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::collections::HashMap;
use std::io::Write;

pub const MOVIES:     i32 = 0x1;
pub const TESTMOVIES: i32 = 0x1000;

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);


#[derive(Debug)]
pub struct IdHandler {
    movie_ids: HashMap<String, i32>,
    movie_id_max: i32
}

impl IdHandler {
    pub fn new(sections: i32) -> IdHandler {
        let mut handler = IdHandler { movie_ids: HashMap::new(), movie_id_max: 0 };
        if (sections & MOVIES) != 0 {
            handler.generate_movie_ids();
        }
        return handler;
    }

    fn generate_movie_ids(&mut self) {
        println_stderr!("Loading ids...");
        let mut highest_value = 0;
        let mut counter = 0;
        self.movie_ids = HashMap::new();
        let f = File::open("data/movies_ids.dat").unwrap();
        let file = BufReader::new(&f);
        for line in file.lines() {
            let line = line.unwrap();
            let parts : Vec<&str> = line.split("\t").collect();
            let id = parts[0].to_owned().parse::<i32>().unwrap();
            let name = parts[1].to_owned();
            self.movie_ids.insert(name, id);
            if id > highest_value {
                highest_value = id;
            }
            if counter % 100000 == 0 {
                println_stderr!("{}     {}: {}", counter, parts[0], parts[1]);
            }
            counter += 1;
        }

        self.movie_id_max = 10000000;
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
