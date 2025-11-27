# React-Like Component System Refactoring Plan

**Created**: 2025-11-26
**Last Updated**: 2025-11-27

## Executive Summary

This document outlines a phased approach to refactor the NHL TUI app to fully use its React-like component system. The app has a well-designed Component trait with Props, State, Message, and lifecycle methods. We've successfully migrated most UI state from global `AppState` to component-owned state.

## Current State (2025-11-27)

### Completed Phases

✅ **Phase 1**: Runtime Foundation - ComponentStateStore implemented
✅ **Phase 2**: Message Dispatch - ComponentMessage infrastructure
✅ **Phase 3**: ScoresTab POC - Component with State and Messages
✅ **Phase 3.5**: ScoresTab Integration - Full integration with Runtime
✅ **Phase 7**: Generic Document Navigation - Completed!

### Architecture Status

**Component State (✅ Complete)**:
- `ScoresTab`: Full component with `ScoresTabState` managing all UI state
- `StandingsTab`: Full component with `StandingsTabState` managing all UI state
- `DemoTab`: Full component using `DocumentNavState` directly as state
- All components handle their own messages and state updates
- Component states persist across renders

**Document Navigation (✅ Generic)**:
- Created `src/tui/document_nav.rs` - Generic document navigation module
- `DocumentNavState` struct - Embedded in components that need document behavior
- `DocumentNavMsg` enum - Shared navigation messages (FocusNext, FocusPrev, etc.)
- `handle_message()` function - Generic navigation logic reused by all components
- No code duplication between DemoTab and StandingsTab

**Key Routing (✅ Working)**:
- `key_to_action()` reads from component states via helper functions
- `is_box_selection_active()`, `is_browse_mode_active()` read from component state
- Document navigation dispatches `ComponentMessage` with `DocNav(DocumentNavMsg)`
- All navigation handled by components, not global reducers

**Global State (✅ Cleaned)**:
- `ScoresUiState`: Kept as empty struct (removed in reducers, not updated)
- `StandingsUiState`: Kept as empty struct (removed in reducers, not updated)
- `state.ui.demo`: Still has `DocumentState` but NOT used (component state is source of truth)
- `state.ui.standings_doc`: Removed (was never actually in UiState, was in document reducer only)

**Reducers (✅ Simplified)**:
- `reducers/scores.rs`: Simple message forwarder to ScoresTab component
- `reducers/standings.rs`: Simple message forwarder to StandingsTab component
- `reducers/document.rs`: Only handles `UpdateViewportHeight` (which is currently not dispatched)
- `reducers/data_loading.rs`: Updates focusable metadata in component state on data load

### Phase 7 Implementation Details

**Created**:
- `src/tui/document_nav.rs` - Generic document navigation module (230 lines)
  - `DocumentNavState` - Embeddable state struct with focus_index, scroll_offset, viewport_height, focusable metadata
  - `DocumentNavMsg` - Shared message enum (FocusNext/Prev, FocusLeft/Right, Scroll*, Page*, etc.)
  - `handle_message()` - Generic handler for all navigation messages
  - Helper functions: `focus_next()`, `focus_prev()`, `autoscroll_to_focus()`, `find_row_sibling()`, etc.

**Updated**:
- `src/tui/components/demo_tab.rs`:
  - Removed `DemoTabState` type (now uses `DocumentNavState` directly)
  - Simplified messages to just `DocNav(DocumentNavMsg)` and `UpdateViewportHeight`
  - Implemented `ComponentMessageTrait`
- `src/tui/components/standings_tab.rs`:
  - Embedded `DocumentNavState` in `StandingsTabState`
  - Added `DocNav(DocumentNavMsg)` and `UpdateViewportHeight` message variants
  - Removed `focus_index` and `scroll_offset` from props (now in component state)
- `src/tui/keys.rs`:
  - `handle_demo_tab_keys()` and `handle_standings_league_keys()` now handle Up/Down arrows
  - Removed duplicate Up/Down handling from main `key_to_action()`
  - All document navigation dispatches `ComponentMessage` instead of `DocumentAction`
  - Removed unused `DocumentAction` import
- `src/tui/mod.rs`:
  - Removed `UpdateViewportHeight` dispatch from render loop (was causing infinite loop)
  - Viewport height comes from `area.height` at render time, no need to store in state

**Fixed**:
- Infinite loop caused by `UpdateViewportHeight` returning `Effect::Batch` with new actions
- Browse mode navigation now works correctly
- All tests pass (656 passed, 0 failed)

---

## Success Criteria for Phase 7 ✅

- ✅ Browse mode works in standings (can navigate with arrows)
- ✅ Focus wraps around correctly
- ✅ Autoscroll keeps focused element visible
- ✅ Left/Right navigation works between columns (Row elements)
- ✅ Generic document navigation (no hardcoded component checks)
- ✅ Document navigation logic lives in components, not global reducers
- ✅ All tests pass (656 passed)
- ✅ No compilation errors or warnings
- ✅ No infinite loops

---

## Remaining Work: Final Cleanup

The system is functionally complete, but there are cleanup tasks remaining:

### Phase 8: Remove Deprecated Global State

**Goal**: Clean up unused global state fields that are no longer the source of truth.

**Files to Update**:
1. `src/tui/state.rs`:
   - Remove fields from `ScoresUiState` (keep empty struct for backward compat)
   - Remove fields from `StandingsUiState` (keep empty struct for backward compat)
   - Remove `pub demo: DocumentState` from `UiState` (no longer used)

2. `src/tui/reducers/document.rs`:
   - Remove `UpdateViewportHeight` handling (no longer dispatched)
   - Consider removing entire file if it becomes empty
   - Or convert to a simple placeholder

**Potential Impact**: Low - These fields are not being read or written anymore

**Benefit**: Cleaner architecture, less confusion about source of truth

---

### Phase 9: Remove Old Sub-Reducers

**Goal**: Simplify reducer architecture now that components handle their own state.

**Current State**:
- `reducers/scores.rs` - Just forwards to ComponentMessage
- `reducers/standings.rs` - Just forwards to ComponentMessage
- `reducers/document.rs` - Empty except UpdateViewportHeight handler (not used)

**Options**:

**Option A: Keep as Message Forwarders**
- Benefit: Clean separation between action types
- Drawback: Extra indirection

**Option B: Inline into Main Reducer**
- Benefit: Less files, clearer flow
- Drawback: Main reducer becomes larger

**Recommendation**: Keep for now (Option A). The indirection is minimal and keeps concerns separated.

---

### Phase 10: Remove DocumentAction Enum

**Goal**: Complete the migration from `DocumentAction` to `ComponentMessage`.

**Current State**:
- `DocumentAction` still exists in `src/tui/action.rs`
- Only `UpdateViewportHeight` is still defined
- Not currently dispatched from anywhere

**Steps**:
1. Remove `DocumentAction::UpdateViewportHeight` variant (or entire enum if it's the only one)
2. Remove `UpdateViewportHeight` message from component messages
3. Remove `UpdateViewportHeight` handler from `document.rs` reducer
4. Verify viewport height works (it should - comes from `area.height`)

**Risk**: Low - viewport height is already working without these actions

---

### Phase 11: Performance Optimization (Future)

**Current Performance**: 10-33ms per action (30-100 FPS) - Acceptable

**Potential Optimizations** (if needed later):

1. **Memoization**:
   - Cache element tree if props/state haven't changed
   - Use pointer equality for Arc-wrapped data
   - Implement `should_update()` check

2. **Virtual DOM Diffing**:
   - Only re-render changed parts of tree
   - Requires more sophisticated rendering architecture

3. **Debouncing/Throttling**:
   - Batch rapid key events
   - Update state but defer render

4. **Lazy Rendering**:
   - Only build elements that are visible
   - Useful for large lists/tables

**Recommendation**: Defer until there's a demonstrated performance problem.

---

## Design Principles (Learned from This Migration)

### Core Principles

1. **Component State is Source of Truth**
   - Never sync component state to global state
   - Global state only holds shared data (API responses, config)
   - UI state lives in components

2. **Messages are the API**
   - Components communicate via messages, not by modifying global state
   - Message types should be strongly typed and specific

3. **Generic Over Specific**
   - Avoid hardcoded component checks (like `is_standings_doc`)
   - Extract shared patterns into reusable modules (like `document_nav.rs`)
   - Use composition over inheritance

4. **Reducers Should Be Simple**
   - Sub-reducers just route actions to component messages
   - Business logic lives in component `update()` methods
   - Data loading is the main concern of global reducers

5. **Embedded Structs Over Traits**
   - Rust idiom: Embed common state structs directly
   - More explicit than trait-based composition
   - Example: `StandingsTabState { doc_nav: DocumentNavState, ... }`

6. **Avoid Infinite Loops**
   - Don't dispatch actions from render loop
   - Effects should eventually resolve to `Effect::None`
   - Use state checks to prevent redundant dispatches

### Testing Principles

1. Test components in isolation (unit tests for `update()`)
2. Test key routing separately from component logic
3. Use `assert_buffer` for render testing
4. Maintain 90%+ coverage

---

## Files Overview (Current State)

### Core Component Files
- `src/tui/component.rs` - Component trait, Effect, Element types ✅
- `src/tui/runtime.rs` - Runtime manages component lifecycle ✅
- `src/tui/component_store.rs` - ComponentStateStore ✅
- `src/tui/document_nav.rs` - Generic document navigation ✅ **NEW**

### Component Implementations
- `src/tui/components/app.rs` - Root component ✅
- `src/tui/components/scores_tab.rs` - Scores component ✅
- `src/tui/components/standings_tab.rs` - Standings component ✅
- `src/tui/components/demo_tab.rs` - Demo component ✅
- `src/tui/components/settings_tab.rs` - Settings (partial component)

### Reducer Files
- `src/tui/reducer.rs` - Main reducer ✅
- `src/tui/reducers/navigation.rs` - Navigation actions ✅
- `src/tui/reducers/panels.rs` - Panel stack ✅
- `src/tui/reducers/data_loading.rs` - Data load handlers ✅
- `src/tui/reducers/scores.rs` - Message forwarder ✅
- `src/tui/reducers/standings.rs` - Message forwarder ✅
- `src/tui/reducers/document.rs` - Nearly empty ⚠️ (candidate for removal)

### State Files
- `src/tui/state.rs` - AppState definition ⚠️ (has unused fields)
- `src/tui/action.rs` - Action enum ⚠️ (has DocumentAction to remove)

---

## Migration Lessons Learned

### What Worked Well

1. **Phased Approach**: Incremental migration prevented breaking everything
2. **Scores Tab POC**: Starting with one component proved the architecture
3. **Generic Patterns**: `document_nav.rs` eliminated code duplication
4. **Component State Store**: Centralized state management without global state
5. **Strong Typing**: Rust's type system caught many migration errors

### What Was Challenging

1. **Infinite Loop Bug**: Action dispatching from render loop
2. **State Sync Removal**: Had to carefully remove all sync code
3. **Test Updates**: Many tests needed updating for new action types
4. **Documentation**: Keeping plans in sync with implementation

### What We'd Do Differently

1. Start with generic patterns (like `document_nav`) earlier
2. Remove old state sync code immediately, not gradually
3. Write more component-level unit tests upfront
4. Document the "no dispatch from render" principle earlier

---

## Recommended Next Steps

### Short Term (Cleanup)

1. **Remove unused global UI state fields** (Phase 8)
   - Low risk, high clarity gain
   - Estimated: 1-2 hours

2. **Remove DocumentAction enum** (Phase 10)
   - Low risk, completes the migration conceptually
   - Estimated: 30 minutes

3. **Update documentation**
   - Document the generic document navigation pattern
   - Add examples to CLAUDE.md
   - Estimated: 1 hour

### Medium Term (Polish)

4. **Settings Tab Component**
   - Complete the component migration for Settings
   - Estimated: 2-4 hours

5. **Component Documentation**
   - Document each component's Props, State, Messages
   - Add architectural diagrams
   - Estimated: 2-3 hours

6. **Integration Tests**
   - Add more end-to-end component interaction tests
   - Test focus transitions, data loading, etc.
   - Estimated: 3-4 hours

### Long Term (Optimization)

7. **Performance Profiling**
   - Measure actual performance in real usage
   - Identify bottlenecks if any
   - Estimated: 2-3 hours

8. **Memoization**
   - Implement `should_update()` for expensive components
   - Cache element trees where appropriate
   - Estimated: 4-6 hours (if needed)

---

## Success Metrics

### Code Quality
- ✅ All tests passing (656/656)
- ✅ No compiler warnings
- ✅ No infinite loops
- ⚠️ Some unused state fields remain (cleanup pending)

### Architecture Quality
- ✅ Component state is source of truth
- ✅ Generic navigation pattern established
- ✅ No hardcoded component checks
- ✅ Clean message-based communication

### Performance
- ✅ 10-33ms per action (acceptable)
- ✅ Responsive to user input
- ✅ No noticeable lag

### Maintainability
- ✅ Clear component boundaries
- ✅ Reusable patterns (document_nav)
- ✅ Well-documented architecture
- ⚠️ Some deprecated code to remove

---

## Conclusion

**Phase 7 is complete!** The React-like component system is now fully functional with:
- Components owning their UI state
- Generic document navigation pattern
- Clean message-based architecture
- No global state pollution

The remaining work is cleanup and polish, not critical functionality. The system is production-ready and can be used as-is while we clean up the deprecated code at our leisure.

**Next Recommended Action**: Start with Phase 8 (Remove unused global state fields) for a quick win that improves code clarity.
