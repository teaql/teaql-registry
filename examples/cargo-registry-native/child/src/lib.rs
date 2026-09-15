#[derive(serde::Serialize)]
pub struct NativeCargoProof {
    pub value: u32,
}

pub fn child_value() -> u32 {
    let proof = NativeCargoProof {
        value: teaql_registry_test_base_0916::base_value() + 1,
    };
    proof.value
}
