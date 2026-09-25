//! Stable JSON representation of routed native input; timestamps stay native.
use crate::event::types::*;
use serde_json::{json, Value};

fn modifiers(value: KeyModifiers) -> Value {
    json!({"shift":value.shift,"ctrl":value.ctrl,"alt":value.alt,"meta":value.meta})
}

pub(super) fn encode(event: &Event) -> String {
    match event {
        Event::Key(key) => {
            let name = match &key.code {
                KeyCode::Char(value) => value.to_string(),
                KeyCode::F(value) => format!("F{value}"),
                value => format!("{value:?}"),
            };
            json!({"type":"key","key":name,"kind":format!("{:?}",key.kind).to_lowercase(),
                   "repeat":key.repeat,"modifiers":modifiers(key.modifiers)})
        }
        Event::Mouse(mouse) => {
            let wheel = mouse.wheel.as_ref().map(|wheel| {
                let (unit,x,y) = match wheel.delta {
                    WheelDelta::Lines{x,y} => ("lines",x,y),
                    WheelDelta::Pixels{x,y} => ("pixels",x,y),
                };
                json!({"unit":unit,"x":x,"y":y,"phase":format!("{:?}",wheel.phase).to_lowercase()})
            });
            json!({"type":"mouse","kind":format!("{:?}",mouse.kind).to_lowercase(),
                   "button":format!("{:?}",mouse.button).to_lowercase(),"x":mouse.position.x(),"y":mouse.position.y(),
                   "unit":if matches!(mouse.position,Position::Cell{..}) {"cells"} else {"pixels"},
                   "modifiers":modifiers(mouse.modifiers),"wheel":wheel})
        }
        Event::Paste(paste) => json!({"type":"paste","text":paste.content}),
        Event::Resize(size) => json!({"type":"resize","width":size.width,"height":size.height,
                                     "pixelWidth":size.pixel_width,"pixelHeight":size.pixel_height}),
        Event::Focus(focus) => json!({"type":"focus","kind":format!("{:?}",focus.kind).to_lowercase()}),
        Event::Custom(custom) => json!({"type":"custom","name":custom.name,"data":custom.data}),
    }.to_string()
}
