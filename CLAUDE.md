# termdemo project notes

## Adding a new effect — required updates

Every new effect added to `src/effects/` must be accompanied by these synchronized updates, or the project's metadata drifts out of sync:

1. **`src/effects/mod.rs`** — declare the new module (`pub mod foo;`).
2. **`src/main.rs`** — add the `use` import and a `Scene::new(Box::new(Foo::new()))` entry in `build_scenes()` in the appropriate Act.
3. **`README.md`** — bump the effect count in the intro line, and add a `#### N. Name` section under the matching Act with **Technique** and (where applicable) **Heritage** lines. Renumber subsequent effects if the new one is inserted mid-list.
4. **Scroller text** in `main.rs` — update the count in the finale `Scroller::new("NN EFFECTS IN YOUR TERMINAL ...")` string so the closing greetz line stays accurate.

The effect count must match across: README intro, README per-section count (`grep -c '^#### '`), `Scene::new` calls in `main.rs`, files under `src/effects/` excluding `mod.rs`, and the scroller's hardcoded number.
