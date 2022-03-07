/**
 * Copyright 2021 Alexey Yerin
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[test]
fn qbe_value() {
    let val = qbe::Value::Temporary("temp42".into());
    assert_eq!(format!("{}", val), "%temp42");

    let val = qbe::Value::Global("main".into());
    assert_eq!(format!("{}", val), "$main");

    let val = qbe::Value::Const(1337);
    assert_eq!(format!("{}", val), "1337");
}

#[test]
fn block() {
    let blk = qbe::Block {
        label: "start".into(),
        statements: vec![qbe::Statement::Volatile(qbe::Instr::Ret(None))],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");

    let blk = qbe::Block {
        label: "start".into(),
        statements: vec![
            qbe::Statement::Assign(
                qbe::Value::Temporary("foo".into()),
                qbe::Type::Word,
                qbe::Instr::Add(qbe::Value::Const(2), qbe::Value::Const(2)),
            ),
            qbe::Statement::Volatile(qbe::Instr::Ret(Some(qbe::Value::Temporary("foo".into())))),
        ],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\t%foo =w add 2, 2");
    assert_eq!(lines.next().unwrap(), "\tret %foo");
}

#[test]
fn function() {
    let func = qbe::Function {
        linkage: qbe::Linkage::public(),
        return_ty: None,
        name: "main".into(),
        arguments: Vec::new(),
        blocks: vec![qbe::Block {
            label: "start".into(),
            statements: vec![qbe::Statement::Volatile(qbe::Instr::Ret(None))],
        }],
    };

    let formatted = format!("{}", func);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "export function $main() {");
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");
    assert_eq!(lines.next().unwrap(), "}");
}

#[test]
fn datadef() {
    let datadef = qbe::DataDef {
        linkage: qbe::Linkage::public(),
        name: "hello".into(),
        align: None,
        items: vec![
            (qbe::Type::Byte, qbe::DataItem::Str("Hello, World!".into())),
            (qbe::Type::Byte, qbe::DataItem::Const(0)),
        ],
    };

    let formatted = format!("{}", datadef);
    assert_eq!(
        formatted,
        "export data $hello = { b \"Hello, World!\", b 0 }"
    );
}

#[test]
fn typedef() {
    let typedef = qbe::TypeDef {
        name: "person".into(),
        align: None,
        items: vec![
            (qbe::Type::Long, 1),
            (qbe::Type::Word, 2),
            (qbe::Type::Byte, 1),
        ],
    };

    let formatted = format!("{}", typedef);
    assert_eq!(formatted, "type :person = { l, w 2, b }");
}

#[test]
fn type_into_abi() {
    // Base types and aggregates should stay unchanged
    let unchanged = |ty: qbe::Type| assert_eq!(ty.clone().into_abi(), ty);
    unchanged(qbe::Type::Word);
    unchanged(qbe::Type::Long);
    unchanged(qbe::Type::Single);
    unchanged(qbe::Type::Double);
    unchanged(qbe::Type::Aggregate("foo".into()));

    // Extended types are transformed into closest base types
    assert_eq!(qbe::Type::Byte.into_abi(), qbe::Type::Word);
    assert_eq!(qbe::Type::Halfword.into_abi(), qbe::Type::Word);
}

#[test]
fn type_into_base() {
    // Base types should stay unchanged
    let unchanged = |ty: qbe::Type| assert_eq!(ty.clone().into_base(), ty);
    unchanged(qbe::Type::Word);
    unchanged(qbe::Type::Long);
    unchanged(qbe::Type::Single);
    unchanged(qbe::Type::Double);

    // Extended and aggregate types are transformed into closest base types
    assert_eq!(qbe::Type::Byte.into_base(), qbe::Type::Word);
    assert_eq!(qbe::Type::Halfword.into_base(), qbe::Type::Word);
    assert_eq!(
        qbe::Type::Aggregate("foo".into()).into_base(),
        qbe::Type::Long
    );
}
