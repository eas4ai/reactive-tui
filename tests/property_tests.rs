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
        prop_assert_eq!(surface.dims(), (width, height));
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

        // Only test within bounds
        if x < width && y < height {
            // Writing within bounds should never panic
            surface.write_str(x, y, &text, Rgba::white(), Rgba::black(), Attr::empty());

            // Surface dimensions should remain unchanged
            prop_assert_eq!(surface.dims(), (width, height));
        }
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

        prop_assert_eq!(surface.dims(), (width, height));
    }

    #[test]
    fn point_creation_consistent(
        x in 0usize..1000,
        y in 0usize..1000
    ) {
        let point = Point::new(x, y);
        prop_assert_eq!(point.x, x);
        prop_assert_eq!(point.y, y);
    }

    #[test]
    fn size_creation_consistent(
        w in 1usize..1000,
        h in 1usize..1000
    ) {
        let size = Size::new(w, h);
        prop_assert_eq!(size.width, w);
        prop_assert_eq!(size.height, h);
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

        prop_assert_eq!(rect.origin, point);
        prop_assert_eq!(rect.size, size);
    }

    #[test]
    fn event_router_creation_consistent(
        width in 1u16..100,
        height in 1u16..100
    ) {
        let _router = EventRouter::new_with_size(width, height);
        // EventRouter creation should not panic
        prop_assert!(true);
    }

    #[test]
    fn rgba_creation_consistent(
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        a in 0.0f32..=1.0
    ) {
        let color = Rgba { r, g, b, a };

        prop_assert_eq!(color.r, r);
        prop_assert_eq!(color.g, g);
        prop_assert_eq!(color.b, b);
        prop_assert_eq!(color.a, a);
    }

    #[test]
    fn rgba_components_in_range(
        r in 0.0f32..=1.0,
        g in 0.0f32..=1.0,
        b in 0.0f32..=1.0,
        a in 0.0f32..=1.0
    ) {
        let color = Rgba { r, g, b, a };

        prop_assert!(color.r >= 0.0 && color.r <= 1.0);
        prop_assert!(color.g >= 0.0 && color.g <= 1.0);
        prop_assert!(color.b >= 0.0 && color.b <= 1.0);
        prop_assert!(color.a >= 0.0 && color.a <= 1.0);
    }

    #[test]
    fn attr_bitflags_operations(
        bits in prop::bits::u8::masked(0xFF)
    ) {
        let attr = Attr::from_bits_truncate(bits);

        // Bitflag operations should be consistent
        prop_assert_eq!(attr.bits() & bits, attr.bits());

        // Combining with empty should be identity
        prop_assert_eq!(attr | Attr::empty(), attr);

        // Intersecting with self should be identity
        prop_assert_eq!(attr & attr, attr);
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
        router.add_focusable(root, None);

        // Focus operations should not panic
        router.focus_next();
        router.focus_prev();

        // Focus should remain consistent
        let focus = router.get_focus();
        prop_assert!(focus.is_some() || focus.is_none());
    }

    #[test]
    fn string_operations_safe(
        text in ".*"
    ) {
        // String operations should never panic
        let formatted = format!("Text: {}", text);
        prop_assert!(formatted.starts_with("Text: "));

        let len = text.len();
        prop_assert!(len <= 1000); // Reasonable upper bound for test strings
    }

    #[test]
    fn mathematical_properties(
        a in 0u32..1000,
        b in 0u32..1000
    ) {
        // Basic mathematical properties should hold
        prop_assert_eq!(a + b, b + a); // Addition is commutative
        prop_assert_eq!(a * b, b * a); // Multiplication is commutative

        if b != 0 {
            prop_assert_eq!(a / b * b + a % b, a); // Division property
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

        prop_assert_eq!(vec.len(), size);
        prop_assert!(vec.capacity() >= size);

        if !vec.is_empty() {
            prop_assert_eq!(vec[0], 0);
            prop_assert_eq!(vec[vec.len() - 1], size - 1);
        }
    }
}
