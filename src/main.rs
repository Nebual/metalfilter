#[macro_use]
extern crate router;

use std::env;

mod pixel_filters;

extern crate metalfilter;
use metalfilter::*;

#[cfg(feature = "webserver")]
pub mod webserver;

fn main() {
    let args: Vec<String> = env::args().collect();

    if cfg!(feature = "webserver") && args.contains(&("-w".to_owned())) {
        #[cfg(feature = "webserver")]
        {
            webserver::launch_webserver("localhost:3000");
        }
    } else {
        let mut input_name = "test.jpg";
        if args.len() > 1 {
            input_name = args[1].as_str();
        }
        let img = apply_filter(input_name, PixelFilters::RedWeightedMid, 1f32);
        save_to_jpg_file(img, "out.jpg");
    }
}
