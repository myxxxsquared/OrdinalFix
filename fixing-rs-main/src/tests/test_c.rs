use crate::c::fixing::CFixingInputProcessor;
use fixing_rs_base::fixing::{fix, FixTaskInfo, FixTaskResult};

fn test_c_folder(folder: &str, dist: usize) {
    let info = FixTaskInfo {
        input_name: format!("src/tests/test_c/{}/c.tokens", folder),
        env_name: format!("src/tests/test_c/{}/env", folder),
        output_name: None,
        max_len: dist,
        max_new_id: dist,
        verbose_gen: true,
    };
    let result = fix(std::iter::once(info), &CFixingInputProcessor);
    let result: [Result<FixTaskResult, _>; 1] = result.try_into().unwrap();
    let [result] = result;
    let result = result.unwrap();
    assert_eq!(result.found_length.unwrap(), dist);
    println!("{:?}", result);
}

#[test]
fn test_c_basic() {
    test_c_folder("basic", 3)
}

#[test]
fn test_c_ids() {
    test_c_folder("ids", 1)
}

#[test]
fn test_c_lval() {
    test_c_folder("lval", 1)
}

#[test]
fn test_c_decls() {
    test_c_folder("decls", 1)
}

#[test]
fn test_c_decls2() {
    test_c_folder("decls2", 2)
}

#[test]
fn test_c_args() {
    test_c_folder("args", 2)
}

#[test]
fn test_c_array() {
    test_c_folder("array", 1)
}

#[test]
fn test_c_printf() {
    test_c_folder("printf", 2)
}