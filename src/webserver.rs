extern crate iron;
extern crate router;
extern crate image;
extern crate hyper;
extern crate regex;

use std::vec::Vec;
use std::io::{Read, Write};
use std::fs;
use std::fs::File;
use std::path::Path;

use self::hyper::Client;
use self::iron::prelude::*;
use self::iron::status;
use self::regex::Regex;
use metalfilter::*;

pub fn launch_webserver(binding: &str) {
    println!("Listening on {}", binding);

    let router = router!(
        list_images: get "/" => list_images_handler,
        filter_index: get "/filter" => upload_index_handler,
        filter: get "/filter/*" => upload_handler,
    );
    Iron::new(router).http(binding).unwrap();
}

fn upload_index_handler(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::headers::ContentType::html().0, status::Ok, format!("\
        <link href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/css/bootstrap.min.css' rel='stylesheet' crossorigin='anonymous'>\
        <div class='container'>\
            <h1>Image Uploader</h1>\
            You want /filter/&lt;url of image goes here&gt;
        </div>\
    "))))
}

fn upload_handler(req: &mut Request) -> IronResult<Response>
{
    let image_url = get_linked_url_from_req_path(req.url.path());
    lazy_static! {
        static ref REGEX_NON_FILESAFE: Regex = Regex::new(r"([^a-zA-Z0-9_\-\.]+)").unwrap();
    }
    let url_safe_name = REGEX_NON_FILESAFE.replace_all(&image_url, "_").into_owned();
    if !fetch_image_if_not_cached(&("images/".to_string() + &url_safe_name), &image_url) {
        return Ok(Response::with((status::BadRequest, "Image not found")));
    }

    let path = req.url.path();
    let mut path_chunks2 = path.iter();
    path_chunks2.find(|&&x| match x {
        "render" => true,
        _ => false
    });
    let filter_name_option = path_chunks2.next();
    let output_format_option = path_chunks2.next();
    let filter_arg_option = path_chunks2.next();


    match filter_name_option {
        Some(filter_name) => {
            let output_format = match output_format_option {
                Some(x) => *x,
                None => "jpg",
            };
            let filter_arg = filter_arg_option.unwrap_or(&"1").parse().unwrap_or(1f32);
            image_render(&url_safe_name, filter_name, output_format, filter_arg)
        },
        None => {
            list_image_options(&url_safe_name)
        }
    }
}

fn get_linked_url_from_req_path(path: Vec<&str>) -> String {
    let mut path_chunks = path.iter();
    let mut image_url = String::new();

    path_chunks.next(); // skip the /filter part
    for x in path_chunks {
        match *x {
            "render" => break,
            _ => {
                if image_url.len() > 0 && x.len() > 0 { image_url += "/"; }
                image_url += x;
            }
        }
    }
    return image_url;
}

fn fetch_image_if_not_cached(image_path_string: &str, url: &str) -> bool
{
    let url_safe_path = Path::new(image_path_string);

    if url_safe_path.exists() {
        return true;
    }
    println!("Image '{}' not found, looking for it...", url);

    let client = Client::new();
    match client.get(&url.replace("https", "http")).send() {
        Ok(mut response) => {
            let ref mut fout = File::create( &url_safe_path).unwrap();
            let mut read = Vec::new();
            response.read_to_end(& mut read).unwrap();
            fout.write_all(read.as_slice()).unwrap();
            return true;
        }
        Err(err) => {
            println!("Image download failed: {}", err);
            return false
        }
    }
}

fn list_images_handler(_: &mut Request) -> IronResult<Response> {

    let paths = fs::read_dir("images/").unwrap();

    let mut html = String::new();
    for path in paths {
        match path.unwrap().file_name().to_str() {
            Some(filename) => {
                html += &format!("\
                    <div class='img-link'>\
                        <a href='/filter/{0}'>\
                            <img src='/filter/{0}/render/vanilla/jpg'>\
                        </a>\
                    </div>\
                ", filename);
            }
            None => {}
        }

    }

    Ok(Response::with((iron::headers::ContentType::html().0, status::Ok, format!("\
        <link href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/css/bootstrap.min.css' rel='stylesheet' crossorigin='anonymous'>\
        <style>
            .img-link {{
                display: block;
                margin-bottom: 10px;
            }}
            img {{
                max-width: 250px;
            }}
        </style>
        <div class='container'>\
            <h1>Images:</h1>\
            {}
        </div>\
    ", html))))
}

fn list_image_options(source_name: &str) -> IronResult<Response> {
    Ok(Response::with((iron::headers::ContentType::html().0, status::Ok, format!("\
        <link href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/css/bootstrap.min.css' rel='stylesheet' crossorigin='anonymous'>\
        <script src='https://code.jquery.com/jquery-3.2.1.min.js' integrity='sha256-hwg4gsxgFZhOsEEamdOYGBf13FyQuiTwlAQgxVSNgt4=' crossorigin='anonymous'></script>
        <script src='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/js/bootstrap.min.js' integrity='sha384-Tc5IQib027qvyjSMfHjOMaLkfuWVxZxUPnCJA7l2mCWNIpG9mGCD8wGNIcPD7Txa' crossorigin='anonymous'></script>
        <style>
            .image-filter-container {{
                text-align: center;
            }}
            .image-filter-container img {{
                max-width: 100%;
            }}
        </style>
        <div class='container'>\
            <h1>Image Filters:</h1>\
            <ul class='nav nav-tabs' role='tablist'>
                <li role='presentation' class='active'><a href='#vanilla' aria-controls='vanilla' role='tab' data-toggle='tab'>Vanilla</a></li>
                <li role='presentation'><a href='#red_average' aria-controls='red_average' role='tab' data-toggle='tab'>Red (averaging b/g)</a></li>
                <li role='presentation'><a href='#red_low' aria-controls='red_low' role='tab' data-toggle='tab'>Red Weighted Low</a></li>
                <li role='presentation'><a href='#red_mid' aria-controls='red_mid' role='tab' data-toggle='tab'>Red Weighted Medium</a></li>
                <li role='presentation'><a href='#red_high' aria-controls='red_high' role='tab' data-toggle='tab'>Red Weighted High</a></li>
                <li role='presentation'><a href='#red_custom' aria-controls='red_custom' role='tab' data-toggle='tab'>Red Weighted Custom</a></li>
            </ul>
            <div class='tab-content image-filter-container'>
                <div role='tabpanel' class='tab-pane active' id='vanilla'>
                    <img src='/filter/{0}/render/vanilla/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_average'>
                    <img src='/filter/{0}/render/red_average/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_low'>
                    <img src='/filter/{0}/render/red_low/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_mid'>
                    <img src='/filter/{0}/render/red_mid/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_high'>
                    <img src='/filter/{0}/render/red_high/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_custom'>
                    <img src='/filter/{0}/render/red_custom/jpg/1.5'><br>
                    <input id='red_custom_amount' type='range' min='0.25' max='10' step='0.125'/>
                </div>
            </div>
        </div>
        <script>
            $('#red_custom_amount').on('input', function() {{
                $('#red_custom img').prop('src', '/filter/{0}/render/red_custom/jpg/' + $(this).val());
            }});
        </script>\
    ", source_name))))
}

fn image_render(source_name: &str, filter_name: &str, output_format: &str, filter_option: f32) -> IronResult<Response> {
    let filter = match filter_name {
        "vanilla" => PixelFilters::None,
        "inverted" => PixelFilters::Inverted,
        "red_average" => PixelFilters::RedAverage,
        "red_high" => PixelFilters::RedWeightedHigh,
        "red_mid" => PixelFilters::RedWeightedMid,
        "red_low" => PixelFilters::RedWeightedLow,
        "red_custom" => PixelFilters::RedWeightedCustom,
        _ => PixelFilters::RedWeightedMid,
    };

    let img = match apply_filter(&("images/".to_owned() + source_name), filter, filter_option) {
        Ok(img) => img,
        Err(err) => return Err(IronError::new(err, status::BadRequest))
    };

    let mut payload = Vec::new();
    match output_format {
        "png" => match img.save(&mut payload, image::PNG) {
            Ok(_) => Ok(Response::with((iron::headers::ContentType::png().0, status::Ok, payload))),
            Err(_) => Ok(Response::with((status::InternalServerError, "img failed to encode")))
        },
        "jpg" => match img.save(&mut payload, image::JPEG) {
            Ok(_) => Ok(Response::with((iron::headers::ContentType::jpeg().0, status::Ok, payload))),
            Err(_) => Ok(Response::with((status::InternalServerError, "img failed to encode")))
        },
        _ => Ok(Response::with((status::BadRequest, "URL must end in supported image type")))
    }

}
