use crate::{Wkt, WktFloat};

/// A trait for converting values to WKT
pub trait ToWkt<T>
where
    T: WktFloat + std::fmt::Display,
{
    /// Converts the value of `self` to an [`Wkt`] struct.
    ///
    /// Typically you won't need to call this, but by implementing it for your own type, your type
    /// gains the other methods in this trait.
    fn to_wkt(&self) -> Wkt<T>;

    /// Serialize as a WKT string
    ///
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
    ///
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
    fn write_wkt(&self, mut writer: impl std::io::Write) -> std::io::Result<()> {
        writer.write_all(self.wkt_string().as_bytes())
    }
}
