# Polygon Crunchyroll Checker Archive

**Status:** Archived – Polygon has shut down. No longer operational.  
Open-sourced for archival purposes. No future updates. Credit Polygon if used.

## What is Crunchyroll Checker - Polygon?
Polygon's Crunchyroll account checker. Validates email:pass, detects Free/Premium, captures detailed profile info. Built in Rust with web interface.

## Features
- Validates Crunchyroll accounts
- Detects Free vs Premium subscription
- Captures username, profile, email verification, creation date, maturity rating
- Single & bulk checking modes
- Proxy rotation (premium proxies recommended)
- Discord webhook support
- Modern Axum-based web dashboard

## How to Install
1. Install Rust (rustup.rs)
2. `git clone` the repo
3. `cd` into the folder
4. `cargo build --release`
5. Run: `./target/release/polygon-crunchyroll-checker`

Dependencies are in `Cargo.toml`
