#![cfg_attr(any(), rustfmt::skip)]
// generated with `cargo run --bin gen_versions > src/proto/version/mod.rs`

/// for minecraft versions: 1.7 & 1.7.1
// TODO: need o look at old mc source as the wiki is missing this?
// TODO: play state?
// pub mod v3;
/// for minecraft versions: 1.7.2, 1.7.3, 1.7.4 & 1.7.5
pub mod v4;
/// for minecraft versions: 1.7.6, 1.7.7, 1.7.8, 1.7.9 & 1.7.10
pub mod v5;
/// for minecraft versions: 1.8, 1.8.1, 1.8.2, 1.8.3, 1.8.4, 1.8.5, 1.8.6, 1.8.7, 1.8.8 & 1.8.9
pub mod v47;
/// for minecraft versions: 1.9
pub mod v107;
/// for minecraft versions: 1.9.1
pub mod v108;
/// for minecraft versions: 1.9.2
pub mod v109;
/// for minecraft versions: 1.9.3 & 1.9.4
pub mod v110;
/// for minecraft versions: 1.10, 1.10.1 & 1.10.2
pub mod v210;
/// for minecraft versions: 1.11
pub mod v315;
/// for minecraft versions: 1.11.1 & 1.11.2
pub mod v316;
/// for minecraft versions: 1.12
pub mod v335;
/// for minecraft versions: 1.12.1
pub mod v338;
/// for minecraft versions: 1.12.2
pub mod v340;
/// for minecraft versions: 1.13
pub mod v393;
/// for minecraft versions: 1.13.1
pub mod v401;
/// for minecraft versions: 1.13.2
pub mod v404;
/// for minecraft versions: 1.14
pub mod v477;
/// for minecraft versions: 1.14.1
pub mod v480;
/// for minecraft versions: 1.14.2
pub mod v485;
/// for minecraft versions: 1.14.3
pub mod v490;
/// for minecraft versions: 1.14.4
pub mod v498;
/// for minecraft versions: 1.15
pub mod v573;
/// for minecraft versions: 1.15.1
pub mod v575;
/// for minecraft versions: 1.15.2
pub mod v578;
/// for minecraft versions: 1.16
pub mod v735;
/// for minecraft versions: 1.16.1
pub mod v736;
/// for minecraft versions: 1.16.2
pub mod v751;
/// for minecraft versions: 1.16.3
pub mod v753;
/// for minecraft versions: 1.16.4 & 1.16.5
// TODO: 1.16.4 & 1.16.5 have the same protocol version but have different entity metadatas
pub mod v754;
/// for minecraft versions: 1.17
pub mod v755;
/// for minecraft versions: 1.17.1
pub mod v756;
/// for minecraft versions: 1.18 & 1.18.1
// TODO: same as 754?
pub mod v757;
/// for minecraft versions: 1.18.2
pub mod v758;
/// for minecraft versions: 1.19
pub mod v759;
/// for minecraft versions: 1.19.1 & 1.19.2
pub mod v760;
/// for minecraft versions: 1.19.3
pub mod v761;
/// for minecraft versions: 1.19.4
pub mod v762;
/// for minecraft versions: 1.20 & 1.20.1
// TODO: same as 754?
pub mod v763;
/// for minecraft versions: 1.20.2
pub mod v764;
/// for minecraft versions: 1.20.3 & 1.20.4
// TODO: same as 754?
pub mod v765;
/// for minecraft versions: 1.20.5 & 1.20.6
pub mod v766;
/// for minecraft versions: 1.21
pub mod v767;

pub use v767 as latest;
