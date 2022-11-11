=================================
Tracing subscriber for COVESA DLT
=================================

This crate is part of the Sabaton Automotive platform, but can also be
used independantly.  

The tracing crate https://crates.io/crates/tracing is the most popular
application level tracing library for Rust.  This crate implements 
a tracing subscriber that will route trace and log messages to a dlt-daemon
using libdlt.  The crate depends on https://crates.io/crates/libdlt-sys .



