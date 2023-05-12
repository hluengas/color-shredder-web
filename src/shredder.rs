use image::{GrayImage, ImageFormat, Luma, Rgb, RgbImage};
use palette::{convert::TryIntoColor, Hsv, Srgb};
use rand::random;
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::{fs, time::Instant};
use strict_yaml_rust::StrictYamlLoader;

struct Painting {
    image: RgbImage,
    boundry_region_image: GrayImage,
    boundry_region_list: Vec<Coordinate>,
    starting_locations: Vec<Coordinate>,
    canvas_constraints: Constraints,
    canvas_stats: Stats,
}
struct Constraints {
    x_size: u32,
    y_size: u32,
}
struct Stats {
    current_pixels_placed_count: u64,
}
struct Pixel {
    position: Coordinate,
    color: Rgb<u8>,
}
#[derive(Copy, Clone)]
struct Coordinate {
    x: u32,
    y: u32,
}

fn main() {
    // set output constraints, initialize canvas, get starting positions
    let mut working_canvas: Painting = initialize_canvas();

    // initial update of the output files
    write_output_files(&working_canvas);

    // create a timer to update at regular intervals
    let mut current_time = Instant::now();

    // run the simulation loop as long as there are available positions in the boundry region
    while working_canvas.boundry_region_list.len() > 0 {
        let temp_color: Srgb = Hsv::new(
            0.55f32 * 360f32,
            random::<f32>().clamp(0.5f32, 1.0f32),
            random::<f32>().clamp(0.0f32, 1.0f32),
        )
        .try_into_color()
        .unwrap();
        let temp_color_alt: Srgb = Hsv::new(
            0.59f32 * 360f32,
            random::<f32>().clamp(0.5f32, 1.0f32),
            random::<f32>().clamp(0.0f32, 1.0f32),
        )
        .try_into_color()
        .unwrap();

        // choose a random color
        let mut target_color: Rgb<u8> = Rgb([
            (temp_color.red * 255f32).floor() as u8,
            (temp_color.green * 255f32).floor() as u8,
            (temp_color.blue * 255f32).floor() as u8,
        ]);
        if random::<bool>() {
            target_color = Rgb([
                (temp_color_alt.red * 255f32).floor() as u8,
                (temp_color_alt.green * 255f32).floor() as u8,
                (temp_color_alt.blue * 255f32).floor() as u8,
            ]);
        }
        // determine best location
        let target_pixel = get_best_position_for_color(target_color, &mut working_canvas);

        // update the canvas
        place_pixel(&target_pixel, &mut working_canvas);

        // update output files after given interval
        if current_time.elapsed().as_secs_f32() > (1f32 / 2f32) {
            current_time = Instant::now();
            write_output_files(&working_canvas);
        }
    }

    // final update of the output files
    write_output_files(&working_canvas);
}

fn initialize_canvas() -> Painting {
    // read config file into a string
    let config_string =
        fs::read_to_string("./config/config.yaml").expect("[ERROR] unable to read config.yaml");

    // parse config string using yaml structure
    let config = &StrictYamlLoader::load_from_str(&config_string)
        .expect("[ERROR] unable to parse config.yaml")[0]["config"];

    // hold the output image dimensions
    let working_constraints: Constraints = Constraints {
        x_size: config["canvas"]["size"]["x"]
            .as_str()
            .expect("[ERROR] failed to parse config value as string")
            .parse::<u32>()
            .expect("[ERROR] failed to convert config str to int"),
        y_size: config["canvas"]["size"]["y"]
            .as_str()
            .expect("[ERROR] failed to parse config value as string")
            .parse::<u32>()
            .expect("[ERROR] failed to convert config str to int"),
    };

    // hold running stats
    let working_stats: Stats = Stats {
        current_pixels_placed_count: 0u64,
    };

    let mut starting_points = Vec::new();

    // CUSTOMIZED:
    for location in config["canvas"]["starting_locations"] {
        let _x = &location["x"]
        .as_str()
        .expect("[ERROR] failed to parse config value as string")
        .parse::<u32>()
        .expect("[ERROR] failed to convert config str to int");
        
        let _y = &location["y"]
        .as_str()
        .expect("[ERROR] failed to parse config value as string")
        .parse::<u32>()
        .expect("[ERROR] failed to convert config str to int");


        starting_points.push(Coordinate {
            x: _x,
            y: _y,
        });
    }

    // hold all info required for painting
    let mut working_canvas: Painting = Painting {
        image: RgbImage::new(working_constraints.x_size, working_constraints.y_size),
        boundry_region_image: GrayImage::new(
            working_constraints.x_size,
            working_constraints.y_size,
        ),
        starting_locations: starting_points,
        canvas_constraints: working_constraints,
        canvas_stats: working_stats,
        boundry_region_list: Vec::new(),
    };

    // loop over starting positions and place random colors at each
    for index in 0..working_canvas.starting_locations.len() {
        let temp_color: Srgb = Hsv::new(
            0.55f32 * 360f32,
            random::<f32>().clamp(0.5f32, 1.0f32),
            random::<f32>().clamp(0.0f32, 1.0f32),
        )
        .try_into_color()
        .unwrap();
        let temp_color_alt: Srgb = Hsv::new(
            0.59f32 * 360f32,
            random::<f32>().clamp(0.5f32, 1.0f32),
            random::<f32>().clamp(0.0f32, 1.0f32),
        )
        .try_into_color()
        .unwrap();

        // choose a random color
        let mut target_color: Rgb<u8> = Rgb([
            (temp_color.red * 255f32).floor() as u8,
            (temp_color.green * 255f32).floor() as u8,
            (temp_color.blue * 255f32).floor() as u8,
        ]);
        if random::<bool>() {
            target_color = Rgb([
                (temp_color_alt.red * 255f32).floor() as u8,
                (temp_color_alt.green * 255f32).floor() as u8,
                (temp_color_alt.blue * 255f32).floor() as u8,
            ]);
        }
        let target_pixel = Pixel {
            position: working_canvas.starting_locations[index].clone(),
            color: target_color,
        };
        place_pixel(&target_pixel, &mut working_canvas)
    }

    return working_canvas;
}

fn get_initial_locations(working_constraints: &Constraints) -> Vec<Coordinate> {
    // hold starting locations
    let mut starting_points = Vec::new();

    // CUSTOMIZED:
    for i in 0..working_constraints.y_size - 1 {
        if (i + 1) % 64 == 0 {
            starting_points.push(Coordinate { x: 256, y: i });
        }
    }
    // starting_points.push(Coordinate { x: 256, y: 511 });
    // starting_points.push(Coordinate { x: 1792, y: 0 });
    return starting_points;
}

// write the output PNG files
fn write_output_files(working_canvas: &Painting) {
    // write the RGB painting file
    working_canvas
        .image
        .save_with_format("./output/painting.png", ImageFormat::Png)
        .unwrap();

    // write the boundry region image
    working_canvas
        .boundry_region_image
        .save_with_format("./output/boundry.png", ImageFormat::Png)
        .unwrap();
}

// update a pixel on the canvas and add its neighbors to the boundry region
fn place_pixel(target: &Pixel, working_canvas: &mut Painting) {
    // update a pixel on the canvas
    working_canvas
        .image
        .put_pixel(target.position.x, target.position.y, target.color);

    // update counter
    working_canvas.canvas_stats.current_pixels_placed_count += 1;

    // loop over neighbors in a 3x3 grid around the target
    for i in 0..3 {
        for j in 0..3 {
            // skip self
            if i == 1 && j == 1 {
                continue;
            }
            // prevent less than zero out-of-bounds
            if target.position.x == 0 && i == 0 {
                continue;
            }
            if target.position.y == 0 && j == 0 {
                continue;
            }
            // prevent greater than dimensions out-of-bounds
            if target.position.x == (working_canvas.canvas_constraints.x_size - 1) && i == 2 {
                continue;
            }
            if target.position.y == (working_canvas.canvas_constraints.y_size - 1) && j == 2 {
                continue;
            }

            // calculate the neighbor's coordinate
            let neighbor_x_coord: u32 = target.position.x + i - 1;
            let neighbor_y_coord: u32 = target.position.y + j - 1;

            // get the neigbor's luma (boundry region value)
            let neighbor_luma: Luma<u8> = *working_canvas
                .boundry_region_image
                .get_pixel(neighbor_x_coord, neighbor_y_coord);

            // get the neighbor's color
            let neighbor_color: Rgb<u8> = *working_canvas
                .image
                .get_pixel(neighbor_x_coord, neighbor_y_coord);

            // ensure locations are not added to the boundry region as duplicates
            if neighbor_luma != Luma([0u8]) || neighbor_color != Rgb([0u8, 0u8, 0u8]) {
                continue;
            }

            // add this neighbor to the boundry region LIST
            working_canvas.boundry_region_list.push(Coordinate {
                x: neighbor_x_coord,
                y: neighbor_y_coord,
            });
            // add this neighbor to the boundry region IMAGE (luma)
            working_canvas.boundry_region_image.put_pixel(
                neighbor_x_coord,
                neighbor_y_coord,
                Luma([255u8]),
            );
        }
    }
}

fn get_best_position_for_color(target_color: Rgb<u8>, working_canvas: &mut Painting) -> Pixel {
    let (_best_value, best_position, best_position_index) = working_canvas
        .boundry_region_list
        .par_iter()
        .enumerate()
        .map(|available_location| {
            evaluate_position(
                available_location.1,
                available_location.0,
                &target_color,
                &working_canvas.image,
                &working_canvas.canvas_constraints,
            )
        })
        .reduce_with(|a, b| {
            if a.0 < b.0 {
                return a;
            } else if a.0 == b.0 && random::<bool>() {
                return a;
            } else {
                return b;
            }
        })
        .unwrap();

    // remove target pixel from boundrry region IMAGE
    working_canvas
        .boundry_region_image
        .put_pixel(best_position.x, best_position.y, Luma([0u8]));

    // if this is not the last location of the boundry region
    // swap remove the target pixel location from the boundry region LIST
    // (swap remove is much faster)
    if working_canvas.boundry_region_list.len() > 1 {
        return Pixel {
            color: target_color,
            position: working_canvas
                .boundry_region_list
                .swap_remove(best_position_index),
        };
    }
    // for the last elemet remove normally
    else {
        return Pixel {
            color: target_color,
            position: working_canvas
                .boundry_region_list
                .remove(best_position_index),
        };
    }
}

fn evaluate_position(
    target_location: &Coordinate,
    target_index: usize,
    target_color: &Rgb<u8>,
    canvas_image: &RgbImage,
    canvas_constraints: &Constraints,
) -> (f32, Coordinate, usize) {
    let mut cummulative_color_distance: f32 = 0f32;
    let mut neighbor_count: u64 = 0;
    let mut color_distance: f32;
    let _average_color_distance: f32;
    let mut min_color_distance: f32 = f32::MAX;

    // loop over neighbors in a 3x3 grid around the target
    for i in 0..3 {
        for j in 0..3 {
            // skip self
            if i == 1 && j == 1 {
                continue;
            }
            // prevent less than zero out-of-bounds
            if target_location.x == 0 && i == 0 {
                continue;
            }
            if target_location.y == 0 && j == 0 {
                continue;
            }
            // prevent greater than dimensions out-of-bounds
            if target_location.x == (canvas_constraints.x_size - 1) && i == 2 {
                continue;
            }
            if target_location.y == (canvas_constraints.y_size - 1) && j == 2 {
                continue;
            }

            // calculate neighbor coordinates
            let neighbor_x_coord: u32 = target_location.x + i - 1;
            let neighbor_y_coord: u32 = target_location.y + j - 1;

            // get color at neighbor's coordinates
            let neighbor_color: Rgb<u8> =
                *canvas_image.get_pixel(neighbor_x_coord, neighbor_y_coord);

            // skip un-colored
            if neighbor_color == Rgb([0u8, 0u8, 0u8]) {
                continue;
            }

            // increment neighbor count
            neighbor_count += 1;

            //compute color distance
            color_distance = 0f32;
            for i in 0..3 {
                color_distance +=
                    (target_color[i] as f32 - neighbor_color[i] as f32).powf(2f32) as f32;
            }
            cummulative_color_distance += color_distance;

            // update MIN
            if color_distance < min_color_distance {
                min_color_distance = color_distance;
            } else if color_distance == min_color_distance && random::<bool>() {
                min_color_distance = color_distance;
            }
        }
    }

    // update AVG
    _average_color_distance = cummulative_color_distance / neighbor_count as f32;
    return (min_color_distance, *target_location, target_index);
}
