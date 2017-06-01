#[macro_use]
extern crate router;

#[macro_use]
extern crate lazy_static;

use std::env;

mod pixel_filters;

extern crate metalfilter;
use metalfilter::*;

pub mod webserver;

const DEFAULT_BINDING : &str = "localhost:3000";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&("-w".to_owned())) {
        {
            let mut args_iter = args.iter();
            args_iter.position(|&ref r| r == "-w"); // find the position of the -w
            let default_binding = DEFAULT_BINDING.to_string();
            let binding = args_iter.next().unwrap_or(&default_binding); // and then get the argument immediately after it
            webserver::launch_webserver(binding);
        }
    } else {
        let mut input_name = "test.jpg";
        if args.len() > 1 {
            input_name = args[1].as_str();
        }
        match apply_filter(input_name, PixelFilters::RedWeightedMid, 1f32) {
            Ok(img) => save_to_jpg_file(img, "out.jpg"),
            Err(err) => println!("Writing failed {}", err),
        }
    }
}
