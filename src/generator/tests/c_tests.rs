use crate::ast::types::TypeKind;
use crate::generator::c::generate_type;
use crate::util::Either;

#[test]
fn test_generate_type_regular_type() {
    let t = generate_type(Either::Right(Some(TypeKind::Int)));
    assert_eq!(t, "int")
}
