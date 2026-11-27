# Seam links (`seams::SeamLink`)

A seam link says which holes of two stitch lines are sewn together. The user picks two seams (or the same seam twice, for a folded flap), and the editor pairs their holes: it shows which hole meets which, how many are left without a pair, and how far the two seams drift apart along their length.

A link is part of the document: it lives in `Project.seam_links` and is saved in `.leathercraft`. Like the holes (`docs/stitch-holes.md`), the pairs are never stored. A link keeps only its parameters (the hole ranges, the direction, the offset and the pairs set by hand), and `SeamLink::pairs` computes the pairs from the current lines, so they follow any edit of a seam. The rules live in the library crate and are tested without Bevy; the dialog and the drawing are in the binary.

The dialog follows the "Stitch Link Dialog — Зв'язати шви" frame (`0f4a53ae18e2d8f3`) of the Brilliant design. Its body (schematic, table) fills the space the cards above and the warning and footer below leave, so it stretches with the dialog; the footer's height is measured on the previous frame. egui's default fonts have no `→` or `↔`, so the texts use `·` and `:` where the design has arrows.

| File | Contents |
|---|---|
| `src/seams.rs` | `SeamSide`, `ManualPair`, `SeamLink`, `HolePair`, `LinkPairs`; `SeamLink::pairs`, `lines`, `uses_child`, `uses_piece`, `ranges_overlap`, `validate`; `Project::seam_link`, `next_seam_link_id`, `seam_links_of`, `validate_seam_links`; `stitch_line`, `stitch_lines`, `stitch_number`; tests and their fixtures (`two_lines`, `side`, `link`), shared with the other modules' tests |
| `src/shapes/stitch.rs` | `StitchLine::hole_offsets` (each hole's distance along the path) and `pitch_for_count` (the pitch that gives a hole count) |
| `src/sewing.rs` | `StitchMethod`, `Assembly`, `OnTop`, `Sewing` (`thread_mm`, `validate`) and their limits, with tests |
| `src/model.rs` | `Project.seam_links`; `validate` ends with `validate_seam_links` |
| `src/events.rs` | `AddSeamLink`, `SetSeamLink`, `RemoveSeamLink`, `ProjectChange::SeamLinkAdded`; the cascade in `RemoveChild`, `RemoveChildren`, `RemovePiece` and `RemovePieces` |
| `src/history.rs` | `describe` arms; `MergeKey::SeamLink` |
| `src/i18n/` | The labels and messages under [Text](#text) |
| `src/panels/seam_link/mod.rs` | `SeamLinkDraft`, `OpenLink`, `SeamLinkResult`, `seam_name`, `seam_link_dialog`: the header, seam cards, start offset, warning with the Fit pitch buttons, summary and footer |
| `src/panels/seam_link/sewing.rs` | `sewing_cards`: the stitch and assembly tiles with their drawings, Lock, Overlap and On top; the thread card under them (type dropdown, thickness, shade swatches, a color picker, the hex field) with the thread estimate |
| `src/panels/seam_link/schematic.rs` | `schematic`: the two seams unrolled into rows of holes, the pair lines, the clicks that set manual pairs |
| `src/panels/seam_link/table.rs` | `pair_table`: the pairs, then the unpaired holes collapsed into ranges |
| `src/panels/properties/stitch.rs` | `seam_links_section`: the selected seam's links, with Edit and Remove, and Link seams… |
| `src/editor/ui.rs` | `pattern_menu` (Pattern → Link seams…), the dialog's results in `editor_ui`, the dialog in `dialog_open` |
| `src/editor/editor_state_struct.rs` | `seam_link_dialog`, `open_seam_link`, `selected_seam`; a new or opened project closes the dialog |
| `src/editor/canvas.rs` | `draw_seam_links`: the pair lines while a linked seam is selected |
| `src/ui_components/palette/seam_link.rs` | `SeamLinkPaint` |
| `src/ui_components/icons/mod.rs` | Painted `paint_link`, `paint_reverse`, `paint_warning`, `paint_close`, `paint_reset` |

## For users

| Action | How |
|---|---|
| Link two seams | **Pattern → Link seams…**, or **Link seams…** under the selected stitch line in Properties. The selected seam starts as both A and B; pick the other seam for B |
| Choose the holes that take part | **Holes from** and **to** on each seam's card, numbered from 1 |
| Sew one seam in the other direction | **Reverse** on its card |
| Shift one seam against the other | **Start offset**, in holes: a positive offset skips the first holes of B, a negative one the first holes of A |
| Pair two holes by hand | Click a hole of A on the schematic, then a hole of B. Click the A hole again, or press Esc, to drop it |
| Remove a manual pair | Alt+click one of its holes |
| Drop every manual pair | **Reset manual pairs** |
| Make the counts match | The warning's **Pitch A → …** or **Pitch B → …** button sets that seam's pitch. It is an ordinary edit, with its own History entry; the dialog stays open and shows the new pairs |
| Choose the stitch | A tile on the **Stitch** card: Saddle, Running, Baseball, Whip or Cross. **Lock** sets how many holes are sewn back over at each end; the card shows the thread it takes |
| Choose how the pieces lie | A tile on the **Assembly** card: Flesh sides, Lapped, Butted, Turned or Folded. **Overlap** is on while Lapped; **On top** while the pieces stack (Flesh sides, Lapped, Turned) |
| Save the link | **Link N pairs**, or **Save** when changing a link |
| Resize the dialog | Drag any of its edges or corners; it stays centered and grows on both sides. The size is kept until the app closes. It opens at the design's 1000 × 812 and never gets smaller than 860 × 712 or larger than the window |
| See, change or remove links | Select a linked stitch line: Properties lists its links, each with the other seam and its pair count, **Edit** and **Remove**. The canvas draws the pair lines |
| Fold a flap | Pick the same seam for A and B with ranges that don't share holes, usually with **Reverse** on B |

- Picking another seam on a card takes all of its holes and drops the manual pairs, whose hole numbers belonged to the old seam.
- On one seam, ranges that share holes are an error: it shows under the cards and the commit button is off.
- The schematic places each hole at its distance along its seam, lined up at the first automatic pair, so a seam that creeps shows as slanting lines. When holes would be under 24 px apart it shows every 2nd, 5th, 10th… hole, as in the design, plus both ends, the manual pairs and the unpaired holes that have room. The rows' gap grows with the dialog's height.
- The table's **Δ** column is how far B's hole is along its range beyond A's hole along its own: how much the seam has crept by that pair. The summary shows the pairs, the unpaired holes, and the difference between the lengths of the two ranges.
- Deleting a seam or its piece deletes its links in the same edit, so one Undo brings both back.
- A link whose range runs past its seam's holes, after the seam got shorter, still opens; its range is cut short and Properties shows it in the warning color.

## The model

```rust
pub struct SeamSide {
    pub piece: u64,
    pub child: u64,      // a ChildShape whose shape is Shape::StitchLine
    pub from: usize,     // 0-based hole index, inclusive
    pub to: usize,       // inclusive; cut to the line's holes when the pairs are computed
    #[serde(default)]
    pub reversed: bool,
}

pub struct ManualPair { pub a: usize, pub b: usize }   // hole indices on A and B

pub struct SeamLink {
    pub id: u64,
    pub a: SeamSide,
    pub b: SeamSide,
    #[serde(default)]
    pub offset: i64,     // B's sequence shifted against A's, in holes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub manual: Vec<ManualPair>,
}
```

```rust
pub struct Sewing {                // #[serde(default)]: a missing field takes its default
    pub stitch: StitchMethod,      // Saddle
    pub lock_holes: u32,           // 2, at most MAX_LOCK_HOLES (10)
    pub assembly: Assembly,        // FleshToFlesh
    pub overlap_mm: f64,           // 8, from MIN_OVERLAP_MM (1) to MAX_OVERLAP_MM (100); used while Lapped
    pub on_top: OnTop,             // A; used while the assembly stacks
    pub thread: Thread,            // also #[serde(default)]
}

pub struct Thread {
    pub kind: ThreadKind,          // WaxedPolyester; WaxedLinen, BondedNylon, ArtificialSinew
    pub diameter_mm: f64,          // 0.55, from MIN_THREAD_MM (0.2) to MAX_THREAD_MM (1.5)
    pub color: ThreadColor,        // "#E8DCC0" (ThreadShade::Natural); any sRGB color
}
```

`ThreadColor` is saved as a `"#RRGGBB"` string. The dialog offers the eight `ThreadShade`s as swatches and any other color through a picker or the hex field; the sewn 3D preview draws each link's thread in its color.

`SeamLink.sewing` is `#[serde(default, skip_serializing_if = "Sewing::is_default")]`, so a link sewn the default way saves as before and older links load as a saddle stitch on pieces laid flesh to flesh.

`Project.seam_links` is `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so a project without links saves without the key and older files load unchanged; `PROJECT_VERSION` stays 2. Link ids are unique within the project and given by `AddSeamLink`, as `max + 1`.

## Pairing

`SeamLink::pairs(&Project) -> Result<LinkPairs, Message>`:

1. Find both stitch lines (`SeamNotFound` otherwise) and take their `hole_offsets`. Cut each range to the line's holes; if it ran past them, `clamped` is set.
2. Make each side's sequence of hole indices, turned around when `reversed`. Its distances are measured from where the sequence starts.
3. Skip the first `offset` holes of B (or `-offset` of A) and pair the two sequences in step.
4. Lay each manual pair over them: first remove the automatic pairs that use either of its holes, then add it. A manual pair outside a range is ignored and sets `clamped`.
5. Sort the pairs in the order of A's sequence. Holes of the ranges in no pair are `free_a` and `free_b`. Each pair's `delta_mm` is B's distance minus A's.

`LinkPairs` also returns the sequences and per-hole distances, which the schematic uses to place the holes, and the length of each range.

### Fitting the pitch

`StitchLine::pitch_for_count(count)` returns a pitch, rounded to 0.01 mm, at which the line has exactly `count` holes:

- With one run the pitch is the length over `count - 1`.
- With several runs, the pitches that give `count` holes form one interval, since the hole count never grows as the pitch grows. Its ends are found by bisection between `MIN_PITCH_MM` and `MAX_PITCH_MM`. The candidates are the length over `count - 1` (when it already works), that value clamped into the interval, and the interval's middle.
- A candidate must pass `has_valid_parameters` and not make the holes overlap; with none left, `None`.

The dialog asks for the line's count plus the difference between the two ranges, so the holes outside the range keep their number, and moves the range's end by the same difference.

## Sewing

The sewing doesn't change the pairs: it records how the seam is made, for the maker and for the sewn 3D preview, which lays the pieces together by the assembly and leaves the stitch's slack in the seam (`docs/sewn-preview.md`). The overlap is kept while another assembly is chosen, so switching back restores it.

`Sewing::thread_mm(seam, pitch)` estimates the thread: the seam plus `lock_holes` pitches at each end, times the stitch's factor. The factors are rules of thumb, tails included: saddle 4, running 2, baseball 5, whip 3.5, cross 6. The dialog measures the seam along A's range at A's pitch and shows the estimate in the project's units.

The tiles' drawings are painted, on the design's 64 × 24 grid: the stitches from the grain side, the assemblies in cross-section with A lighter than B. They shrink on narrow tiles.

## Validation

`Project::validate` calls `validate_seam_links`, so `Project::apply` rejects a bad link, and saving or loading checks it:

| Rule | Error |
|---|---|
| Both sides' piece and child exist, and the child is a stitch line | `SeamNotFound` |
| `from <= to` on each side | `InvalidSeamRange` |
| On one seam, the ranges don't share holes | `SeamRangesOverlap` |
| No hole is in two manual pairs on the same side | `DuplicateManualPair` |
| Link ids are unique and not 0 | `DuplicateSeamLink` |
| At most `MAX_LOCK_HOLES` locked holes | `LockHolesOutOfRange` |
| The overlap is within `MIN_OVERLAP_MM..=MAX_OVERLAP_MM` | `OverlapOutOfRange` |

A range past the line's holes is **not** an error: otherwise an edit that shortens a seam would be rejected. It is cut when the pairs are computed and reported as `clamped`, the way `ResultEmpty` leaves a valid project flagged in the editor.

## Events and history

| Event | Change | History entry |
|---|---|---|
| `AddSeamLink { link }` (its id is ignored) | `SeamLinkAdded(id)` | Linked seams of … |
| `SetSeamLink { link }` | `Edited`; consecutive changes of one link merge (`MergeKey::SeamLink`) | Changed the seam link of … |
| `RemoveSeamLink { id }` | `Edited` | Removed the seam link of … |

`RemoveChild`, `RemoveChildren`, `RemovePiece` and `RemovePieces` remove the links that use what they remove, in the same edit. Moving or reshaping a seam leaves its links alone, as the pairs follow it.

## Text

| Kind | Entries |
|---|---|
| `Label` | `LinkSeams`, `LinkSeamsTitle`, `LinkSeamsSubtitle`, `SeamA`, `SeamB`, `HolesFrom`, `HolesTo`, `Reverse`, `StartOffset`, `HolesShort`, `LinkSchematic`, `HolePairs`, `PairAuto`, `PairManual`, `PairManualLegend`, `PairFree`, `LinkHint`, `ResetManualPairs`, `Pairs`, `LengthDifference`, `SeamLinks`, `EditLink`, `RemoveLink`, `SeamLinkStale`, `StitchMethodTitle`, `StitchMethod`, `StitchMethodNote`, `StitchMethodHint`, `AssemblyTitle`, `Assembly`, `AssemblyNote`, `LockHoles`, `Overlap`, `OnTop`, `ThreadTitle`, `ThreadKindCaption`, `ThreadKind`, `ThreadDiameter`, `ThreadColorCaption`, `ThreadShade`, `ThreadCustomColor` |
| `Message` | Errors: `SeamNotFound`, `InvalidSeamRange`, `SeamRangesOverlap`, `DuplicateManualPair`, `DuplicateSeamLink`, `LockHolesOutOfRange`, `OverlapOutOfRange`, `ThreadDiameterOutOfRange`. Dialog: `AssemblyHint`, `ThreadEstimate`, `HoleCountMismatch`, `UnpairedHoles`, `FitPitch` (its pitch is formatted by the caller, since `Message` is `Eq`), `LinkPairsButton`, `SeamName`, `SeamOption`, `HoleCountBadge`, `PairsCount`, `FreeIn`, `SeamLinkRow`. History: `ActionLinkSeams`, `ActionEditSeamLink`, `ActionRemoveSeamLink` |

Ukrainian counts use the catalog's `plural` helper.

## Colors and icons

`SeamLinkPaint` holds the dialog's own colors: the automatic, manual and unpaired pair colors, hole fills, the guides and arrows, the A/B badge, the warning box, and the table's row and badge colors. Dark values come from the design; it has no light board, so the light ones are picked to match. Cards, fields, the table and the tiles use the shared `surface`, `border`, `text` and `accent` groups.

The link, reverse, warning, close and reset icons are painted (`docs/icons.md`), not font glyphs: adding glyphs needs the icon font project, which is edited in its own app. They can move to the font later without changing their callers' look.

## Tests

`cargo test --lib --no-default-features`:

| Module | Covers |
|---|---|
| `shapes::stitch` | `hole_offsets` grow and end at the length; `pitch_for_count` hits the count on one run and on a bent line, and refuses unreachable counts |
| `sewing` | Missing fields take their defaults; the thread color saved as hex and read leniently; the thread estimate; the limits; which assemblies stack or overlap |
| `seams` | Equal counts pair every hole with Δ 0; reverse; positive and negative offset; a manual pair replacing one or two automatic pairs; 48 against 52 holes; creep in Δ; clamping after a seam got shorter and for a manual pair outside a range; one seam linked to itself; a missing seam; listing and numbering seams |
| `model` | Each validation rule, and that a range past the holes is accepted |
| `events` | Add, set and remove, with their errors; a rejected link, including one with an overlap out of range, leaves the project unchanged; each removal cascades and Undo restores the link; moving a seam keeps its links |
| `history` | The entries' text; consecutive `SetSeamLink` merge; Undo restores the merged state |
| `persistence` | A link round-trips, without its sewing while it is the default and with it otherwise; a project without links saves without the key and loads |

`panels::seam_link::table` has a binary test for the hole ranges (`cargo test --bin leathercraft`). The dialog, the schematic and the canvas lines are checked by hand.
