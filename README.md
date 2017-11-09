bbox
====

Compute the bounding box of a GeoJSON file.

This is an experiment to play with Rust.


Usage
-----

Example:

```
$ cargo run ./data/polys.geojson
Total bbox: Bbox { xmin: -71.1906871, xmax: -71.1894741, ymin: 42.228073,
ymax: 42.2285172 }
Time to parse: 0.000339
Time to bbox: 0.000002
```
