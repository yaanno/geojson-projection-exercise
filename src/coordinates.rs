use std::fmt;
use std::iter::FromIterator;

/// A 2D coordinate with x and y values
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    /// Create a new coordinate
    ///
    /// # Arguments
    ///
    /// * `x` - The x coordinate
    /// * `y` - The y coordinate
    ///
    /// # Returns
    ///
    /// * `Coordinate` - A new coordinate
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    ///
    /// let coord = Coordinate::new(13.377, 52.518);
    /// ```
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Convert a vector of coordinates to a vector of points
    ///
    /// # Arguments
    ///
    /// * `coords` - A vector of coordinates
    ///
    /// # Returns
    ///
    /// * `Vec<geo::Point<f64>>` - A vector of points
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let points = Coordinate::to_points(&coords);
    /// ```
    pub fn to_points(coords: &[Coordinate]) -> Vec<geo::Point<f64>> {
        coords.iter().map(|c| geo::Point::new(c.x, c.y)).collect()
    }

    /// Convert a vector of coordinates to a vector of coordinate vectors
    ///
    /// # Arguments
    ///
    /// * `coords` - A vector of coordinates
    ///
    /// # Returns
    ///
    /// * `Vec<Vec<f64>>` - A vector of coordinate vectors
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let vecs = Coordinate::to_vecs(&coords);
    /// ```
    pub fn to_vecs(coords: &[Coordinate]) -> Vec<Vec<f64>> {
        coords.iter().map(|c| vec![c.x, c.y]).collect()
    }

    pub fn to_vec(&self) -> Vec<f64> {
        vec![self.x, self.y]
    }
}

impl From<geo::Point<f64>> for Coordinate {
    /// Convert a geo point to a coordinate
    ///
    /// # Arguments
    ///
    /// * `point` - A geo point
    ///
    /// # Returns
    ///
    /// * `Coordinate` - A coordinate
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    ///
    /// let point = geo::Point::new(13.377, 52.518);
    /// let coord = Coordinate::from(point);
    /// ```
    fn from(point: geo::Point<f64>) -> Self {
        Self {
            x: point.x(),
            y: point.y(),
        }
    }
}

impl From<Coordinate> for geo::Point<f64> {
    /// Convert a coordinate to a geo point
    ///
    /// # Arguments
    ///
    /// * `coord` - A coordinate
    ///
    /// # Returns
    ///
    /// * `geo::Point<f64>` - A geo point
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    ///
    /// let coord = Coordinate::new(13.377, 52.518);
    /// let point = geo::Point::from(coord);
    /// ```
    fn from(coord: Coordinate) -> Self {
        Self::new(coord.x, coord.y)
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

/// A collection of coordinates that form a line
#[derive(Debug, Clone)]
pub struct Line {
    pub coordinates: Vec<Coordinate>,
}

impl Line {
    /// Create a new line from a vector of coordinates
    ///
    /// # Arguments
    ///
    /// * `coordinates` - A vector of coordinates
    ///
    /// # Returns
    ///
    /// * `Line` - A new line
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// ```
    pub fn new(coordinates: Vec<Coordinate>) -> Self {
        Self { coordinates }
    }

    /// Convert a line to a GeoJSON line string
    ///
    /// # Returns
    ///
    /// * `geojson::Value` - A GeoJSON line string
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let geojson = line.to_geojson();
    /// ```
    pub fn to_geojson(&self) -> geojson::Value {
        geojson::Value::LineString(Coordinate::to_vecs(&self.coordinates))
    }

    /// Convert a line to a geo line string
    ///
    /// # Returns
    ///
    /// * `geo::LineString<f64>` - A geo line string
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let geo = line.to_geo();
    /// ```
    pub fn to_geo(&self) -> geo::LineString<f64> {
        geo::LineString::from(
            self.coordinates
                .iter()
                .map(|c| geo::Coord::from((c.x, c.y)))
                .collect::<Vec<_>>(),
        )
    }

    pub fn from_geo(ls: &geo::LineString<f64>) -> Self {
        Self {
            coordinates: ls.coords().map(|c| Coordinate::new(c.x, c.y)).collect(),
        }
    }

    pub fn to_vecs(&self) -> Vec<Vec<f64>> {
        self.coordinates.iter().map(|c| c.to_vec()).collect()
    }
}

impl FromIterator<Coordinate> for Line {
    /// Create a new line from an iterator of coordinates
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator of coordinates
    ///
    /// # Returns
    ///
    /// * `Line` - A new line
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::from_iter(coords);
    /// ```
    fn from_iter<T: IntoIterator<Item = Coordinate>>(iter: T) -> Self {
        Self {
            coordinates: iter.into_iter().collect(),
        }
    }
}

/// A collection of lines that form a polygon
#[derive(Debug, Clone)]
pub struct Polygon {
    pub exterior: Line,
    pub interiors: Vec<Line>,
}

impl Polygon {
    /// Create a new polygon from an exterior line and interior lines
    ///
    /// # Arguments
    ///
    /// * `exterior` - The exterior line
    /// * `interiors` - The interior lines
    ///
    /// # Returns
    ///
    /// * `Polygon` - A new polygon
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    /// use proj_exercise_simple::coordinates::Polygon;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let polygon = Polygon::new(line, vec![]);
    /// ```
    pub fn new(exterior: Line, interiors: Vec<Line>) -> Self {
        Self {
            exterior,
            interiors,
        }
    }

    /// Convert a polygon to a GeoJSON polygon
    ///
    /// # Returns
    ///
    /// * `geojson::Value` - A GeoJSON polygon
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    /// use proj_exercise_simple::coordinates::Polygon;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let polygon = Polygon::new(line, vec![]);
    /// let geojson = polygon.to_geojson();
    /// ```
    pub fn to_geojson(&self) -> geojson::Value {
        let mut rings = vec![Coordinate::to_vecs(&self.exterior.coordinates)];
        rings.extend(
            self.interiors
                .iter()
                .map(|line| Coordinate::to_vecs(&line.coordinates)),
        );
        geojson::Value::Polygon(rings)
    }

    /// Convert a polygon to a geo polygon
    ///
    /// # Returns
    ///
    /// * `geo::Polygon<f64>` - A geo polygon
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    /// use proj_exercise_simple::coordinates::Polygon;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let polygon = Polygon::new(line, vec![]);
    /// let geo = polygon.to_geo();
    /// ```
    pub fn to_geo(&self) -> geo::Polygon<f64> {
        let exterior = self.exterior.to_geo();
        let interiors = self
            .interiors
            .iter()
            .map(|line| line.to_geo())
            .collect::<Vec<_>>();
        geo::Polygon::new(exterior, interiors)
    }
}

impl FromIterator<Line> for Polygon {
    /// Create a new polygon from an iterator of lines
    ///
    /// # Arguments
    ///
    /// * `iter` - An iterator of lines
    ///
    /// # Returns
    ///
    /// * `Polygon` - A new polygon
    ///
    /// # Example
    ///
    /// ```rust
    /// use proj_exercise_simple::coordinates::Coordinate;
    /// use proj_exercise_simple::coordinates::Line;
    /// use proj_exercise_simple::coordinates::Polygon;
    ///
    /// let coords = vec![Coordinate::new(13.377, 52.518), Coordinate::new(13.377, 52.518)];
    /// let line = Line::new(coords);
    /// let polygon = Polygon::from_iter(vec![line]);
    /// ```
    fn from_iter<T: IntoIterator<Item = Line>>(iter: T) -> Self {
        let mut lines: Vec<Line> = iter.into_iter().collect();
        if lines.is_empty() {
            panic!("Cannot create polygon from empty iterator");
        }
        let exterior = lines.remove(0);
        Self {
            exterior,
            interiors: lines,
        }
    }
}
