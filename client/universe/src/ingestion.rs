//! Offline, source-specific adapters. Consumers supply verified local bytes;
//! these APIs neither download data nor authorize production publication.
mod csv;
pub mod exoplanets;
pub mod gaia;
pub mod horizons;
