import datetime
import json
import sys


def merge(left, right):
    return (min(left[0], right[0]),
            min(left[1], right[1]),
            max(left[2], right[2]),
            max(left[3], right[3]))


def bbox_list(lst, func):
    if len(lst) == 0:
        raise ValueError("None found")
    elif len(lst) == 1:
        return func(lst[0])
    else:
        mid = len(lst) // 2
        left = lst[:mid]
        right = lst[mid:]
        ans = merge(bbox_list(left, func), bbox_list(right, func))
        return ans


def bbox_point(p):
    return (p[0], p[1], p[0], p[1])


def bbox_multi(m):
    return bbox_list(m, bbox_point)


def bbox_multipoly(mp):
    return bbox_list(mp[0], bbox_point)


def bbox_feature(f):
    return bbox_geom(f['geometry'])


geom_map = {
    'Point': lambda coords, geoms: bbox_point(coords),
    'MultiPoint': lambda coords, geoms: bbox_list(coords, bbox_point),
    'LineString': lambda coords, geoms: bbox_list(coords, bbox_point),
    'MultiLineString': lambda coords, geoms: bbox_list(coords, bbox_multi),
    'Polygon': lambda coords, geoms: bbox_list(coords[0], bbox_point),
    'MultiPolygon': lambda coords, geoms: bbox_list(coords, bbox_multipoly),
    'GeometryCollection': lambda coords, geoms: bbox_list(geoms, bbox_geom),
}


def bbox_geom(g):
    func = geom_map.get(g['type'])
    if func is None:
        raise ValueError('Unrecognized geometry type: %s' % g['type'])

    coords = g.get('coordinates')
    geoms = g.get('geometries')
    return func(coords, geoms)


if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: $par_bbox /path/to/file.geojson")
        sys.exit(1)

    with open(sys.argv[1]) as f:
        start_time = datetime.datetime.now()
        data = json.load(f)
        end_parse_time = datetime.datetime.now()

    bbox = bbox_list(data['features'], bbox_feature) 
    end_bbox_time = datetime.datetime.now()

    print("Total bbox %s" % (bbox,))
    print("Time to parse: %s" % (end_parse_time - start_time).total_seconds())
    print("Time to bbox: %s" % (end_bbox_time - end_parse_time).total_seconds())
