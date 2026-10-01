//! Permissive hold: how a tap-hold key is decided when other keys are typed
//! while it is down.
//!
//! A tap-hold key is undecided (pending) from its press until one of:
//! - it is released first → **tap** (its tap key is typed),
//! - a key pressed after it is also released while it is still down (a
//!   complete tap "inside" it) → **hold** (its modifier/layer applies) -
//!   except for `tap_hold_timeout_only` keys, which ignore typing entirely,
//! - its threshold passes → **hold** (see [`crate::runtime::Remapper::tick`]).
//!
//! Keys pressed while it is pending are buffered and replayed, in order, once
//! it is decided - so they are looked up in the right layer. This is QMK's
//! PERMISSIVE_HOLD. The earlier behaviour resolved HOLD on the mere press of
//! another key, which turns ordinary typing rollover into layer keys: with
//! `tap_hold("B", "Enter", "MD_00")`, `B↓ E↓ B↑ E↑` produced the MD_00
//! mapping of E instead of Enter, E.
//!
//! Releases of keys pressed *before* the tap-hold key are not buffered:
//! their presses were already sent.

extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use crate::runtime::event::{
    check_tap_hold_timeouts, process_event_for_identities, resolve_pending_as_hold,
};
use crate::runtime::{DeviceState, KeyEvent, KeyLookup};

/// Most events held back while a tap-hold key is undecided. Past this the
/// pending keys are resolved as hold so the buffer cannot grow unbounded.
const MAX_BUFFERED: usize = 32;

/// One device's view of the engine, as [`feed`] needs it.
pub struct DeviceCtx<'a> {
    pub lookup: &'a KeyLookup,
    pub state: &'a mut DeviceState,
    pub identities: &'a [&'a str],
    /// Events held back while a tap-hold key is undecided (per device).
    pub buffer: &'a mut Vec<KeyEvent>,
}

/// Runs `event` through permissive hold and the mapping engine; returns the
/// output events (possibly none while the event is buffered).
pub fn feed(ctx: &mut DeviceCtx<'_>, event: KeyEvent) -> Vec<KeyEvent> {
    let mut out = Vec::new();
    let mut queue = VecDeque::from([event]);
    while let Some(event) = queue.pop_front() {
        step(ctx, event, &mut out, &mut queue);
    }
    out
}

/// Resolves pending tap-hold keys whose threshold passed at `now_us`, and
/// replays what was buffered behind them once none is pending any more.
pub fn tick(ctx: &mut DeviceCtx<'_>, now_us: u64) -> Vec<KeyEvent> {
    let mut out = check_tap_hold_timeouts(now_us, ctx.state);
    if !ctx.buffer.is_empty() && !has_pending(ctx.state) {
        let buffered: Vec<KeyEvent> = ctx.buffer.drain(..).collect();
        for event in buffered {
            out.extend(feed(ctx, event));
        }
    }
    out
}

fn has_pending(state: &DeviceState) -> bool {
    state.tap_hold_processor_ref().has_pending_keys()
}

fn step(
    ctx: &mut DeviceCtx<'_>,
    event: KeyEvent,
    out: &mut Vec<KeyEvent>,
    queue: &mut VecDeque<KeyEvent>,
) {
    if ctx.buffer.is_empty() {
        if event.is_press() && has_pending(ctx.state) {
            ctx.buffer.push(event);
        } else {
            out.extend(run(ctx, event));
        }
        return;
    }
    let key = event.keycode();
    if event.is_press() {
        ctx.buffer.push(event);
        if ctx.buffer.len() > MAX_BUFFERED {
            out.extend(resolve_pending_as_hold(ctx.state, event_time(ctx)));
            replay_front(ctx, queue, None);
        }
    } else if ctx.state.tap_hold_processor_ref().is_pending(key) {
        // Released before anything typed inside it completed: a tap.
        out.extend(run(ctx, event));
        replay_front(ctx, queue, None);
    } else if ctx
        .buffer
        .iter()
        .any(|b| b.is_press() && b.keycode() == key)
    {
        if ctx.state.tap_hold_processor_ref().has_permissive_pending() {
            // A key pressed inside it was released while it is still down.
            out.extend(resolve_pending_as_hold(ctx.state, event.timestamp_us()));
            replay_front(ctx, queue, Some(event));
        } else {
            // Timeout-only tap-hold: typing never decides it. Keep the
            // release behind its press until the tap-hold is decided.
            ctx.buffer.push(event);
        }
    } else {
        // A key pressed before the tap-hold key: its press is already out.
        out.extend(run(ctx, event));
    }
}

/// Puts the buffered events (then `then`, if any) at the front of the queue,
/// in their original order, so they are processed before anything later.
fn replay_front(ctx: &mut DeviceCtx<'_>, queue: &mut VecDeque<KeyEvent>, then: Option<KeyEvent>) {
    if let Some(event) = then {
        queue.push_front(event);
    }
    for event in ctx.buffer.drain(..).rev() {
        queue.push_front(event);
    }
}

fn event_time(ctx: &DeviceCtx<'_>) -> u64 {
    ctx.buffer.last().map_or(0, KeyEvent::timestamp_us)
}

fn run(ctx: &mut DeviceCtx<'_>, event: KeyEvent) -> Vec<KeyEvent> {
    process_event_for_identities(event, ctx.lookup, ctx.state, ctx.identities)
}
