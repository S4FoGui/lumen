# Task Checklist: Overlay Persistence Fix (v1.1.1)

- [/] Phase 3: Architect (Implementation)
    - [x] Create implementation plan (approved)
    - [ ] Add `is_recording` state to `overlay.rs`
    - [ ] Link `ShowRecording` and `HideRecording` to the state
    - [ ] Update ticker auto-dismiss logic (check recording status)
    - [ ] Fix potential tick callback reactivation bug
- [ ] Phase 5: Verify
    - [ ] Build check
    - [ ] Multiple recording cycles check
