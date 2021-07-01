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
use crate::generator::qbe::*;

#[test]
fn qbe_value() {
    let val = QbeValue::Temporary("temp42".into());
    assert_eq!(format!("{}", val), "%temp42");

    let val = QbeValue::Global("main".into());
    assert_eq!(format!("{}", val), "$main");

    let val = QbeValue::Const(1337);
    assert_eq!(format!("{}", val), "1337");
}

#[test]
fn block() {
    let blk = QbeBlock {
        label: "start".into(),
        instructions: vec![QbeStatement::Volatile(QbeInstr::Ret(None))],
    };

    let formatted = format!("{}", blk);
    let mut lines = formatted.lines();
    assert_eq!(lines.next().unwrap(), "@start");
    assert_eq!(lines.next().unwrap(), "\tret");

    let blk = QbeBlock {
        label: "start".into(),
        instructions: vec![
            QbeStatement::Assign(
                QbeValue::Temporary("foo".into()),
                QbeType::Word,
                QbeInstr::Add(QbeValue::Const(2), QbeValue::Const(2)),
            ),
            QbeStatement::Volatile(QbeInstr::Ret(Some(QbeValue::Temporary("foo".into())))),
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
    let func = QbeFunction {
        exported: true,
        return_ty: None,
        name: "main".into(),
        arguments: Vec::new(),
        blocks: vec![QbeBlock {
            label: "start".into(),
            instructions: vec![QbeStatement::Volatile(QbeInstr::Ret(None))],
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
    let datadef = QbeDataDef {
        exported: true,
        name: "hello".into(),
        align: None,
        items: vec![
            (QbeType::Byte, QbeDataItem::Str("Hello, World!".into())),
            (QbeType::Byte, QbeDataItem::Const(0)),
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
    let typedef = QbeTypeDef {
        name: "person".into(),
        align: None,
        items: vec![(QbeType::Long, 1), (QbeType::Word, 2), (QbeType::Byte, 1)],
    };

    let formatted = format!("{}", typedef);
    assert_eq!(formatted, "type :person = { l, w 2, b }");
}

#[test]
fn type_into_abi() {
    // Base types and aggregates should stay unchanged
    let unchanged = |ty: QbeType| assert_eq!(ty.clone().into_abi(), ty);
    unchanged(QbeType::Word);
    unchanged(QbeType::Long);
    unchanged(QbeType::Single);
    unchanged(QbeType::Double);
    unchanged(QbeType::Aggregate("foo".into()));

    // Extended types are transformed into closest base types
    assert_eq!(QbeType::Byte.into_abi(), QbeType::Word);
    assert_eq!(QbeType::Halfword.into_abi(), QbeType::Word);
}

#[test]
fn type_into_base() {
    // Base types should stay unchanged
    let unchanged = |ty: QbeType| assert_eq!(ty.clone().into_base(), ty);
    unchanged(QbeType::Word);
    unchanged(QbeType::Long);
    unchanged(QbeType::Single);
    unchanged(QbeType::Double);

    // Extended and aggregate types are transformed into closest base types
    assert_eq!(QbeType::Byte.into_base(), QbeType::Word);
    assert_eq!(QbeType::Halfword.into_base(), QbeType::Word);
    assert_eq!(QbeType::Aggregate("foo".into()).into_base(), QbeType::Long);
}
