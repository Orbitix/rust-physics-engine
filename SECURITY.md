# Security Policy

## Known Vulnerabilities

### Macroquad Soundness Issues

**Status**: MITIGATED (Not included in default builds)

**Affected Component**: version_3d binary only

**Details**:
- Package: macroquad v0.4.14
- Issue: Multiple soundness issues
- Patched Version: Not available
- CVE: N/A (Rust advisory)

**Mitigation**:
The macroquad dependency is marked as **optional** and is NOT included in default builds. 

- ✅ Safe by default: `cargo build` does not include macroquad
- ✅ Explicit opt-in required: version_3d requires `--features version_3d`
- ✅ Alternative available: Use version_2d (Bevy-based) which has no known vulnerabilities

**Recommendations**:
1. **Use version_2d**: The Bevy-based 2D version is the recommended, secure version
2. **Avoid version_3d**: Only build version_3d if you understand and accept the security risks
3. **Stay updated**: Monitor macroquad releases for security patches

## Building Securely

### Safe Build (Recommended)
```bash
cargo build --bin version_2d
cargo run --bin version_2d
```

### Vulnerable Build (Not Recommended)
```bash
# Only if you understand and accept the risks
cargo build --bin version_3d --features version_3d
```

## Reporting Vulnerabilities

If you discover a security vulnerability in this project, please report it by creating a private security advisory on GitHub.
