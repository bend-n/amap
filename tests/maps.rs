use amap::amap;

fn main() {
    assert_eq!(
        amap! {
          2 => 7,
          3 | 6 => 3,
          4..6 => 2,
        },
        [None, None, Some(7), Some(3), Some(2), Some(2), Some(3)]
    );
}
