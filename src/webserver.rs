extern crate iron;
extern crate router;
extern crate image;

use std::vec::Vec;
use self::iron::prelude::*;
use self::iron::status;
use self::router::Router;
use metalfilter::*;

pub fn launch_webserver(binding: &str) {
    println!("Listening on {}", binding);

    let router = router!(
        list_images: get "/render" => list_images_handler,
        list_image_options: get "/render/:image_name" => list_image_options_handler,
        render_image: get "/render/:image_name/:filter_name/:output_format" => image_render_handler,
        render_image_opt: get "/render/:image_name/:filter_name/:output_format/:filter_option" => image_render_handler,
    );
    Iron::new(router).http(binding).unwrap();
}

fn list_images_handler(_: &mut Request) -> IronResult<Response> {

    Ok(Response::with((iron::headers::ContentType::html().0, status::Ok, format!("\
        <link href='https://maxcdn.bootstrapcdn.com/bootstrap/3.3.7/css/bootstrap.min.css' rel='stylesheet' crossorigin='anonymous'>\
        <style>
            img {{
                max-width: 250px;
            }}
        </style>
        <div class='container'>\
            <h1>Sample Images:</h1>\
            <a href='/render/red-car.jpg'><img src='/render/red-car.jpg/vanilla/jpg'></a><br><br>
            <a href='/render/red-boat.jpg'><img src='/render/red-boat.jpg/vanilla/jpg'></a><br><br>
            <a href='/render/park-bench.jpg'><img src='/render/park-bench.jpg/vanilla/jpg'></a><br><br>
        </div>\
    "))))
}

fn list_image_options_handler(req: &mut Request) -> IronResult<Response> {
    let source_name: &str = req.extensions.get::<Router>().unwrap().find("image_name").unwrap();

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
                    <img src='/render/{0}/vanilla/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_average'>
                    <img src='/render/{0}/red_average/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_low'>
                    <img src='/render/{0}/red_low/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_mid'>
                    <img src='/render/{0}/red_mid/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_high'>
                    <img src='/render/{0}/red_high/jpg'>
                </div>
                <div role='tabpanel' class='tab-pane' id='red_custom'>
                    <img src='/render/{0}/red_custom/jpg/1.5'><br>
                    <input id='red_custom_amount' type='range' min='0.25' max='10' step='0.125'/>
                </div>
            </div>
        </div>
        <script>
            $('#red_custom_amount').on('input', function() {{
                $('#red_custom img').prop('src', '/render/{0}/red_custom/jpg/' + $(this).val());
            }});
        </script>\
    ", source_name))))
}

fn image_render_handler(req: &mut Request) -> IronResult<Response> {
    let source_name: &str = req.extensions.get::<Router>().unwrap().find("image_name").unwrap();
    let filter_name = req.extensions.get::<Router>().unwrap().find("filter_name").unwrap_or("default");
    let output_format = req.extensions.get::<Router>().unwrap().find("output_format").unwrap_or("jpg");
    let filter_option : f32 = req.extensions.get::<Router>().unwrap().find("filter_option").unwrap_or("1").parse().unwrap_or(1f32);

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

    let img = apply_filter(&("sample_images/".to_owned() + source_name), filter, filter_option);

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
