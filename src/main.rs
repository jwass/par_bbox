extern crate geojson;
extern crate rayon;
extern crate time;

use std::error::Error;
use std::env;
use std::fs::File;
use std::io::{Read};

use geojson::{GeoJson, Feature, FeatureCollection, Geometry, Position, Value};
use time::PreciseTime;


#[derive(Debug)]
struct Bbox {
    xmin: f64,
    xmax: f64,
    ymin: f64,
    ymax: f64,
}


impl Bbox {
    // Ignore antimeridian crossings for now
    pub fn merge(&self, other: &Bbox) -> Self {
        Bbox {
            xmin: self.xmin.min(other.xmin),
            xmax: self.xmax.max(other.xmax),
            ymin: self.ymin.min(other.ymin),
            ymax: self.ymax.max(other.ymax),
        }
    }
}


trait ToBbox {
    fn to_bbox(&self) -> Bbox;
}


impl ToBbox for Position {
    // A GeoJson::Position is a (longitude, latitude) tuple. The min/max of
    // the bounding box are the longitude, latitude of the Position.
    fn to_bbox(&self) -> Bbox {
        Bbox { xmin: self[0], ymin: self[1], xmax: self[0], ymax: self[1] }
    }
}


impl ToBbox for Geometry {
    fn to_bbox(&self) -> Bbox { self.value.to_bbox() }
}


impl ToBbox for Feature {
    // A Feature's bounding box is the bounding box of its geometry. We assume
    // features will have a geometry, even though it is technically optional.
    fn to_bbox(&self) -> Bbox { self.geometry.as_ref().unwrap().to_bbox() }
}


impl ToBbox for FeatureCollection {
    // Recursively split up the feature collection's bounding box into the
    // bounding box of the individual features.
    fn to_bbox(&self) -> Bbox {
        compute_bbox(&self.features, &|ref f| f.to_bbox())
    }
}


impl ToBbox for GeoJson {
    fn to_bbox(&self) -> Bbox {
        match *self {
            GeoJson::Geometry(ref geometry) => geometry.to_bbox(),
            GeoJson::Feature(ref feature) => feature.to_bbox(),
            GeoJson::FeatureCollection(ref fc) => fc.to_bbox(),
        }
    }
}


// This is a helper function that we use a bunch below in the bounding box
// calculation of each geometry type.
fn position_bbox(p: &Position) -> Bbox { p.to_bbox() }


impl ToBbox for Value {
    fn to_bbox(&self) -> Bbox {
        match *self {
            // Point is GeoJson::Position or Vec<f64> which is
            // a [longitude,latitude] pair
            Value::Point(ref p) => p.to_bbox(),

            // MultiPoint is Vec<Position>
            // Break up the MultiPoint into smaller MultiPoints until we get
            // to a single Position value, then use position_bbox to return
            // the single position's value and combine back up the chain.
            Value::MultiPoint(ref vp) => compute_bbox(vp, &position_bbox),

            // LineString is Vec<Position>
            Value::LineString(ref vp) => compute_bbox(vp, &position_bbox),

            // MultiLineString is Vec<Vec<Position>>
            Value::MultiLineString(ref vvp) => compute_bbox(vvp, &|ref vp| compute_bbox(vp, &position_bbox)),

            // Polygon is Vec<Vec<Position>>. The first element is the outer
            // ring / exterior of the polygon which we use to compute the
            // bounding box of the total polygon.  Extract the first element
            // (which is like a LineString) and return its bounding box.
            Value::Polygon(ref vvp) => compute_bbox(&vvp[0], &position_bbox),

            // MultiPolygon is Vec<Vec<Vec<Position>>>, a Vec of polygon
            // coordinates. When we get to an individual polygon, just use its
            // outer ring like the Polygon code above.
            Value::MultiPolygon(ref vvvp) => compute_bbox(vvvp, &|ref vvp| compute_bbox(&vvp[0], &position_bbox)),

            // GeometryCollection is Vec<Geometry>.
            Value::GeometryCollection(ref geoms) => compute_bbox(geoms, &|ref g| g.to_bbox()),
        }
    }
}


// Divide and conquer approach for computing bounding boxes.  This relies on
// the fact that the bounding box of an array of objects is the merged
// bounding box of the first half of the array with the bounding box of the
// second half of the array. We recursively split up the array until we
// compute the bounding box of a single element, and the combining the
// bounding boxes to compute the overall bounding box. Computing the bounding
// box of the individual elements are broken down the same way until we reach
// a single coordinate (Position) pair.  The final process may have varying
// levels of nesting depending on the structure of the data.  `func` is
// supplied to compute the bounding box of a single value. We use different
// behavior for the same type (such as Vec<Vec<Position>>) depending on the
// geometry type (i.e., Polygon vs.  MultiLineString).
fn compute_bbox<T, F>(v: &[T], func: &F) -> Bbox 
    where F: Fn(&T) -> Bbox + Sync, T: Sync {
    match v.len() {
        0 => panic!("No positions!"),
        1 => func(&v[0]),
        _ => {
            let mid = v.len() / 2;
            let (left, right) = v.split_at(mid);
            let (left_bbox, right_bbox) = rayon::join(|| compute_bbox(left,
func), || compute_bbox(right, func));
            left_bbox.merge(&right_bbox)
        }
    }
}


// Open the file specified on the command line.
// Bail if we're not called correctly or can't open the file.
fn get_file_or_fail() -> File {
    let args : Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: $par_bbox /path/to/file.geojson");
        std::process::exit(1);
    }

    let filename = &args[1];
    match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            println!("Could not open '{}': {}", filename, e.description());
            std::process::exit(1);
        }
    }
}


fn main() {
    let mut file = get_file_or_fail();

    // Load the file into a String, then parse. This is faster than
    // parsing directly from the File.
    let mut data = String::new();

    let start = PreciseTime::now();
    println!("Reading file");
    file.read_to_string(&mut data).unwrap();
    println!("Parsing JSON");
    let geojson : GeoJson = data.parse().unwrap();
    let end_parsed = PreciseTime::now();
    println!("Parsed.");

    let total_bbox = geojson.to_bbox();
    let end_bbox = PreciseTime::now();
 
    println!("Total bbox: {:?}", total_bbox);
    println!("Time to parse: {}", start.to(end_parsed).num_microseconds().unwrap() as f64 * 1e-6);
    println!("Time to bbox: {:?}", end_parsed.to(end_bbox).num_microseconds().unwrap() as f64 * 1e-6)
}
