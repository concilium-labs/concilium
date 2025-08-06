pub const STAKED_PER_NODE: f32 = 500.0;
pub const NTP: f32 = 500.0; // number of transactions processed

// calculating the number of nodes required
pub fn calculating_nnr(cyn: f32) -> u32 {
    let result = (24.0 * NTP * cyn) / STAKED_PER_NODE;
    result.ceil() as u32
}