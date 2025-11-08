# TUI Widget Migration Plan

**Goal:** Migrate from string-based rendering to composable widget-based architecture

**Strategy:** Incremental refactoring with backward compatibility at each step

**Process:** After each step:
1. ✅ Manual testing in terminal
2. ✅ Unit tests (90%+ coverage)
3. ✅ User approval before proceeding

---

## Phase 1: Foundation (Infrastructure)

### Step 1.1: Create Widget Infrastructure
**Estimated time:** 2-3 hours
**Files created:**
- `src/tui/widgets/mod.rs` - Core trait and utilities
- `src/tui/widgets/testing.rs` - Test helpers

**Changes:**
- Add `RenderableWidget` trait
- Add buffer testing utilities
- Add snapshot testing setup

**Testing approach:**
- Unit tests for test utilities
- Example widget to validate infrastructure

**Manual test:** Not user-facing, but compile check confirms no breakage

**Status:** ⏸️ Not started

---

### Step 1.2: Create Buffer-Based Rendering Utilities
**Estimated time:** 1-2 hours
**Files created:**
- `src/tui/widgets/primitives.rs` - Low-level rendering functions

**Changes:**
- Buffer-based box drawing
- Buffer-based text rendering
- Border rendering utilities

**Testing approach:**
- Unit tests for each primitive
- Snapshot tests for border patterns

**Manual test:** Not user-facing yet

**Status:** ⏸️ Not started

---

## Phase 2: First Real Widget (Proof of Concept)

### Step 2.1: Extract ScoringTable Widget
**Estimated time:** 4-5 hours
**Files created:**
- `src/tui/widgets/scoring_table.rs`

**Changes:**
- Create `ScoringTable` widget struct
- Implement `RenderableWidget` for `ScoringTable`
- Keep existing `format_scoring_summary()` as fallback
- Add feature flag to switch between implementations

**Testing approach:**
- Unit tests for widget rendering
- Snapshot tests comparing old vs new output
- Test all edge cases (no goals, single goal, multiple goals)
- Test border types (top, middle, bottom)
- Test UTF-8 handling

**Manual test:**
- View boxscore with scoring summary
- Toggle between old/new with environment variable
- Verify identical output

**Acceptance criteria:**
- ✅ Pixel-perfect match with existing output
- ✅ No performance regression
- ✅ 90%+ code coverage

**Status:** ⏸️ Not started

---

### Step 2.2: Integrate ScoringTable into Boxscore View
**Estimated time:** 2-3 hours
**Files modified:**
- `src/tui/scores/view.rs` - Use widget in boxscore rendering

**Changes:**
- Replace string-based scoring summary with widget
- Remove old implementation after validation
- Update boxscore rendering to use widget

**Testing approach:**
- Integration tests for boxscore rendering
- Test with real API data
- Test scrolling behavior

**Manual test:**
- Open boxscore for game with goals
- Scroll through scoring summary
- Verify styling matches theme

**Acceptance criteria:**
- ✅ Boxscore displays correctly
- ✅ Scrolling works smoothly
- ✅ Theme colors applied correctly

**Status:** ⏸️ Not started

---

## Phase 3: Score Table Components

### Step 3.1: Create ScoreTable Widget
**Estimated time:** 5-6 hours
**Files created:**
- `src/tui/widgets/score_table.rs`

**Changes:**
- Extract score table logic from `scores_format.rs`
- Create `ScoreTable` widget
- Support period-by-period display
- Support OT/SO columns
- Support in-progress highlighting

**Testing approach:**
- Unit tests for all game states (scheduled, live, final)
- Test period combinations (3 periods, OT, SO)
- Test column width calculations
- Snapshot tests for each state

**Manual test:**
- View scores tab
- Check scheduled games
- Check live games (if available)
- Check final games
- Verify OT/SO columns appear when needed

**Acceptance criteria:**
- ✅ All game states render correctly
- ✅ Column widths adapt to content
- ✅ Current period highlighting works

**Status:** ⏸️ Not started

---

### Step 3.2: Create GameBox Widget
**Estimated time:** 6-8 hours
**Files created:**
- `src/tui/widgets/game_box.rs`

**Changes:**
- Create `GameBox` widget composing:
  - Header (game status/time)
  - ScoreTable
  - Team names
- Support selection styling
- Fixed dimensions (37×7)

**Testing approach:**
- Unit tests for each game state
- Test selection highlighting
- Test with/without scores
- Test header variations
- Snapshot tests for visual regression

**Manual test:**
- View scores tab
- Navigate game grid
- Select different games
- Verify selection highlighting
- Check all game states

**Acceptance criteria:**
- ✅ Game boxes render identically to current
- ✅ Selection highlighting works
- ✅ All game states covered

**Status:** ⏸️ Not started

---

### Step 3.3: Create GameGrid Widget
**Estimated time:** 8-10 hours
**Files created:**
- `src/tui/widgets/game_grid.rs`

**Changes:**
- Create `GameGrid` widget using Layout system
- Dynamic column calculation (1-3 columns)
- Compose multiple GameBox widgets
- Replace manual position calculations
- Support scrolling
- Support selection

**Testing approach:**
- Unit tests for column calculations
- Test 1, 2, 3 column layouts
- Test grid with 0, 1, 5, 13 games
- Test selection wrapping
- Test scrolling behavior

**Manual test:**
- Resize terminal (small, medium, large)
- Verify column count changes
- Navigate entire grid
- Test with real schedule data
- Verify scrolling

**Acceptance criteria:**
- ✅ Layout matches current behavior
- ✅ All column counts work
- ✅ Selection navigation correct
- ✅ Scrolling smooth

**Status:** ⏸️ Not started

---

### Step 3.4: Replace Scores Tab Rendering
**Estimated time:** 4-5 hours
**Files modified:**
- `src/tui/scores/view.rs` - Use GameGrid widget
- `src/commands/scores_format.rs` - Mark as deprecated or remove

**Changes:**
- Replace `format_scores_for_tui_with_width()` with GameGrid
- Remove string-based styling logic
- Remove `apply_box_styling_ratatui()`
- Simplify `render_content()` significantly

**Testing approach:**
- Integration tests for entire scores tab
- Test with various schedule sizes
- Test selection and navigation
- Performance testing

**Manual test:**
- Full scores tab testing
- Multiple dates
- Different terminal sizes
- Navigate and select games
- Open boxscore
- Verify performance

**Acceptance criteria:**
- ✅ Scores tab identical to before
- ✅ All interactions work
- ✅ Performance equal or better
- ✅ Code significantly simpler

**Status:** ⏸️ Not started

---

## Phase 4: Standings Components

### Step 4.1: Create TeamRow Widget
**Estimated time:** 4-5 hours
**Files created:**
- `src/tui/widgets/team_row.rs`

**Changes:**
- Extract team row rendering
- Support selection highlighting
- Support divider lines between divisions

**Testing approach:**
- Unit tests for team row formatting
- Test column alignment
- Test selection styling
- Snapshot tests

**Manual test:**
- View standings tab
- Check team rows render correctly
- Verify alignment

**Acceptance criteria:**
- ✅ Team rows render correctly
- ✅ Columns aligned properly
- ✅ Selection works

**Status:** ⏸️ Not started

---

### Step 4.2: Create StandingsTable Widget
**Estimated time:** 6-8 hours
**Files created:**
- `src/tui/widgets/standings_table.rs`

**Changes:**
- Create table widget for standings
- Support Division/Conference/League views
- Support team selection
- Use Layout for columns

**Testing approach:**
- Unit tests for each view type
- Test 1 and 2 column layouts
- Test team selection
- Test scrolling

**Manual test:**
- View standings in all modes
- Navigate teams
- Verify layout

**Acceptance criteria:**
- ✅ All views render correctly
- ✅ Team selection works
- ✅ Scrolling smooth

**Status:** ⏸️ Not started

---

### Step 4.3: Replace Standings Tab Rendering
**Estimated time:** 5-6 hours
**Files modified:**
- `src/tui/standings/view.rs` - Use new widgets

**Changes:**
- Replace string-based rendering
- Use StandingsTable widget
- Simplify layout code

**Testing approach:**
- Integration tests
- Test all view modes
- Test team selection and navigation

**Manual test:**
- Full standings tab testing
- All view modes
- Team selection
- Panel navigation

**Acceptance criteria:**
- ✅ Standings identical to before
- ✅ All interactions work
- ✅ Code simpler

**Status:** ⏸️ Not started

---

## Phase 5: Polish and Optimization

### Step 5.1: Performance Optimization
**Estimated time:** 3-4 hours
**Files modified:** Various

**Changes:**
- Profile rendering performance
- Optimize hot paths
- Add render caching where beneficial

**Testing approach:**
- Performance benchmarks
- Compare before/after metrics

**Manual test:**
- Test responsiveness
- Rapid tab switching
- Large data sets

**Acceptance criteria:**
- ✅ No performance regressions
- ✅ Smooth at 60 FPS

**Status:** ⏸️ Not started

---

### Step 5.2: Code Cleanup
**Estimated time:** 2-3 hours
**Files modified:** Various

**Changes:**
- Remove dead code
- Remove old string-based functions
- Update documentation
- Clean up imports

**Testing approach:**
- Full test suite passes
- No clippy warnings

**Manual test:**
- Full manual testing pass
- All features work

**Acceptance criteria:**
- ✅ No deprecated code remains
- ✅ Documentation updated
- ✅ Clean clippy

**Status:** ⏸️ Not started

---

## Progress Tracking

**Completed Steps:** 0 / 15
**Current Phase:** Phase 1
**Current Step:** 1.1
**Overall Progress:** 0%

---

## Notes for Each Step

After each step is coded:

1. **Run the code:**
   ```bash
   cargo build
   cargo run
   ```

2. **Manual testing checklist:**
   - [ ] Feature works as before
   - [ ] No visual regressions
   - [ ] No performance issues
   - [ ] No crashes or panics

3. **After approval, run tests:**
   ```bash
   cargo test
   cargo clippy
   ```

4. **Verify coverage:**
   ```bash
   cargo tarpaulin --out Html
   # Check coverage report for new code
   ```

5. **Mark step complete and proceed to next**

---

## Rollback Strategy

Each step maintains backward compatibility:
- Old code kept until new code validated
- Feature flags for switching implementations
- Can revert any step by removing new code

---

## Risk Mitigation

**Risk:** Breaking existing functionality
**Mitigation:** Incremental changes, extensive testing, feature flags

**Risk:** Performance regression
**Mitigation:** Benchmark each step, optimize before proceeding

**Risk:** Test coverage gaps
**Mitigation:** 90% coverage requirement, snapshot tests

**Risk:** Visual regressions
**Mitigation:** Snapshot tests, manual testing, side-by-side comparison

---

## Success Metrics

**Code Quality:**
- [ ] Test coverage > 90% for all new code
- [ ] No clippy warnings
- [ ] All tests passing

**Functionality:**
- [ ] All features work identically
- [ ] No visual regressions
- [ ] No performance regressions

**Maintainability:**
- [ ] Code significantly simpler
- [ ] Components reusable
- [ ] Easy to test new features

---

## Context Restoration

To restore context for any step:

1. Read this file
2. Check "Current Step" above
3. Review the step's:
   - Estimated time
   - Files to create/modify
   - Changes description
   - Testing approach
   - Acceptance criteria

---

**Last Updated:** 2025-01-07
**Current Status:** Planning complete, ready to begin Step 1.1
