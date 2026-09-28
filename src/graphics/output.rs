//! How a canvas shows its picture (GFX-005): as Kitty graphics where the
//! host takes them, else as Sixel, else as block glyphs, unless the
//! environment or the application says otherwise. The painter knows what
//! the host takes; it tells each canvas it paints through the canvas's
//! link, and the canvas draws its next picture for that.

use super::GraphicsFrame;
use crate::reactive::ThreadSafeSignal;
use std::sync::{Arc, Mutex};

/// How a canvas shows its picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CanvasOutput {
    /// Pixels through the Kitty graphics protocol.
    Kitty,
    /// Pixels as Sixel.
    Sixel,
    /// Block glyphs, drawn with the blitter image fallback uses (BLT-002).
    Blocks,
}

impl CanvasOutput {
    /// The environment variable that names the output for every canvas:
    /// `kitty`, `sixel` or `blocks`. It wins over the application's choice,
    /// so a user can correct a terminal that reports what it cannot show.
    pub const ENV: &'static str = "REACTIVE_TUI_CANVAS";

    /// The name [`CanvasOutput::from_name`] reads.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Kitty => "kitty",
            Self::Sixel => "sixel",
            Self::Blocks => "blocks",
        }
    }

    /// The output a name selects, ignoring case and surrounding space.
    pub fn from_name(name: &str) -> Option<Self> {
        [Self::Kitty, Self::Sixel, Self::Blocks]
            .into_iter()
            .find(|output| output.name().eq_ignore_ascii_case(name.trim()))
    }

    /// The output the environment names, if it names one.
    pub(crate) fn from_environment() -> Option<Self> {
        let value = std::env::var(Self::ENV).ok()?;
        let named = Self::from_name(&value);
        if named.is_none() && !value.trim().is_empty() {
            log::warn!(
                "{}={value:?} names no canvas output; use kitty, sixel or blocks",
                Self::ENV
            );
        }
        named
    }

    /// The output of a canvas: the environment's, else the application's,
    /// else what the host takes, Kitty graphics before Sixel before block
    /// glyphs.
    pub(crate) fn choose(
        host: Option<HostReport>,
        application: Option<Self>,
        environment: Option<Self>,
    ) -> Self {
        environment.or(application).unwrap_or(match host {
            Some(host) if host.kitty => Self::Kitty,
            Some(host) if host.sixel => Self::Sixel,
            _ => Self::Blocks,
        })
    }
}

/// What the host takes, as the painter reports it to a canvas.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HostReport {
    pub kitty: bool,
    /// Kitty graphics may travel through shared memory.
    pub kitty_shared_memory: bool,
    pub sixel: bool,
    /// The pixels of one cell.
    pub cell: (u16, u16),
}

/// The line between a canvas and the painter that paints it.
pub(crate) struct CanvasLink {
    host: Mutex<Option<HostReport>>,
    /// Counts the reports that differed from the one before.
    reports: std::sync::atomic::AtomicU64,
    changed: ThreadSafeSignal<u64>,
}

impl Default for CanvasLink {
    fn default() -> Self {
        Self {
            host: Mutex::default(),
            reports: std::sync::atomic::AtomicU64::new(0),
            changed: ThreadSafeSignal::new(0),
        }
    }
}

impl CanvasLink {
    /// The painter's report; a new one has the canvas's App redraw.
    pub fn report(&self, report: HostReport) {
        let mut host = self.host.lock().unwrap_or_else(|e| e.into_inner());
        if *host == Some(report) {
            return;
        }
        *host = Some(report);
        drop(host);
        let reports = self
            .reports
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.changed.set(reports + 1);
    }

    /// What the host takes; `None` until a painter has painted the canvas.
    pub fn host(&self) -> Option<HostReport> {
        *self.host.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Have the App that is rendering redraw when the report changes.
    pub fn observe(&self) {
        self.changed.get();
    }
}

/// What a canvas's element hands the painter.
pub(crate) struct CanvasPaint {
    /// Names the picture to the terminal; one per canvas.
    pub id: u32,
    pub link: Arc<CanvasLink>,
    /// The picture to show as pixels, and how: Kitty graphics or Sixel.
    pub pixels: Option<(Arc<GraphicsFrame>, CanvasOutput)>,
}

impl PartialEq for CanvasPaint {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && Arc::ptr_eq(&self.link, &other.link)
            && match (&self.pixels, &other.pixels) {
                (Some(a), Some(b)) => Arc::ptr_eq(&a.0, &b.0) && a.1 == b.1,
                (None, None) => true,
                _ => false,
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOST: HostReport = HostReport {
        kitty: true,
        kitty_shared_memory: false,
        sixel: true,
        cell: (8, 16),
    };

    #[test]
    fn the_host_decides_unless_the_application_or_the_environment_does() {
        use CanvasOutput::{Blocks, Kitty, Sixel};
        assert_eq!(CanvasOutput::choose(Some(HOST), None, None), Kitty);
        let sixel_only = HostReport {
            kitty: false,
            ..HOST
        };
        assert_eq!(CanvasOutput::choose(Some(sixel_only), None, None), Sixel);
        let neither = HostReport {
            sixel: false,
            ..sixel_only
        };
        assert_eq!(CanvasOutput::choose(Some(neither), None, None), Blocks);
        assert_eq!(CanvasOutput::choose(None, None, None), Blocks);
        assert_eq!(CanvasOutput::choose(Some(HOST), Some(Blocks), None), Blocks);
        assert_eq!(
            CanvasOutput::choose(Some(HOST), Some(Blocks), Some(Sixel)),
            Sixel
        );
    }

    #[test]
    fn names_read_back_and_a_new_report_is_kept() {
        for output in [
            CanvasOutput::Kitty,
            CanvasOutput::Sixel,
            CanvasOutput::Blocks,
        ] {
            assert_eq!(CanvasOutput::from_name(output.name()), Some(output));
        }
        assert_eq!(CanvasOutput::from_name(" KITTY "), Some(CanvasOutput::Kitty));
        assert_eq!(CanvasOutput::from_name("pixels"), None);
        let link = CanvasLink::default();
        assert_eq!(link.host(), None);
        link.report(HOST);
        assert_eq!(link.host(), Some(HOST));
    }
}
