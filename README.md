The general algorithm used for rasterization is a vertical sweep of
the shape that maintains an active edge list.  The sweep is done
at a sub-scanline resolution and results in either:
   1. Sub-scanlines being combined in the coverage buffer and output
      as "complex scans".
   2. Simple trapezoids being recognized in the active edge list
      and output using a faster simple trapezoid path.
