use crate::coordinates::{Coordinate, Line, Polygon};
use geo::{CoordsIter, LineString, Point, Polygon as GeoPolygon};
use geojson::Value;

pub trait ToGeo {
    type Output;
    fn to_geo(&self) -> Self::Output;
}

pub trait FromGeo<T> {
    fn from_geo(value: &T) -> Self;
}

pub trait ToGeoJson {
    fn to_geojson(&self) -> Value;
}

impl ToGeo for Coordinate {
    type Output = Point<f64>;
    fn to_geo(&self) -> Point<f64> {
        Point::new(self.x, self.y)
    }
}

impl FromGeo<Point<f64>> for Coordinate {
    fn from_geo(point: &Point<f64>) -> Self {
        Self {
            x: point.x(),
            y: point.y(),
        }
    }
}

impl ToGeo for Line {
    type Output = LineString<f64>;
    fn to_geo(&self) -> LineString<f64> {
        LineString::from(
            self.coordinates
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        )
    }
}

impl FromGeo<LineString<f64>> for Line {
    fn from_geo(ls: &LineString<f64>) -> Self {
        Self {
            coordinates: ls
                .coords_iter()
                .map(|c| Coordinate::new(c.x, c.y))
                .collect(),
        }
    }
}

impl ToGeo for Polygon {
    type Output = GeoPolygon<f64>;
    fn to_geo(&self) -> GeoPolygon<f64> {
        GeoPolygon::new(
            self.exterior.to_geo(),
            self.interiors.iter().map(|l| l.to_geo()).collect(),
        )
    }
}

impl ToGeoJson for Coordinate {
    fn to_geojson(&self) -> Value {
        Value::Point(vec![self.x, self.y])
    }
}

impl ToGeoJson for Line {
    fn to_geojson(&self) -> Value {
        Value::LineString(self.to_vecs())
    }
}

impl ToGeoJson for Polygon {
    fn to_geojson(&self) -> Value {
        let mut rings = vec![self.exterior.to_vecs()];
        rings.extend(self.interiors.iter().map(|l| l.to_vecs()));
        Value::Polygon(rings)
    }
}
