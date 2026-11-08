# Jump reset

When you take a hit, jumping at the right moment cuts the knockback. The trainer tells you whether
you jumped too early or too late, and how much of the hit's shove was left a tick later (a
successful reset leaves almost none).

> **Status: timing is solid, the "Reset!" verdict is not calibrated yet.**
> The millisecond timing is measured from real events and is accurate to about one frame. The
> knock-left threshold that decides `Reset!` (0.08 blocks per tick) was set from **one** recorded
> jump reset, not from a data set. Treat the verdict as a good guide rather than a measurement,
> and see [Help calibrate it](#help-calibrate-it) if you want to improve it.

Needs the **Controls** (your jump key), **Player status** (hurt events and your velocity) and
**HUD** permissions.

## What it shows

A line under the crosshair says `Too early: 85 ms`, `Too late: 120 ms`, or
`Perfect (12 ms early)` when your jump landed within the perfect window of the hit. It is yellow
when you were close (within twice the window), red when you were far off and green when perfect.
It goes away after a moment, and your next hit starts it again.

- **The hit** is the host's `hurt` event for you: the instant the game shows you being hurt.
- **Your jump** is the press of your own jump key, read from your key bindings (Space by default).
- Both are timed to the frame, so the milliseconds are accurate to about one frame. Your network
  delay is not taken off the timing: it shows what you feel on your screen.
- A jump and a hit more than 0.4 seconds apart are not about each other, so ordinary jumping shows
  nothing. Only your latest jump before a hit, or your first jump after it, counts, and it judges
  the key press, not holding the key down.

## Knock left

Every hit sets your horizontal speed to a fixed shove (0.4 blocks per tick, and 0.4 up), so the
shove's size says nothing. What differs is how much of it is left one tick (50 ms) later.

After each hit the mod records your velocity every 50 ms for 0.4 seconds, finds the slot where the
shove shows (horizontal speed of 0.3 or more) and reads your horizontal speed in the next one.
That is shown after the timing as `Knock left 0.15`.

- **`Reset! knock left 0.04`** (green): it stayed under the threshold (0.08 blocks per tick by
  default) **and** you pressed jump within twice the perfect window of the hit (150 ms by
  default).
- **`(no jump)` / `(jump off)`**: a small knock with no jump near it. A wall or a corner kills the
  shove too, so this is not called a reset.
- **`(in air)`**: a hit taken in the air, because a jump cannot reset it (you need to be on the
  ground when the hit lands).
- **Nothing shown**: a hurt that never shows the shove (fire, lava, poison, drowning, anything
  that hurts without pushing you) is not a melee hit. A hurt within 0.4 seconds of a fall faster
  than 0.55 blocks per tick (a fall that hurts, three blocks or more) is fall damage and is
  ignored.

What was measured so far: a normal hit on the ground leaves about 0.13 to 0.22 (ground friction
eats the rest) and one in the air about 0.29 to 0.34. In the one jump reset measured (a jump
pressed at 0 ms) 0.044 was left.

## The page

Open **Jump reset** from the start page. Switch it on or off (here or on the start page) and set
the **Perfect window** (20 to 200 ms in steps of 5, 75 by default; click the value to go back to
the default). The page shows a sample verdict while it is open.

The threshold (`jump_reset_carry`, 0.01 to 0.4) is file-only; see [settings.md](settings.md).

## Help calibrate it

The mod keeps the data needed to tune the threshold in the settings file
(`block-outline.settings.json`):

- **`jump_reset_log`**: the last 16 hits, one `offset,carried,grounded,speed` per hit, oldest
  first. `offset` is the jump's offset from the hit in ms with a minus sign when early and `x` for
  no jump; `carried` is the knock left a tick later in thousandths of a block per tick; `speed` is
  your horizontal speed just before the hit in the same unit; `grounded` is 1 or 0 for on the
  ground, or `x` when unknown.
- **`jump_reset_trace`** (write-only, not read back): for each of those hits, your velocity just
  before it and every 50 ms for 0.4 seconds after (`x,y,z` and the on-the-ground flag in
  thousandths of a block per tick), with `/` between hits.
- **`jump_reset_resets`**: every hit that counted as a jump reset, kept in full in the same form
  (the last 24, newest last). It is read back when the mod starts, so resets survive restarts and
  are not pushed out by the ordinary hits after them.

The settings file holds 16 KiB, so if everything does not fit the oldest rows are left out, the
ordinary hits' recordings before the resets.

### Let the tool do the comparison

```powershell
python scripts\jump-reset-report.py
```

It reads the installed settings file, keeps the hits you took on the ground, separates those with a
jump inside the verdict's window from the rest, and recommends a `jump_reset_carry` only when the
knock left after your jumps falls into two clearly separate groups (resets, and jumps that missed).
With too little data (it wants at least 10 hits on the ground and 4 with a jump close to the hit) or
no clean split, it says so instead of inventing a number. Set the value it suggests in the settings
file; it takes effect without a rebuild.
