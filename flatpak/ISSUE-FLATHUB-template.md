# Flathub submission issue template

Copy into: https://github.com/flathub/flathub/issues/new (new application template, if available).

**Suggested issue title:** `Add com.formatpen.FormatPen`

---

## Application

| Field | Value |
|-------|--------|
| **Name** | FormatPen |
| **Application ID** | `com.formatpen.FormatPen` |
| **Upstream** | https://github.com/brayanmonteiroo/formatpen |
| **License** | MIT |
| **Initial tag** | `v1.0.2` |
| **Commit** | `c1014ea0529634d68dea5131abac3e27064ebe28` |

## Summary

FormatPen is a GTK4/Libadwaita USB drive formatter for Linux. It lists the whole block device (not individual partitions), repartitions into a single partition, and supports FAT32, exFAT, NTFS, and ext4 via UDisks2.

Metainfo and desktop entry include **English** and **Brazilian Portuguese** (`pt_BR`). The application UI is primarily in Portuguese.

## Permission justification

| Permission | Reason |
|------------|--------|
| `--talk-name` / `--system-talk-name=org.freedesktop.UDisks2` | Format and repartition removable block devices |
| `--talk-name=org.freedesktop.PolicyKit1` | Administrator password prompt when formatting |
| `--filesystem=host` | UDisks2 integration and host filesystem tools (`mkfs`, etc.) |
| `--device=all` | Access to removable block devices |

## Submission readiness

- [x] AppStream metainfo with screenshot (upstream tag `v1.0.2`, en + pt_BR)
- [x] Local `flatpak-builder` test passed with `cargo-sources.json` (offline Rust build)
- [ ] Pull request to `flathub/flathub` branch `new-pr` (after this issue is acknowledged)

## PR title (when opening the Flathub fork PR)

`Add com.formatpen.FormatPen`
