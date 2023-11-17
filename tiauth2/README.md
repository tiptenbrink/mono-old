## This repository has been archived

This was supposed to be the OAuth-compliant version of [tiptenbrink/tiauth](https://github.com/tiptenbrink/tiauth). It was supposed to basically be a Rust-copy of [DSAV-Dodeka/backend](https://github.com/DSAV-Dodeka/backend). However, time constraints have made this impossible. There was a short-lived idea for a mixed Rust/Python auth server (#1 and #2). This project contains little interesting code. It is kept only for archival purposes.

## Original README

This is a native Rust project, but also accommodates Python bindings in a way that allow the database and/or server to be run from Python.

It has three main components, `server`, `auth` and `data`. The core of this project is the `auth` component, which will be fully written in pure Rust. The `auth` component concerns the general endpoint processing, OAuth/OpenID logic and JWT logic.
