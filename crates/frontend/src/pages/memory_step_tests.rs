use super::*;

const CEILING: u32 = 15 * 1024;

#[test]
fn one_click_is_one_gigabyte() {
    assert_eq!(step(2048, 10240, false, true, CEILING), (2048, 11264));
    assert_eq!(step(2048, 10240, false, false, CEILING), (2048, 9216));
    assert_eq!(step(2048, 10240, true, true, CEILING), (3072, 10240));
}

/// A value saved in 512 MB steps lands on the nearest gigabyte in the
/// direction of the click instead of staying half a gigabyte off.
#[test]
fn half_gigabytes_snap_to_whole_ones() {
    assert_eq!(step(1536, 4608, false, true, CEILING), (1536, 5120));
    assert_eq!(step(1536, 4608, false, false, CEILING), (1536, 4096));
    assert_eq!(step(1536, 4608, true, false, CEILING), (1024, 4608));
}

#[test]
fn bounds_hold() {
    assert_eq!(step(1024, 1024, true, false, CEILING), (1024, 1024));
    assert_eq!(
        step(1024, 15 * 1024, false, true, CEILING),
        (1024, 15 * 1024)
    );
    // A ceiling that isn't a whole gigabyte tops out at the gigabyte below it.
    assert_eq!(step(1024, 6144, false, true, 6900), (1024, 6144));
}

#[test]
fn the_maximum_follows_the_minimum_up() {
    assert_eq!(step(4096, 4096, true, true, CEILING), (5120, 5120));
    assert_eq!(step(4096, 4096, false, false, CEILING), (4096, 4096));
}
