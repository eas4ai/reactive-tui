# Chart plot pictures

Reference pictures for `tests/charts_pictures.rs` (CHT-012, CHT-013 and
CHT-037): the plot area of a line, area, scatter, bar or candlestick chart
as the chart draws it on a terminal that takes pixels, at 80 by 24 cells of
8 by 16 pixels, drawn by the software renderer (GFX-002 makes the hardware
adapter's picture the same within its tolerance). One PNG per variant,
compared with GFX-002's tolerance. Regenerate with `REGENERATE=1`, look at
the pictures, then commit; nothing else writes here.
