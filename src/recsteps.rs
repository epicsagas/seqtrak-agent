//! Clock-synced note scheduling: fires notes on the unit's transmitted MIDI
//! clock so its real-time recorder (arm REC on the panel, transport running)
//! captures them into the pattern memory. This is the write path into the
//! unit's own sequencer: the Data List documents no step-note SysEx, but the
//! unit records incoming MIDI notes while recording (User Guide panel item 10).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use midir::MidiOutputConnection;

use crate::midi;

struct Sched {
    tick: AtomicUsize,
    sub: AtomicUsize,
    /// (absolute 16th index from start, note, velocity)
    notes: Vec<(usize, u8, u8)>,
    /// (channel, note, tick to send note off)
    active: Mutex<Vec<(u8, u8, usize)>>,
}

/// Fire `notes` on 16th-grid steps while the unit plays. Sends start; stops
/// transport at the end unless `keep_playing`. `gate` is the note length in
/// 16th ticks. Returns once the window has played out.
#[allow(clippy::too_many_arguments)]
pub fn record_steps(
    ch: u8,
    steps: &[usize],
    note: u8,
    vel: u8,
    bars: usize,
    gate: usize,
    keep_playing: bool,
    pattern: Option<usize>,
) -> Result<String, String> {
    let notes: Vec<(usize, u8, u8)> = steps
        .iter()
        .copied()
        .filter(|s| *s < bars * 16)
        .map(|s| (s, note, vel))
        .collect();
    seq(
        ch,
        notes,
        bars,
        gate,
        keep_playing,
        false,
        0,
        1,
        60,
        1,
        48,
        pattern,
    )
}

/// Fire arbitrary (step, note, velocity) events in ONE clock-synced sweep.
///
/// start mode (default): sends a single Start after the clock input connects
/// and counts ticks from there. Chained `record_steps` calls desync on
/// firmware 2.0 because the unit ignores Start while running while the local
/// tick counter resets; one sweep avoids the restarts.
///
/// follow mode: never touches the transport. Anchors the tick grid on the
/// next drum-channel note_on the unit itself transmits (its loop-start
/// replay, e.g. the kick on step 1), so arming REC and firing stay
/// decoupled from Start/Stop and no record session is killed.
#[allow(clippy::too_many_arguments)]
pub fn seq(
    ch: u8,
    events: Vec<(usize, u8, u8)>,
    bars: usize,
    gate: usize,
    keep_playing: bool,
    follow: bool,
    clock_offset: usize,
    anchor_ch: u8,
    anchor_note: u8,
    anchor_step: usize,
    loop_steps: usize,
    pattern: Option<usize>,
) -> Result<String, String> {
    let total = bars * 16;
    // Follow mode rides the user's running transport; it must never stop it.
    let keep_playing = keep_playing || follow;
    // Follow events are loop-absolute steps; convert to ticks-after-anchor
    // using the unit's loop length so a mid-loop anchor still lands right.
    let mut notes: Vec<(usize, u8, u8)> = if follow {
        let l = loop_steps.max(1);
        events
            .into_iter()
            .map(|(s, n, v)| ((s + l - anchor_step % l) % l, n, v))
            .filter(|(s, _, _)| *s < total)
            .collect()
    } else {
        events.into_iter().filter(|(s, _, _)| *s < total).collect()
    };
    notes.sort();
    if notes.is_empty() {
        return Err("no events inside the bars window".into());
    }

    let mut out = midi::open_out()?;
    // Each track holds 6 pattern slots and the recorder writes whichever one
    // the track currently plays (Pattern Select 30:5p:0F). Select the target
    // slot first so the take lands in it; the selection stays afterwards.
    if let Some(p) = pattern {
        if !(1..=6).contains(&p) {
            return Err("pattern must be 1-6".into());
        }
        let addr = [0x30, 0x50 + ch, 0x0F];
        let msg = crate::sysex::parameter_change(0, addr, &[(p - 1) as u8]).to_bytes();
        out.send(&msg).map_err(|e| e.to_string())?;
    }
    // Start mode: send Start only after the clock input is connected; opening
    // the port after Start loses the first clocks and notes land early.
    let start_out = if follow {
        None
    } else {
        Some(midi::open_out()?)
    };
    let sched = Arc::new(Sched {
        tick: AtomicUsize::new(0),
        sub: AtomicUsize::new(0),
        notes,
        active: Mutex::new(Vec::new()),
    });
    let s2 = sched.clone();
    let fired = Arc::new(AtomicUsize::new(0));
    let f2 = fired.clone();
    let anchored = Arc::new(AtomicBool::new(!follow));
    let a2 = anchored.clone();
    let planned = sched.notes.len();

    fn on_tick(
        s2: &Sched,
        f2: &AtomicUsize,
        out: &mut MidiOutputConnection,
        ch: u8,
        gate: usize,
        total: usize,
        keep_playing: bool,
    ) -> bool {
        let t = s2.tick.fetch_add(1, Ordering::Relaxed);
        if std::env::var_os("ST_DEBUG").is_some() {
            eprintln!("tick {t}");
        }
        let mut active = s2.active.lock().unwrap();
        active.retain(|(c, n, off)| {
            if *off <= t {
                let _ = out.send(&[0x80 | c, *n, 0x40]);
                false
            } else {
                true
            }
        });
        if let Some((_, n, v)) = s2.notes.iter().find(|(s, _, _)| *s == t) {
            let _ = out.send(&[0x90 | ch, *n, *v]);
            active.push((ch, *n, t + gate));
            f2.fetch_add(1, Ordering::Relaxed);
        }
        if t + 1 >= total && active.is_empty() {
            if !keep_playing {
                let _ = out.send(&[0xFC]);
            }
            return false;
        }
        if t + 1 > total * 4 {
            return false; // safety: clock kept running past the window
        }
        true
    }

    midi::run_clock_port_after_connect(
        move |bytes| {
            match bytes.first() {
                Some(0xF8) => {
                    if !a2.load(Ordering::Relaxed) {
                        // follow mode, waiting for the loop anchor: count raw
                        // clocks as a watchdog (8 loops with no anchor = give up)
                        let c = s2.sub.fetch_add(1, Ordering::Relaxed);
                        return c < total * 6 * 8;
                    }
                    // 96 PPQ: 6 clocks per 16th
                    if !s2.sub.fetch_add(1, Ordering::Relaxed).is_multiple_of(6) {
                        return true;
                    }
                    on_tick(&s2, &f2, &mut out, ch, gate, total, keep_playing)
                }
                Some(st)
                    if *st == 0x90 | anchor_ch
                        && !a2.load(Ordering::Relaxed)
                        && (anchor_note == 0 || bytes.get(1) == Some(&anchor_note))
                        && bytes.len() >= 3
                        && bytes[2] != 0 =>
                {
                    // follow anchor: a note the unit itself replays, uniquely
                    // identified by channel+pitch so a mid-loop port open
                    // still locks onto the intended step. clock_offset shifts
                    // the grid by that many clocks into the step: notes fired
                    // exactly ON the boundary get dropped by the recorder.
                    a2.store(true, Ordering::Relaxed);
                    s2.tick.store(0, Ordering::Relaxed);
                    if clock_offset == 0 {
                        s2.sub.store(1, Ordering::Relaxed); // next tick boundary in 5 clocks
                        on_tick(&s2, &f2, &mut out, ch, gate, total, keep_playing)
                    } else {
                        s2.sub.store(clock_offset, Ordering::Relaxed);
                        true
                    }
                }
                Some(0xFC) => {
                    s2.tick.store(0, Ordering::Relaxed);
                    s2.sub.store(0, Ordering::Relaxed);
                    if follow {
                        a2.store(false, Ordering::Relaxed); // re-anchor on next loop
                    }
                    true
                }
                _ => true,
            }
        },
        Some(Duration::from_secs(5)),
        move || {
            if let Some(mut conn) = start_out {
                // restart from step 0 so step 1 lands on tick 0
                let _ = conn.send(&[0xFA]);
            }
        },
    )?;
    let n = fired.load(Ordering::Relaxed);
    if follow && !anchored.load(Ordering::Relaxed) {
        return Err("no loop anchor seen (drum note_on); is the transport playing?".into());
    }
    Ok(format!(
        "fired {n}/{planned} notes over {total} steps ({}); the unit records them while its REC is armed",
        if follow {
            "followed the running transport"
        } else if keep_playing {
            "transport left running"
        } else {
            "transport stopped"
        }
    ))
}
