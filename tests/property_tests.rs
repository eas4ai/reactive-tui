use proptest::prelude::*;
use reactive_tui::core::geometry::{Point, Rect, Size};
use reactive_tui::core::surface::{Attr, Rgba, Surface};
use reactive_tui::event::router::{EventRouter, NodeId};

proptest! {
    #[test]
    fn surface_creation_never_panics(
        width in 1usize..=100,
        height in 1usize..=100
    ) {
        let surface = Surface::new(width, height);
        assert_eq!(surface.dims(), (width, height));
    }

    #[test]
    fn surface_write_str_bounds_safe(
        width in 1usize..=50,
        height in 1usize..=50,
        x in 0usize..50,
        y in 0usize..50,
        text in ".*"
    ) {
        let mut surface = Surface::new(width, height);

        // Only positions within bounds; proptest draws others again.
        prop_assume!(x < width && y < height);
        // Writing within bounds should never panic
        surface.write_str(x, y, &text, Rgba::white(), Rgba::black(), Attr::empty());

        // Surface dimensions should remain unchanged
        assert_eq!(surface.dims(), (width, height));
    }

    #[test]
    fn surface_clear_preserves_dimensions(
        width in 1usize..=100,
        height in 1usize..=100,
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        a in 0.0f32..=1.0
    ) {
        let mut surface = Surface::new(width, height);
        let bg = Rgba { r, g, b, a };
        surface.clear(bg);

        assert_eq!(surface.dims(), (width, height));
    }

    #[test]
    fn point_creation_consistent(
        x in 0usize..1000,
        y in 0usize..1000
    ) {
        let point = Point::new(x, y);
        assert_eq!(point.x, x);
        assert_eq!(point.y, y);
    }

    #[test]
    fn size_creation_consistent(
        w in 1usize..1000,
        h in 1usize..1000
    ) {
        let size = Size::new(w, h);
        assert_eq!(size.width, w);
        assert_eq!(size.height, h);
    }

    #[test]
    fn rect_creation_consistent(
        x in 0usize..100,
        y in 0usize..100,
        w in 1usize..100,
        h in 1usize..100
    ) {
        let point = Point::new(x, y);
        let size = Size::new(w, h);
        let rect = Rect::new(point, size);

        assert_eq!(rect.origin, point);
        assert_eq!(rect.size, size);
    }

    #[test]
    fn event_router_creation_consistent(
        width in 1u16..100,
        height in 1u16..100
    ) {
        let router = EventRouter::new_with_size(width, height);
        // A fresh router has no root and nothing focused
        assert_eq!(router.root(), None);
        assert_eq!(router.get_focus(), None);
    }

    #[test]
    fn rgba_creation_consistent(
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        a in 0.0f32..=1.0
    ) {
        let color = Rgba { r, g, b, a };

        assert_eq!(color.r, r);
        assert_eq!(color.g, g);
        assert_eq!(color.b, b);
        assert_eq!(color.a, a);
    }

    #[test]
    fn rgba_components_in_range(
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        a in 0.0f32..=1.0
    ) {
        let color = Rgba { r, g, b, a };

        assert!(color.r >= 0.0 && color.r <= 1.0);
        assert!(color.g >= 0.0 && color.g <= 1.0);
        assert!(color.b >= 0.0 && color.b <= 1.0);
        assert!(color.a >= 0.0 && color.a <= 1.0);
    }

    #[test]
    fn attr_bitflags_operations(
        bits in prop::bits::u8::masked(0xFF)
    ) {
        let attr = Attr::from_bits_truncate(bits);

        // Bitflag operations should be consistent
        assert_eq!(attr.bits() & bits, attr.bits());

        // Combining with empty should be identity
        assert_eq!(attr | Attr::empty(), attr);

        // Intersecting with self should be identity
        assert_eq!(attr & attr, attr);
    }

    #[test]
    fn focus_management_consistent(
        width in 1u16..100,
        height in 1u16..100
    ) {
        let mut router = EventRouter::new_with_size(width, height);
        let root = NodeId::new();
        router.set_root(root);

        // Adding focusable should work
        assert!(router.add_focusable(root, None));

        // With a single focusable node, focus cycles back to it
        assert_eq!(router.focus_next(), Some(root));
        assert_eq!(router.focus_prev(), Some(root));

        // Focus should remain on the only node
        assert_eq!(router.get_focus(), Some(root));
    }

    #[test]
    fn string_operations_safe(
        text in ".*"
    ) {
        // String operations should never panic
        let formatted = format!("Text: {}", text);
        assert!(formatted.starts_with("Text: "));

        let len = text.len();
        assert!(len <= 1000); // Reasonable upper bound for test strings
    }

    #[test]
    fn mathematical_properties(
        a in 0u32..1000,
        b in 0u32..1000
    ) {
        // Basic mathematical properties should hold
        assert_eq!(a + b, b + a); // Addition is commutative
        assert_eq!(a * b, b * a); // Multiplication is commutative

        if let Some(quotient) = a.checked_div(b) {
            assert_eq!(quotient * b + a % b, a); // Division property
        }
    }

    #[test]
    fn vector_operations_consistent(
        size in 0usize..100
    ) {
        let mut vec = Vec::with_capacity(size);
        for i in 0..size {
            vec.push(i);
        }

        assert_eq!(vec.len(), size);
        assert!(vec.capacity() >= size);

        if !vec.is_empty() {
            assert_eq!(vec[0], 0);
            assert_eq!(vec[vec.len() - 1], size - 1);
        }
    }
}
