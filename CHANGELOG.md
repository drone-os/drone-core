# Changelog

This project follows semantic versioning.

Possible log types:

- `[added]` for new features.
- `[changed]` for changes in existing functionality.
- `[deprecated]` for once-stable features removed in upcoming releases.
- `[removed]` for deprecated features removed in this release.
- `[fixed]` for any bug fixes.
- `[security]` to invite users to upgrade in case of vulnerabilities.

### Unreleased

- [added] Implement multi-variant registers
- [added] Add `load_bits`, `store_bits` methods for register tokens
- [removed] Remove `shrink_in_place` and `grow_in_place` hooks in `heap!`

### v0.11.1 (2019-11-27)

- [added] Implement 11 `inventory` counters (was 9)
- [changed] Upgraded to `syn` 1.0
- [changed] Using the newly released `futures` 0.3 instead of `futures-preview`
  0.3-alpha

### v0.11.0 (2019-11-06)

- [changed] Renamed `to_ptr`/`to_mut_ptr` to `as_ptr`/`as_mut_ptr`
- [removed] `bmp_uart_baudrate!` macro removed in favor of
  `drone_cortex_m::itm::update_prescaler!` macro
- [added] `periph!`, `periph_map!`, `periph_singular!` now accept arbitrary
  condition compilation flags
- [fixed] Accept `VAL` as a field name in `reg!` macro

### v0.10.1 (2019-09-27)

- [fixed] Fix API documentation by moving to self-hosted https://api.drone-os.com
