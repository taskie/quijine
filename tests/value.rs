use maplit::btreemap;
use quijine::{EvalFlags, FromQj};

#[test]
fn test_iterator_array() {
    quijine::context(|ctx| {
        let xs = ctx.new_array_from([2, 3, 5])?;
        let mut last_i = 0;
        for (i, x) in xs.iterator()?.enumerate() {
            last_i = i;
            let x: i32 = x?.try_into()?;
            assert_eq!(
                match i {
                    0 => 2,
                    1 => 3,
                    2 => 5,
                    _ => -1,
                },
                x
            );
        }
        assert_eq!(2, last_i);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_iterator_generator() {
    quijine::context(|ctx| {
        let f = ctx.eval(
            r#"
                (function * () { for (let i = 0; i < 3; ++i) yield i; })();
            "#,
            "<input>",
            EvalFlags::TYPE_GLOBAL | EvalFlags::FLAG_STRICT,
        )?;
        let mut last_i = 0;
        for (i, v) in f.iterator()?.enumerate() {
            last_i = i;
            let x: i32 = v?.try_into()?;
            assert_eq!(i as i32, x);
        }
        assert_eq!(2, last_i);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_keys() {
    quijine::context(|ctx| {
        let obj = ctx.new_object_from_entries(btreemap! {"H" => 1, "He" => 2})?;
        let mut last_i = 0;
        for (i, k) in obj.keys()?.enumerate() {
            last_i = i;
            let k: String = k?.try_into()?;
            assert_eq!(
                match i {
                    0 => "H",
                    1 => "He",
                    _ => "",
                },
                k
            );
        }
        assert_eq!(1, last_i);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_values() {
    quijine::context(|ctx| {
        let obj = ctx.new_object_from_entries(btreemap! {"H" => 1, "He" => 2})?;
        let mut last_i = 0;
        for (i, v) in obj.values()?.enumerate() {
            last_i = i;
            let k: i32 = v?.try_into()?;
            assert_eq!(
                match i {
                    0 => 1,
                    1 => 2,
                    _ => -1,
                },
                k
            );
        }
        assert_eq!(1, last_i);
        Ok(())
    })
    .unwrap();
}

#[test]
fn test_entries() {
    quijine::context(|ctx| {
        let obj = ctx.new_object_from_entries(btreemap! {"H" => 1, "He" => 2})?;
        let mut last_i = 0;
        for (i, v) in obj.entries()?.enumerate() {
            last_i = i;
            let entry = v?;
            let entry = (String::from_qj(entry.0)?, i32::from_qj(entry.1)?);
            assert_eq!(
                match i {
                    0 => ("H".to_owned(), 1),
                    1 => ("He".to_owned(), 2),
                    _ => ("".to_owned(), -1),
                },
                entry
            );
        }
        assert_eq!(1, last_i);
        Ok(())
    })
    .unwrap();
}
