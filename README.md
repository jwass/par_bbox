par_bbox
====

Compute the total bounding box of all features in a GeoJSON file in parallel
with a divide-and-conquer approach using
[rayon](https://github.com/rayon-rs/rayon).

This works by recursively splitting arrays of GeoJSON objects, computing the
bounding box of each half of the array and then combining the bounding boxes.
All objects are recursively broken down until we reach just a single
coordinate point.

For example, to compute the bounding box of a FeatureCollection, we combine
the bounding boxes of the two halves of its Features array. The bounding box
of each Feature geometry is similarly split. Different geometry types have different levels of nesting.
For example, a MultiLineString
will be split until we have a single LineString. That LineString will then be
broken down until we compute the bounding box of just a single Position
([longitude, latitude]). The bounding boxes are then combined up the call
stack to return a single bounding box.

Any contributions, suggestions or advice are more than welcome.

Usage
-----
```
$ par_bbox ./data/polys.geojson
Total bbox: Bbox { xmin: -71.1906871, xmax: -71.1894741, ymin: 42.228073,
ymax: 42.2285172 }
Time to parse: 0.000339
Time to bbox: 0.000002
```


Disclaimer
----------
This is just a toy experiment for me to learn and play with Rust and Rayon. It
likely has little practical value.


See Also
--------
[Rust Geo](https://crates.io/crates/geo) - which contains a more
straightforward bounding box algorithm.
