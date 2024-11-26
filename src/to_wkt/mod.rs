use crate::{Wkt, WktNum};

mod geo_trait_impl;

pub use geo_trait_impl::{
    write_geometry, write_geometry_collection, write_line, write_linestring,
    write_multi_linestring, write_multi_point, write_multi_polygon, write_point, write_polygon,
    write_rect, write_triangle,
};

/// A wrapper around something that implements std::io::Write to be used with our writer traits,
/// which require std::fmt::Write
struct WriterWrapper<W: std::io::Write>(W);

impl<W: std::io::Write> std::fmt::Write for WriterWrapper<W> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        // Sadly, this will lose the content of the error when mapping to std::fmt::Error
        self.0.write(s.as_bytes()).map_err(|_| std::fmt::Error)?;
        Ok(())
    }
}

/// A trait for converting values to WKT
pub trait ToWkt<T>
where
    T: WktNum + std::fmt::Display,
{
    /// Converts the value of `self` to an [`Wkt`] struct.
    ///
    /// Typically you won't need to call this, but by implementing it for your own type, your type
    /// gains the other methods in this trait.
    fn to_wkt(&self) -> Wkt<T>;

    /// Serialize as a WKT string
    #[cfg_attr(feature = "geo-types", doc = "```")]
    #[cfg_attr(not(feature = "geo-types"), doc = "```ignore")]
    /// // This example requires the geo-types feature (on by default).
    /// use wkt::ToWkt;
    /// let point: geo_types::Point<f64> = geo_types::point!(x: 1.2, y: 3.4);
    /// assert_eq!("POINT(1.2 3.4)", &point.wkt_string());
    /// ```
    fn wkt_string(&self) -> String {
        self.to_wkt().to_string()
    }

    /// Write a WKT string to a [`File`](std::fs::File), or anything else that implements [`Write`](std::io::Write).
    #[cfg_attr(feature = "geo-types", doc = "```")]
    #[cfg_attr(not(feature = "geo-types"), doc = "```ignore")]
    /// // This example requires the geo-types feature (on by default).
    /// use wkt::ToWkt;
    /// use std::fs::File;
    /// let point: geo_types::Point<f64> = geo_types::point!(x: 1.2, y: 3.4);
    ///
    /// // use a vec as a fake "file" for the purpose of example, but you could equally replace the
    /// // following with:
    /// //     let mut file = File::create("myfile.wkt").unwrap();
    /// let mut file = vec![] ;
    ///
    /// point.write_wkt(&mut file).unwrap();
    /// let wkt_string = String::from_utf8(file).unwrap();
    ///
    /// assert_eq!(wkt_string, "POINT(1.2 3.4)");
    /// ```
    ///
    /// ## Panics
    ///
    /// - If
    fn write_wkt(&self, writer: impl std::io::Write) -> std::io::Result<()> {
        write_geometry(&mut WriterWrapper(writer), &self.to_wkt())
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    }
}
