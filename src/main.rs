extern crate geojson;
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
    fn to_bbox(&self) -> Bbox { self.geometry.as_ref().unwrap().to_bbox() }
}


impl ToBbox for FeatureCollection {
    // Because we impl ToBbox for Feature, this takes advantage of
    // `impl<T: ToBbox> ToBbox for [T]`
    fn to_bbox(&self) -> Bbox { self.features.to_bbox() }
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


impl ToBbox for Value {
    fn to_bbox(&self) -> Bbox {
        match *self {
            // Point is GeoJson::Position
            // `impl ToBbox for Position { fn to_bbox(...) ... }`
            Value::Point(ref p) => p.to_bbox(),

            // MultiPoint is Vec<Position>
            // `impl<T: ToBbox> ToBbox for [T]`
            Value::MultiPoint(ref vp) => vp.to_bbox(),

            // LineString is Vec<Position>
            // `impl<T: ToBbox> ToBbox for [T]`
            Value::LineString(ref vp) => vp.to_bbox(),

            // MultiLineString is Vec<Vec<Position>>
            // `impl ToBbox for [Vec<Position>]`
            Value::MultiLineString(ref vvp) => vvp.to_bbox(),

            // Polygon is Vec<Vec<Position>>. The first element is the outer
            // ring / exterior of the polygon which we use to compute the
            // bounding box of the total polygon.  Extract the first element
            // (which looks like a LineString) and return its bounding box.
            // `impl<T: ToBbox> ToBbox for [T]`
            Value::Polygon(ref vvp) => vvp[0].to_bbox(),

            // MultiPolygon is Vec<Vec<Vec<Position>>>, a Vec of
            // Polygon objects. multipolygon_bbox recursively splits
            // them up pulling out the exterior of each polygon and
            // computing the merged final bbox.
            Value::MultiPolygon(ref vvvp) => multipolygon_bbox(vvvp),

            // GeometryCollection is Vec<Geometry>.
            // impl<T: ToBbox> ToBbox for [T]
            Value::GeometryCollection(ref geoms) => geoms.to_bbox(),
        }
    }
}


fn multipolygon_bbox(mp: &[Vec<Vec<Position>>]) -> Bbox {
    match mp.len() {
        0 => panic!("No positions!"),
    
        // When there's only one MultiPolygon, extract its outer ring and
        // return its bounding box
        1 => mp[0][0].to_bbox(),
        _ => {
            let midpoint = mp.len() / 2;
            let (left, right) = mp.split_at(midpoint);
            multipolygon_bbox(left).merge(&multipolygon_bbox(right))
        }
    }
}

impl<T: ToBbox> ToBbox for [T] {
    fn to_bbox(&self) -> Bbox { 
        match self.len() {
            0 => panic!("No positions!"),
            1 => self[0].to_bbox(),
            _ => {
                let midpoint = self.len() / 2;
                let (left, right) = self.split_at(midpoint);
                left.to_bbox().merge(&right.to_bbox())
            }
        }
    }
}


impl ToBbox for [Vec<Position>] {
    fn to_bbox(&self) -> Bbox { 
        match self.len() {
            0 => panic!("No positions!"),
            1 => self[0].to_bbox(),
            _ => {
                let midpoint = self.len() / 2;
                let (left, right) = self.split_at(midpoint);
                left.to_bbox().merge(&right.to_bbox())
            }
        }
    }
}


// Open the file specified on the command line.
// Bail if we're not called correctly or can't open the file.
fn get_file_or_fail() -> File {
    let mut args : Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: $par_bbox /path/to/file.geojson");
        std::process::exit(1);
    }

    let filename = args.remove(1);
    match File::open(filename.clone()) {
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
    file.read_to_string(&mut data).unwrap();
    let geojson : GeoJson = data.parse().unwrap();
    let end_parsed = PreciseTime::now();

    let total_bbox = geojson.to_bbox();
    let end_bbox = PreciseTime::now();

    //println!("Number of points {}", n_point(&geojson));
    println!("Total bbox: {:?}", total_bbox);
    println!("Time to parse: {}", start.to(end_parsed).num_microseconds().unwrap() as f64 * 1e-6);
    println!("Time to bbox: {:?}", end_parsed.to(end_bbox).num_microseconds().unwrap() as f64 * 1e-6)
}
