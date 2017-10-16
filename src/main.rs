extern crate regex;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::error::Error;
use std::env;
use std::str;
use std::slice::Iter;
use std::iter::Peekable;
use std::collections::HashMap;

enum Entry{
    Calendar(Option<VCALENDAR>),
    Timezone(Option<VTIMEZONE>),
    Event(Option<VEVENT>),
}

struct VCALENDAR {
    version: Option<String>,
    method: Option<String>,
    prodid: Option<String>,
    timezone: Option<VTIMEZONE>,
    events: Vec<VEVENT>,
    unknown: Vec<(String, String)>,
}


struct VEVENT{
    uid: Option<String>,
    class: Option<String>,
    summary: Option<String>,
    sequenz: Option<String>,
    dtstamp: Option<String>,
    location: Option<String>,
    categories: Option<String>,
    description: Option<String>,
    unknown: Vec<(String, String)>,
}

struct TZ{
    offsetfrom: Option<String>,
    offsetto: Option<String>,
    name: Option<String>,
    dtstart: Option<String>,
    rrule: Option<String>,
}

struct VTIMEZONE{
    tzid: Option<String>,
    definitions: HashMap<String, TZ>,
    unknown: Vec<(String, String)>,
}



fn check_format(input: &(&str, &str), expected: &str){
    if input.0 != expected{
        panic!("expected {}, got {}", expected, input.0);
    }
}


fn read_begin(begin: &(&str,&str) , mut entries: &mut Peekable<Iter<(&str,&str)>>) -> Option<Entry>{

    check_format(&begin, "BEGIN");

    println!("[BEGIN]");

    println!("\tBEGIN: reading {}", begin.1);

    match begin.1 {
        "VEVENT" => Some(Entry::Event(read_vevent(&mut entries))),
        "VCALENDAR" => Some(Entry::Calendar(read_vcalendar(&mut entries))),
        "VTIMEZONE" => Some(Entry::Timezone(read_vtimezone(&mut entries))),
        x => {
            println!("\tBEGIN: Not yet parsing {:?}", x);
            return None;
        },
    }


}

fn read_tz(name:&str, entries: &mut Peekable<Iter<(&str,&str)>>) -> Option<TZ> {
    println!("[TZ]");

    let mut this = TZ{
        offsetfrom: None,
        offsetto: None,
        name: None,
        dtstart: None,
        rrule: None,
    };

    while true {
        let e;
        {
            if let Some(entry) = entries.next(){
                e = entry;
            } else {
                return Some(this);
            }

        }

        if e.0 == "END" && e.1 == name {
            return Some(this);
        }

        println!("\tTZ: reading {} {}", e.0, e.1);
        match e.0 {
            "TZOFFSETFROM" => this.offsetfrom = Some(e.1.to_string()),
            "TZOFFSETTO" => this.offsetto = Some(e.1.to_string()),
            "TZNAME" => this.name = Some(e.1.to_string()),
            "DTSTART" => this.dtstart = Some(e.1.to_string()),
            "RRULE" => this.rrule = Some(e.1.to_string()),
            x => {
                println!("\tTZ: Unhandled/Unexpected {}", x);
            },
        }

    }
    return None;
}

fn read_vtimezone(entries: &mut Peekable<Iter<(&str,&str)>>) -> Option<VTIMEZONE>{
    println!("[VTIMEZONE]");

    let mut this = VTIMEZONE{
        tzid: None,
        definitions: HashMap::new(),
        unknown: Vec::new(),
    };

    while true {
        let e;
        {
            if let Some(entry) = entries.next(){
                e = entry;
            } else {
                return Some(this);
            }

        }

        if e.0 == "END" && e.1 == "VTIMEZONE" {
            return Some(this);
        }

        println!("\tTIMEZONE: reading {} {}", e.0, e.1);
        match e.0 {
            "BEGIN" => {
                this.definitions.insert(e.0.to_string(), read_tz(e.1,entries).unwrap());
            },
            "TZID" => this.tzid = Some(e.1.to_string()),
            x => {
                println!("\tTIMEZONE: unknown {}", x);
                this.unknown.push((e.0.to_string(),e.1.to_string()));
            },
        }

    }
    return None;
}

fn read_vcalendar(entries: &mut Peekable<Iter<(&str,&str)>>) -> Option<VCALENDAR>{

    println!("[VCALENDAR]");

    let mut this = VCALENDAR {
        method: None,
        prodid: None,
        timezone: None,
        version: None,
        unknown: Vec::new(),
        events: Vec::new(),
    };

    while true {
        let e;
        {
            if let Some(entry) = entries.next(){
                e = entry;
            } else {
                return Some(this);
            }

        }

        if e.0 == "END" && e.1 == "VCALENDAR" {
            return Some(this);
        }

        println!("\tCALENDAR: reading {}", e.0);

        match e.0 {
            "BEGIN" => if let Some(object) = read_begin(e,entries) {
                match object {
                    Entry::Calendar(_) => {panic!("Calendar inside a calendar!");}
                    Entry::Timezone(tz) => {this.timezone = tz;}
                    Entry::Event(ev) => if let Some(ev) = ev {this.events.push(ev);}
                }
            }
            "METHOD" => this.method = Some(e.1.to_string()),
            "PRODID" => this.prodid = Some(e.1.to_string()),
            "VERSION" => this.version = Some(e.1.to_string()),
            x => {
                println!("\tCALENDAR: unknown {}", x);
                this.unknown.push((e.0.to_string(),e.1.to_string()));
            },
        }

    }
    return None;
}

fn read_vevent(entries: &mut Peekable<Iter<(&str,&str)>>) -> Option<VEVENT> {
    println!("[VEVENT]");
    let mut this = VEVENT {
        uid: None,
        class: None,
        summary: None,
        sequenz: None,
        dtstamp: None,
        location: None,
        categories: None,
        description: None,
        unknown: Vec::new(),
    };

    while true {
        let e;
        {
            if let Some(entry) = entries.next(){
                e = entry;
            } else {
                return Some(this);
            }

        }

        if e.0 == "END" && e.1 == "VEVENT" {
            return Some(this);
        }

        println!("\tEVENT: reading {} {}", e.0, e.1);
        match e.0 {
            "BEGIN" => panic!("Unexpected begin inside VEVENT!"),
            "UID" => this.uid = Some(e.1.to_string()),
            "SUMMARY" => this.summary = Some(e.1.to_string()),
            "SEQUENCE" => this.sequenz = Some(e.1.to_string()),
            "DTSTAMP"  => this.dtstamp = Some(e.1.to_string()),
            "LOCATION" => this.location = Some(e.1.to_string()),
            "CATEGORIES" => this.categories = Some(e.1.to_string()),
            "DESCRIPTION" => this.description = Some(e.1.to_string()),
            "CLASS" => this.class = Some(e.1.to_string()),
            x => {
                println!("\tEVENT: unknown {}", x);
                this.unknown.push((e.0.to_string(),e.1.to_string()));
            },
        }

    }
    return None;
}

fn main() {
  
    // read cmdlines
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        panic!("Please supply ical file to be parsed");
    }
    
    // open and check file
    let ref filename = args[1];
    println!("Opening {}", filename);
    let f = match File::open(filename) {
        Err(why) => panic!("couldn't open {}: {}", filename, why.description()),
        Ok(file) => file,
    };

    // read file to buffer
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();

    match reader.read_to_string(&mut buffer){
        Err(why) => panic!("couldn't read line: {}", why),
        Ok(_) => {},
    };    
    

    /* lexing */ 
    let mut entries: Vec<(&str, &str)>  = Vec::new();

    // match regex
    let re = regex::Regex::new(r"(?m)^([[[:upper:]]|-]+):([\w[\t\v\f ]]*)").unwrap();

    //TODO use map
    for caps in re.captures_iter(&buffer) {
        entries.push((caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str()));
    }

    let entries = &mut entries.iter().peekable();

    let e;
    {
        if let Some(entry) = entries.next(){
            e = entry;
        } else {
            panic!("Empty ics");
        }

    }
    read_begin(e, entries);
}
