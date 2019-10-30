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

- [removed] `bmp_uart_baudrate!` macro removed in favor of
  `drone_cortex_m::itm::update_prescaler!` macro
- [added] `periph!`, `periph_map!`, `periph_singular!` now accept arbitrary
  condition compilation flags
- [fixed] Accept `VAL` as a field name in `reg!` macro

### v0.10.1 (2019-09-27)

- [fixed] Fix API documentation by moving to self-hosted https://api.drone-os.com
