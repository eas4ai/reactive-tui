//! Input pieces for the real reader workflow (CTL-011): two number inputs, a
//! switch, a rating and a masked code input, in this focus order.
use reactive_tui::{builder, component::Element};

pub(super) fn inputs() -> Element {
    builder::div()
        .class("w-full h-full flex flex-col gap-1")
        .child(Element::text("Input pieces"))
        .child(
            builder::number_input()
                .aria_label("Volume")
                .value(0.0)
                .min(0.0)
                .max(10.0)
                .step(1.0)
                .class("w-30")
                .build()
                .auto_focus(),
        )
        .child(
            builder::number_input()
                .aria_label("Year")
                .value(1990.0)
                .min(1900.0)
                .max(2100.0)
                .step(1.0)
                .class("w-30")
                .build(),
        )
        .child(builder::switch().label("Wi-Fi").build())
        .child(
            builder::rating()
                .aria_label("Stars")
                .max(5)
                .value(3)
                .build(),
        )
        .child(
            builder::code_input()
                .label("Code")
                .length(6)
                .masked(true)
                .build(),
        )
        .build()
}
