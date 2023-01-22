#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use enterpolation::bspline::{BSpline, BSplineBuilder, BSplineError};
use enterpolation::bezier::{Bezier, BezierBuilder, BezierError};
use enterpolation::linear::{Linear, LinearError};
use enterpolation::{Curve, Generator, DiscreteGenerator};
// use assert_float_eq::{afe_is_f64_near, afe_near_error_msg, assert_f64_near};
use plotlib::{page::Page, repr::Plot, view::ContinuousView, style::{LineStyle, LineJoin}};

#[derive(Debug)]
struct Elements {
    data: Vec<f64>,
}

impl Generator<usize> for Elements {
    type Output = f64;
    fn gen(&self, idx: usize) -> f64 {
        self.data[idx]
    }
}

impl DiscreteGenerator for Elements {
    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Elements {
    fn new(data: Vec<f64>) -> Self {
        Elements { data }
    }
}

// fn main() -> Result<(), BSplineError> {
fn main() -> Result<(), BezierError> {
// fn main() {
    // let points: Vec<(f64, f64)> = vec![
    let points: Vec<(f64, f64)> = vec![
        (0., 0.),
        (5., 4.),
        (7., 12.),
        (15., 10.),
        (20., 13.),
        (27., 3.),
        (32., 7.),
        (38., 15.),
        (45., 17.),
        (48., 14.),
        (54., 21.),
        (55., 24.),
        (61., 19.),
        (72., 7.),
        (79., 10.),
        (84., 6.),
        (89., 6.),
        (96., 2.),
        (100., 0.)
    ];
    // let fake_points: Vec<f64> = vec![0., 4., 12., 10., 13., 3., 7., 15., 17., 14., 21., 24., 19., 7., 10., 6., 6., 2., 0. ];
    // let fake_points: Vec<f32> = vec![0., 4., 12., 10., 13.];
    // let fake_points = [0., 4., 12., 10., 13.];
    // let (knts, elts): (Vec<f64>, Vec<f64>) = points.iter().map(|x| *x).unzip();
    let (knts, elts): (Vec<f64>, Vec<f64>) = points.iter().unzip();
    // let weights: Vec<f64> = vec![0.7; 19];
    // let welts: Vec<(f64, f64)> = elts.iter().zip(weights).map(|(e, w)| (*e, w)).collect();
    // /*
    let lin = Linear::builder()
        .elements(elts)
        .knots(knts)
        .build().unwrap();
    let points2_: Vec<f64> = lin.take(101).collect();
    let points2 = Elements::new(points2_);
    // */
    // let points2: &[f64] = lin.take(101).collect::<Vec<f64>>().as_slice();
    /*
    let weights: Vec<f64> = vec![0.7; 101];
    let wpoints: Vec<(f64, f64)> = points2.iter().zip(weights).map(|(e, w)| (*e, w)).collect();
    */
    /*
    let points2 = Elements::new(elts);
    let bspline = BSpline::builder()
        .clamped()
        .elements(points2)
        // .elements_with_weights(wpoints)
        // .equidistant::<f64>()
        .knots(knts)
        // .degree(3)
        .dynamic()
        // .constant::<4>()
        // .constant()
        // .degree(3)
        // .domain(0.0, 100.0)
        // .range(0.0, 25.0)
        // .constant::<4>()
        .build().unwrap();
    */
    // /*
    let bez = Bezier::builder()
        .elements(points2)
        .domain(0.0, 101.0)
        // .normalized::<f64>()
        // .constant()
        .dynamic()
        .build()?;
    // */

    let xs: Vec<f64> = (0i32..=100i32).collect::<Vec<i32>>().iter().map(|x| x as f64).collect();
    // let ys: Vec<f64> = bez.take(101).collect();
    let ys: Vec<f64> = bspline.take(101).collect();
    let results: Vec<(f64, f64)> = xs.iter().zip(ys).map(|(x, y)| (x, y)).collect();
    let p1: Plot = Plot::new(points).line_style(
        LineStyle::new().colour("#DD3333")
    );
    let p2: Plot = Plot::new(results).line_style(
        LineStyle::new().colour("#3333DD")
    );
    let v = ContinuousView::new()
        .add(p1)
        .add(p2)
        .x_range(0., 101.)
        .y_range(0., 25.)
        .x_label("Distance")
        .y_label("Height");

    Page::single(&v).save("heightslice.svg").unwrap();

    Ok(())
}