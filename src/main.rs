#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate chrono;
mod movies;
mod id_handler;

use movies::Movies;
use id_handler::IdHandler;


fn main() {
    let mut id_handler = IdHandler::new(id_handler::MOVIES);
    let mut movies = Movies::new();

    movies.parse_file(&mut id_handler);
}
