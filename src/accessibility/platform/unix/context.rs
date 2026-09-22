// Copyright 2023 The AccessKit Authors. All rights reserved.
// Licensed under the Apache License, Version 2.0 (found in
// the LICENSE-APACHE file) or the MIT license (found in
// the LICENSE-MIT file), at your option.

use crate::accessibility::platform::translation::{Adapter as AdapterImpl, AppContext, Event};
use accesskit::{ActivationHandler, DeactivationHandler};
use async_channel::Receiver;
use atspi::proxy::bus::StatusProxy;
use futures_util::{pin_mut as pin, select, StreamExt};
use std::sync::{Arc, Mutex, RwLock};
use zbus::{connection::Builder, Connection};

use crate::accessibility::platform::unix::{
    adapter::{AdapterState, Callback, Message},
    atspi::Bus,
    executor::Executor,
    transport::{timed, MessageSender, Result},
};

pub(super) fn new_app_context() -> Arc<RwLock<AppContext>> {
    let name = std::env::current_exe().ok().and_then(|path| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
    });
    AppContext::new(name)
}

pub(super) async fn run(
    executor: &Executor<'_>,
    context: Arc<RwLock<AppContext>>,
    rx: Receiver<Message>,
    messages: MessageSender,
) -> Result<()> {
    let session_bus = timed("session-bus connection", async {
        Builder::session()?.internal_executor(false).build().await
    })
    .await?;
    run_event_loop(executor, session_bus, context, rx, messages).await
}

struct AdapterEntry {
    context: Arc<RwLock<AppContext>>,
    messages: MessageSender,
    id: usize,
    activation_handler: Box<dyn ActivationHandler>,
    deactivation_handler: Box<dyn DeactivationHandler>,
    state: Arc<Mutex<AdapterState>>,
}

fn activate_adapter(entry: &mut AdapterEntry) {
    let mut state = entry.state.lock().unwrap();
    if let AdapterState::Inactive {
        is_window_focused,
        root_window_bounds,
        action_handler,
    } = &*state
    {
        *state = match entry.activation_handler.request_initial_tree() {
            Some(initial_state) => {
                let r#impl = AdapterImpl::with_wrapped_action_handler(
                    entry.id,
                    &entry.context,
                    Callback::new(entry.messages.clone()),
                    initial_state,
                    *is_window_focused,
                    *root_window_bounds,
                    Arc::clone(action_handler),
                );
                AdapterState::Active(r#impl)
            }
            None => AdapterState::Pending {
                is_window_focused: *is_window_focused,
                root_window_bounds: *root_window_bounds,
                action_handler: Arc::clone(action_handler),
            },
        };
    }
}

fn deactivate_adapter(entry: &mut AdapterEntry) {
    let mut state = entry.state.lock().unwrap();
    match &*state {
        AdapterState::Inactive { .. } => (),
        AdapterState::Pending {
            is_window_focused,
            root_window_bounds,
            action_handler,
        } => {
            *state = AdapterState::Inactive {
                is_window_focused: *is_window_focused,
                root_window_bounds: *root_window_bounds,
                action_handler: Arc::clone(action_handler),
            };
            drop(state);
            entry.deactivation_handler.deactivate_accessibility();
        }
        AdapterState::Active(r#impl) => {
            *state = AdapterState::Inactive {
                is_window_focused: r#impl.is_window_focused(),
                root_window_bounds: r#impl.root_window_bounds(),
                action_handler: r#impl.wrapped_action_handler(),
            };
            drop(state);
            entry.deactivation_handler.deactivate_accessibility();
        }
    }
}

fn sync_adapters(adapters: &mut [AdapterEntry], atspi_bus: &Option<Bus>) {
    let active = atspi_bus.is_some();
    for entry in adapters {
        if active {
            activate_adapter(entry);
        } else {
            deactivate_adapter(entry);
        }
    }
}

async fn run_event_loop(
    executor: &Executor<'_>,
    session_bus: Connection,
    context: Arc<RwLock<AppContext>>,
    rx: Receiver<Message>,
    sender: MessageSender,
) -> Result<()> {
    let session_bus_copy = session_bus.clone();
    let _session_bus_task = executor.spawn(
        async move {
            loop {
                session_bus_copy.executor().tick().await;
            }
        },
        "accesskit_session_bus_task",
    );

    let status = timed("accessibility status proxy", StatusProxy::new(&session_bus)).await?;
    let changes = timed("accessibility status subscription", async {
        Ok(status.receive_is_enabled_changed().await)
    })
    .await?
    .fuse();
    pin!(changes);

    let messages = rx.fuse();
    pin!(messages);

    let mut atspi_bus = None;
    let mut adapters: Vec<AdapterEntry> = Vec::new();

    loop {
        select! {
            change = changes.next() => {
                let change = change.ok_or("screen-reader status connection closed")?;
                let enabled = timed("accessibility status read", change.get()).await?;
                if enabled && atspi_bus.is_none() {
                    atspi_bus = Some(timed("AT-SPI connection", Bus::new(&session_bus, executor, &context)).await?);
                } else if !enabled {
                    atspi_bus = None;
                }
                sync_adapters(&mut adapters, &atspi_bus);

            }
            message = messages.next() => {
                let message = message.ok_or("screen-reader outgoing queue closed")?;
                timed("AT-SPI publication", process_adapter_message(&atspi_bus, &mut adapters, &context, &sender, message)).await?;
            }
        }
    }
}

async fn process_adapter_message(
    atspi_bus: &Option<Bus>,
    adapters: &mut Vec<AdapterEntry>,
    context: &Arc<RwLock<AppContext>>,
    messages: &MessageSender,
    message: Message,
) -> zbus::Result<()> {
    match message {
        Message::AddAdapter {
            id,
            activation_handler,
            deactivation_handler,
            state,
        } => {
            adapters.push(AdapterEntry {
                context: context.clone(),
                messages: messages.clone(),
                id,
                activation_handler,
                deactivation_handler,
                state,
            });
            if atspi_bus.is_some() {
                let entry = adapters.last_mut().unwrap();
                activate_adapter(entry);
            }
        }
        Message::RemoveAdapter { id } => {
            if let Ok(index) = adapters.binary_search_by(|entry| entry.id.cmp(&id)) {
                adapters.remove(index);
            }
        }
        Message::RegisterInterfaces { node, interfaces } => {
            if let Some(bus) = atspi_bus {
                bus.register_interfaces(node, interfaces).await?
            }
        }
        Message::UnregisterInterfaces {
            adapter_id,
            node_id,
            interfaces,
        } => {
            if let Some(bus) = atspi_bus {
                bus.unregister_interfaces(adapter_id, node_id, interfaces)
                    .await?
            }
        }
        Message::EmitEvent {
            adapter_id,
            event: Event::Object { target, event },
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_object_event(adapter_id, target, event).await?
            }
        }
        Message::EmitEvent {
            adapter_id,
            event:
                Event::Window {
                    target,
                    name,
                    event,
                },
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_window_event(adapter_id, target, name, event)
                    .await?;
            }
        }
        Message::EmitEvent {
            adapter_id,
            event: Event::Document { target, event },
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_document_event(adapter_id, target, event).await?;
            }
        }
        Message::EmitEvent {
            event: Event::Cache(_),
            ..
        } => unreachable!("cache events are sent as EmitCacheAdd/EmitCacheRemove"),
        Message::EmitCacheAdd { node } => {
            if let Some(bus) = atspi_bus {
                bus.emit_cache_add(node).await?;
            }
        }
        Message::EmitCacheRemove {
            adapter_id,
            node_id,
        } => {
            if let Some(bus) = atspi_bus {
                bus.emit_cache_remove(adapter_id, node_id).await?;
            }
        }
    }

    Ok(())
}
