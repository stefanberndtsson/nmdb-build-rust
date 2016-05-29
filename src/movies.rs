use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use regex::Regex;
use chrono::*;
use id_handler::IdHandler;

#[derive(Debug)]
#[derive(Clone)]
struct Movie {
    id: i32,
    full_title: String,
    full_year: String,

    title: String,
    title_year: String,
    title_category: String,
    years: Vec<i32>,
    year_open_end: bool,
    
    is_episode: bool,
    episode_name: String,
    episode_season: String,
    episode_episode: String,
    episode_parent_title: String,

    suspended: bool
}

impl Movie {
    fn new(mut id_handler: &mut IdHandler, line: &str) -> Movie {
        let mut movie = Movie {
            id: -1, full_title: "".to_owned(), full_year: "".to_owned(),
            title: "".to_owned(), title_year: "".to_owned(), title_category: "".to_owned(),
            years: vec![], year_open_end: false,
            is_episode: false,
            episode_name: "".to_owned(), episode_season: "".to_owned(), episode_episode: "".to_owned(),
            episode_parent_title: "".to_owned(), suspended: false
        };
        
        return movie.parse_line(id_handler, line);
    }
    
    fn parse_line(&mut self, id_handler: &mut IdHandler, line: &str) -> Movie {
        println!("Line: {}", line);
        self.extract_full_title(line);
        self.extract_full_year(line);
        self.extract_year();
        let mut remaining_title = self.extract_episode().to_owned();
        remaining_title = self.extract_title_year(&remaining_title);
        self.extract_title(&remaining_title);
        self.set_id(id_handler);
        return self.clone();
    }

    fn set_id(&mut self, id_handler: &mut IdHandler) {
        self.id = id_handler.find_or_generate_movie_id(&self.full_title);
    }
    
    fn extract_full_title(&mut self, line: &str) {
        lazy_static! {
            static ref TABSPLIT: Regex = Regex::new(r"\t+").unwrap();
        }        
        let parts: Vec<&str> = TABSPLIT.split(line).collect();
        self.full_title = parts[0].to_owned();
    }

    fn extract_full_year(&mut self, line: &str) {
        lazy_static! {
            static ref TABSPLIT: Regex = Regex::new(r"\t+").unwrap();
        }        
        let parts: Vec<&str> = TABSPLIT.split(line).collect();
        self.full_year = parts[1].to_owned();
    }

    fn extract_title(&mut self, line: &str) {
        self.title = line.to_owned();
    }
    
    fn extract_title_year(&mut self, line: &str) -> String {
        lazy_static! {
            static ref TITLEYEAR_WITH_CODE: Regex = Regex::new(r"\((..../[IVX]+)\)$").unwrap();
            static ref TITLEYEAR_WITHOUT_CODE: Regex = Regex::new(r"\((....)\)$").unwrap();
        }
        let mut title_year = "";
        if TITLEYEAR_WITH_CODE.is_match(line) {
            for cap in TITLEYEAR_WITH_CODE.captures_iter(line) {
                title_year = cap.at(1).unwrap_or("");
            }
        } else if TITLEYEAR_WITHOUT_CODE.is_match(line) {
            for cap in TITLEYEAR_WITHOUT_CODE.captures_iter(line) {
                title_year = cap.at(1).unwrap_or("");
            }
        } else {
            title_year = "";
        }
        self.title_year = title_year.to_owned();
        let title_length = line.len() - self.title_year.len();
        return line[0..title_length-3].to_owned();
    }

    fn extract_year(&mut self) {
        let mut start_year = 0;
        let mut end_year = 0;
        lazy_static! {
            static ref YEAR_RE_SINGLE: Regex = Regex::new(r"^(\d\d\d\d)$").unwrap();
            static ref YEAR_RE_CLOSED: Regex = Regex::new(r"^(\d\d\d\d)-(\d\d\d\d)$").unwrap();
            static ref YEAR_RE_OPEN: Regex = Regex::new(r"^(\d\d\d\d)-(\?\?\?\?)$").unwrap();
        }
        if self.full_year == "" {
            start_year = -1;
            end_year = -1;
        } else if YEAR_RE_SINGLE.is_match(&self.full_year) {
            start_year = self.full_year.parse::<i32>().unwrap();
            end_year = start_year;
        } else if YEAR_RE_OPEN.is_match(&self.full_year) {
            for cap in YEAR_RE_OPEN.captures_iter(&self.full_year) {
                start_year = cap.at(1).unwrap_or("0").parse::<i32>().unwrap();
            }
            let date = Local::now();
            end_year = date.year();
            self.year_open_end = true;
        } else if YEAR_RE_CLOSED.is_match(&self.full_year) {
            for cap in YEAR_RE_CLOSED.captures_iter(&self.full_year) {
                start_year = cap.at(1).unwrap_or("0").parse::<i32>().unwrap();
                end_year = cap.at(2).unwrap_or("0").parse::<i32>().unwrap();
            }
        }
        
        self.years.clear();
        let mut range = (start_year..end_year+1).collect::<Vec<i32>>();
        self.years.append(&mut range);
    }

    fn extract_episode(&mut self) -> String {
        let ep_position;
        match self.full_title.rfind(") {") {
            None => return self.full_title.to_owned(),
            Some(x) => { ep_position = x; } 
        }

        self.is_episode = true;
        let episode_data = self.full_title[ep_position+3..self.full_title.len()-1].to_owned();
        let epval_position;
        match episode_data.rfind(" (#") {
            None => {
                if episode_data.starts_with("(#") {
                    epval_position = 0;
                } else {
                    self.episode_name = episode_data;
                    return self.full_title[0..ep_position+1].to_owned();
                }
            },
            Some(x) => { epval_position = x+1; }
        }

        let epval_data = episode_data[epval_position+2..episode_data.len()-1].to_owned();
        let epval_parts : Vec<&str> = epval_data.split(".").collect();

        if epval_parts.len() == 2 {
            self.episode_season = epval_parts[0].to_owned();
            self.episode_episode = epval_parts[1].to_owned();
            self.episode_name = episode_data[0..epval_position-1].to_owned();
        } else {
            self.episode_name = episode_data;
        }
        self.episode_parent_title = self.full_title[0..ep_position+1].to_owned();
        
        return self.episode_parent_title.to_owned();
    }
}

pub struct Movies {
    movies: Vec<Movie>
}

impl Movies {
    pub fn new() -> Movies {
        return Movies {
            movies: vec![],
        }
    }

    pub fn parse_file(&mut self, mut id_handler: &mut IdHandler) {
        let trigger_line = "MOVIES LIST";
        let mut triggered = false;
        let mut trigger_skip = 3;
        let f = File::open("data/movies.list").unwrap();
        let file = BufReader::new(&f);
        for line in file.lines() {
            if !triggered {
                if line.unwrap() == trigger_line {
                    triggered = true;
                }
                continue;
            }
            if triggered {
                trigger_skip -= 1;
            }
            if trigger_skip > 0 {
                continue;
            }
            let m = Movie::new(&mut id_handler, &line.unwrap());
            println!("Movie: {:?}", m);
            self.movies.push(m);
            break;
        }
    }
    
//    pub fn parse_test(&self, id_handler: &mut IdHandler) {
//        let m = Movie::new(id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
//        println!("Movie: {:?}", m);
//    }
}

#[test]
fn test_movie_full_title() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!("\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}", m1.full_title);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!("\"!Next?\" (1994)", m2.full_title);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!("\"#1 Single\" (2006)", m3.full_title);
}

#[test]
fn test_movie_full_year() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!("2005", m1.full_year);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!("1994-1995", m2.full_year);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!("2006-????", m3.full_year);
}

#[test]
fn test_movie_years() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!(1, m1.years.len());
    assert_eq!(2005, m1.years[0]);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!(2, m2.years.len());
    assert_eq!(1994, m2.years[0]);
    assert_eq!(1995, m2.years[1]);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!(true, m3.years.len() > 9);
    assert_eq!(2006, m3.years[0]);
    assert_eq!(2007, m3.years[1]);
    assert_eq!(2008, m3.years[2]);
    assert_eq!(2009, m3.years[3]);
    assert_eq!(2010, m3.years[4]);
    assert_eq!(2011, m3.years[5]);
    assert_eq!(2012, m3.years[6]);
    assert_eq!(2013, m3.years[7]);
    assert_eq!(2014, m3.years[8]);
    assert_eq!(2015, m3.years[9]);
}

#[test]
fn test_movie_episode() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!(true, m1.is_episode);
    assert_eq!("E. Griff/Ralphie May", m1.episode_name);
    assert_eq!("1", m1.episode_season);
    assert_eq!("3", m1.episode_episode);
    assert_eq!("\"1st Amendment Stand Up\" (2005)", m1.episode_parent_title);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!(false, m2.is_episode);
    assert_eq!("", m2.episode_parent_title);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!(false, m3.is_episode);
    assert_eq!("", m3.episode_parent_title);
}

#[test]
fn test_movie_title_year() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!("2005", m1.title_year);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!("1994", m2.title_year);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!("2006", m3.title_year);
}

#[test]
fn test_movie_title() {
    let mut id_handler = IdHandler::new();
    let m1 = Movie::new(&mut id_handler, "\"1st Amendment Stand Up\" (2005) {E. Griff/Ralphie May (#1.3)}	2005");
    assert_eq!("\"1st Amendment Stand Up\"", m1.title);

    let m2 = Movie::new(&mut id_handler, "\"!Next?\" (1994)					1994-1995");
    assert_eq!("\"!Next?\"", m2.title);

    let m3 = Movie::new(&mut id_handler, "\"#1 Single\" (2006)					2006-????");
    assert_eq!("\"#1 Single\"", m3.title);
}

