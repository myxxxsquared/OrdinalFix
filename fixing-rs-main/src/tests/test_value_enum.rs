use fixing_rs_base::utils::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum Enum0 {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
enum EnumTest {
    A,
    B,
    C,
}

#[test]
fn test_value_enum() {
    assert_eq!(EnumTest::A.value(), 0);
    assert_eq!(EnumTest::B.value(), 1);
    assert_eq!(EnumTest::C.value(), 2);
    assert_eq!(EnumTest::from_value(0), Some(EnumTest::A));
    assert_eq!(EnumTest::from_value(1), Some(EnumTest::B));
    assert_eq!(EnumTest::from_value(2), Some(EnumTest::C));
    assert_eq!(EnumTest::from_value(3), None);
    assert_eq!(EnumTest::N, 3);
}
