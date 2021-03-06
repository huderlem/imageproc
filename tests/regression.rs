//! Compares results of image processing functions to existing "truth" images.
//! All test images are taken from the caltech256 dataset.
//! http://authors.library.caltech.edu/7694/

#![feature(test)]
#![feature(unboxed_closures)]
#![feature(fn_traits)]

extern crate image;
extern crate test;
#[macro_use]
extern crate imageproc;

use std::ops::Deref;
use std::path::Path;
use std::f32;
use image::{DynamicImage, GrayImage, ImageBuffer, Pixel, Luma, RgbImage};
use imageproc::utils::{load_image_or_panic};
use imageproc::affine::{affine, Interpolation, rotate_about_center};
use imageproc::edges::canny;
use imageproc::filter::gaussian_blur_f32;
use imageproc::definitions::{Clamp, HasBlack, HasWhite};
use imageproc::gradients;
use imageproc::math::{Affine2, Mat2, Vec2};

// If set to true then all calls to any compare_to_truth function will regenerate
// the truth image.
const REGENERATE: bool = false;

/// Save an image with the given name to the truth data directory "./tests/data/truth".
/// Used when manually (re)generating test data, e.g. when REGENERATE is set to true.
fn save_truth_image<P, Container>(image: &ImageBuffer<P, Container>, file_name: &str)
    where P: Pixel<Subpixel=u8> + 'static, Container: Deref<Target=[u8]>
{
    let _ = image.save(Path::new("./tests/data/truth").join(file_name)).unwrap();
}

/// Load an image with the given name from the input data directory "./tests/data/".
/// Panics if unable to find or load an image at this path.
fn load_input_image(file_name: &str) -> DynamicImage {
    load_image_or_panic(Path::new("./tests/data").join(file_name))
}

/// Load an image with the given name from the truth data directory "./tests/data/truth".
/// Panics if unable to find or load an image at this path.
fn load_truth_image(file_name: &str) -> DynamicImage {
    load_image_or_panic(Path::new("./tests/data/truth").join(file_name))
}

/// Load an input image, apply a function to it and check that the results match a 'truth' image.
fn compare_to_truth_rgb<F>(input_file_name: &str, truth_file_name: &str, op: F)
    where F: Fn(&RgbImage) -> RgbImage
{
    compare_to_truth_rgb_with_tolerance(input_file_name, truth_file_name, op, 0u8);
}

/// Load an input image, apply a function to it and check that the results
/// match a 'truth' image to within a given per-pixel tolerance.
fn compare_to_truth_rgb_with_tolerance<F>(input_file_name: &str, truth_file_name: &str, op: F, tol: u8)
    where F: Fn(&RgbImage) -> RgbImage
{
    let input = load_input_image(input_file_name).to_rgb();
    let actual = op.call((&input,));

    if REGENERATE {
        save_truth_image(&actual, truth_file_name);
    }
    else {
        let truth = load_truth_image(truth_file_name).to_rgb();
        assert_pixels_eq_within!(actual, truth, tol);
    }
}

/// Load an input image, apply a function to it and check that the results
/// match a 'truth' image.
fn compare_to_truth_grayscale<F>(input_file_name: &str, truth_file_name: &str, op: F)
    where F: Fn(&GrayImage) -> GrayImage
{
    let input = load_input_image(input_file_name).to_luma();
    let actual = op.call((&input,));

    if REGENERATE {
        save_truth_image(&actual, truth_file_name);
    }
    else {
        let truth = load_truth_image(truth_file_name).to_luma();
        assert_pixels_eq!(actual, truth);
    }
}

#[test]
fn test_rotate_nearest_rgb() {
    fn rotate_nearest_about_center(image: &RgbImage) -> RgbImage {
        rotate_about_center(image, std::f32::consts::PI/4f32, Interpolation::Nearest)
    }
    compare_to_truth_rgb("elephant.png", "elephant_rotate_nearest.png", rotate_nearest_about_center);
}

#[test]
fn test_equalize_histogram_grayscale() {
    use imageproc::contrast::equalize_histogram;
    compare_to_truth_grayscale("lumaphant.png", "lumaphant_eq.png", equalize_histogram);
}

#[test]
fn test_rotate_bilinear_rgb() {
    fn rotate_bilinear_about_center(image: &RgbImage) -> RgbImage {
        rotate_about_center(image, std::f32::consts::PI/4f32, Interpolation::Bilinear)
    }
    compare_to_truth_rgb_with_tolerance("elephant.png", "elephant_rotate_bilinear.png", rotate_bilinear_about_center, 1);
}

#[test]
fn test_affine_nearest_rgb() {
    fn affine_nearest(image: &RgbImage) -> RgbImage {
        let root_two_inv = 1f32/2f32.sqrt();
        let trans = Affine2::new(
            Mat2::new(root_two_inv, -root_two_inv,
                      root_two_inv, root_two_inv) * 2f32,
            Vec2::new(50f32, -70f32)
        );
        affine(image, trans, Interpolation::Nearest).unwrap()
    }
    compare_to_truth_rgb("elephant.png", "elephant_affine_nearest.png", affine_nearest);
}

#[test]
fn test_affine_bilinear_rgb() {
    fn affine_bilinear(image: &RgbImage) -> RgbImage {
        let root_two_inv = 1f32/2f32.sqrt();
        let trans = Affine2::new(
            Mat2::new(root_two_inv, -root_two_inv,
                      root_two_inv, root_two_inv) * 2f32,
            Vec2::new(50f32, -70f32)
        );
        affine(image, trans, Interpolation::Bilinear).unwrap()
    }
    compare_to_truth_rgb("elephant.png", "elephant_affine_bilinear.png", affine_bilinear);
}

#[test]
fn test_sobel_gradients() {
    fn sobel_gradients(image: &GrayImage) -> GrayImage {
        imageproc::map::map_subpixels(&gradients::sobel_gradients(image), |x| u8::clamp(x))
    }
    compare_to_truth_grayscale("elephant.png", "elephant_gradients.png", sobel_gradients);
}

#[test]
fn test_match_histograms() {
    fn match_to_zebra_histogram(image: &GrayImage) -> GrayImage {
        let zebra = load_input_image("zebra.png").to_luma();
        imageproc::contrast::match_histogram(image, &zebra)
    }
    compare_to_truth_grayscale("elephant.png", "elephant_matched.png", match_to_zebra_histogram);
}

#[test]
fn test_canny() {
    compare_to_truth_grayscale("zebra.png", "zebra_canny.png", |image| canny(image, 250.0, 300.0));
}

#[test]
fn test_gaussian_blur_stdev_3() {
    compare_to_truth_grayscale("zebra.png", "zebra_gaussian_3.png", |image| gaussian_blur_f32(image, 3f32));
}

#[test]
fn test_gaussian_blur_stdev_10() {
    compare_to_truth_grayscale("zebra.png", "zebra_gaussian_10.png", |image| gaussian_blur_f32(image, 10f32));
}

#[test]
fn test_adaptive_threshold() {
    use imageproc::contrast::adaptive_threshold;
    compare_to_truth_grayscale("zebra.png", "zebra_adaptive_threshold.png", |image| adaptive_threshold(image, 41));
}

#[test]
fn test_draw_antialiased_line_segment_rgb() {
    use image::{Rgb};
    use imageproc::drawing::draw_antialiased_line_segment_mut;
    use imageproc::pixelops::interpolate;

    let blue = Rgb([0, 0, 255]);
    let mut image = RgbImage::from_pixel(200, 200, blue);

    let white = Rgb([255, 255, 255]);
    // Connected path:
    //      - horizontal
    draw_antialiased_line_segment_mut(&mut image, (20, 80), (40, 80), white, interpolate);
    //      - shallow ascent
    draw_antialiased_line_segment_mut(&mut image, (40, 80), (60, 70), white, interpolate);
    //      - diagonal ascent
    draw_antialiased_line_segment_mut(&mut image, (60, 70), (70, 70), white, interpolate);
    //      - steep ascent
    draw_antialiased_line_segment_mut(&mut image, (70, 70), (80, 30), white, interpolate);
    //      - shallow descent
    draw_antialiased_line_segment_mut(&mut image, (80, 30), (110, 45), white, interpolate);
    //      - diagonal descent
    draw_antialiased_line_segment_mut(&mut image, (110, 45), (130, 65), white, interpolate);
    //      - steep descent
    draw_antialiased_line_segment_mut(&mut image, (130, 65), (150, 110), white, interpolate);
    //      - vertical
    draw_antialiased_line_segment_mut(&mut image, (150, 110), (150, 140), white, interpolate);

    // Isolated segment, partially outside of image bounds
    draw_antialiased_line_segment_mut(&mut image, (150, 150), (210, 130), white, interpolate);

    if REGENERATE {
        save_truth_image(&image, "antialiased_lines_rgb.png");
    }
    else {
        let truth = load_truth_image("antialiased_lines_rgb.png").to_rgb();
        assert_pixels_eq!(image, truth);
    }
}


#[test]
fn test_draw_convex_polygon() {
    use imageproc::drawing::draw_convex_polygon_mut;
    use imageproc::drawing::Point;

    let mut image = GrayImage::from_pixel(300, 300, Luma::black());
    let white = Luma::white();

    let triangle = vec![
        Point::new(35, 50),
        Point::new(145, 80),
        Point::new(5, 60)];
    draw_convex_polygon_mut(&mut image, &triangle, white);

    let partially_out_of_bounds_triangle = vec![
        Point::new(250, 50),
        Point::new(350, 100),
        Point::new(250, 90)];
    draw_convex_polygon_mut(&mut image, &partially_out_of_bounds_triangle, white);

    let quad = vec![
        Point::new(190, 250),
        Point::new(240, 210),
        Point::new(270, 200),
        Point::new(220, 280)];
    draw_convex_polygon_mut(&mut image, &quad, white);

    let hex: Vec<Point<i32>> = (0..6)
        .map(|i| i as f32 * f32::consts::PI / 3f32)
        .map(|theta| (theta.cos() * 50.0 + 75.0, theta.sin() * 50.0 + 225.0))
        .map(|(x, y)| Point::new(x as i32, y as i32))
        .collect();
    draw_convex_polygon_mut(&mut image, &hex, white);

    if REGENERATE {
        save_truth_image(&image, "polygon.png");
    }
    else {
        let truth = load_truth_image("polygon.png").to_luma();
        assert_pixels_eq!(image, truth);
    }
}
