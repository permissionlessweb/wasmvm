//! Real S-two host installed into zk-cosmwasm [`STWO_HOST_VERIFY`].
//! Dummy DSTW is rejected. FOLD and SSLE are verified by pinned S-two.
//!
//! **Singular verify API:** contracts use `env.proof_instance_verify` (Path A),
//! which loads a `curve_id=5` VK and calls [`AnyVerifyingKey::verify`]. That
//! path uses [`STWO_HOST_VERIFY`] when installed. The C ABI below is **only**
//! for ABCI ProcessProposal (no contract) and **must** go through the same
//! `AnyVerifyingKey::verify` — not a parallel proof format.

use std::panic::{catch_unwind, AssertUnwindSafe};

use zk_cosmwasm::{
    AnyInstance, AnyVerifyingKey, Proof, StwoInstance, StwoVerifyingKey, ZkError, ZkResult,
};

use crate::error::{handle_c_error_default, Error};
use crate::handle_vm_panic::handle_vm_panic;
use crate::memory::{ByteSliceView, UnmanagedVector};

pub fn install() {
    let _ = zk_cosmwasm::STWO_HOST_VERIFY.set(host_verify);
}

/// In-process S-two verify. Used by the C ABI and by `proof_instance_verify`.
pub fn host_verify(proof: &[u8], instances: &[u8]) -> ZkResult<()> {
    if proof.windows(4).any(|w| w == b"DSTW") {
        return Err(ZkError::new_err("stwo: Dummy DSTW rejected"));
    }
    if proof.len() < 10 || &proof[0..4] != b"STWO" {
        return Err(ZkError::new_err("stwo: not STWO"));
    }
    if proof[4] != 2 {
        return Err(ZkError::new_err("stwo: prover_id must be 2"));
    }
    if proof[5] != 5 {
        return Err(ZkError::new_err("stwo: curve_id must be 5 (M31)"));
    }
    match &proof[6..10] {
        b"FOLD" => verify_fold(proof, instances),
        b"SSLE" => verify_ssle(proof, instances),
        _ => Err(ZkError::new_err("stwo: unknown kind (want FOLD or SSLE)")),
    }
}

fn verify_fold(proof: &[u8], instances: &[u8]) -> ZkResult<()> {
    const PI: usize = 96;
    if proof.len() < 10 + PI {
        return Err(ZkError::new_err("stwo: truncated FOLD"));
    }
    let pi = &proof[10..10 + PI];
    if !instances.is_empty() && instances != pi {
        return Err(ZkError::VerifyFailed);
    }
    let mut bf = [0u8; 32];
    let mut dep = [0u8; 32];
    let mut eb = [0u8; 32];
    bf.copy_from_slice(&pi[0..32]);
    dep.copy_from_slice(&pi[32..64]);
    eb.copy_from_slice(&pi[64..96]);
    lean_stwo_dummy::verify_fold(proof, bf, dep, eb).map_err(|_| ZkError::VerifyFailed)
}

fn verify_ssle(proof: &[u8], instances: &[u8]) -> ZkResult<()> {
    const PI: usize = 48;
    if proof.len() < 10 + PI {
        return Err(ZkError::new_err("stwo: truncated SSLE"));
    }
    let pi = &proof[10..10 + PI];
    if !instances.is_empty() && instances != pi {
        return Err(ZkError::VerifyFailed);
    }
    let period = u64::from_be_bytes(
        pi[0..8]
            .try_into()
            .map_err(|_| ZkError::format_err("stwo: ssle period"))?,
    );
    let height = u64::from_be_bytes(
        pi[8..16]
            .try_into()
            .map_err(|_| ZkError::format_err("stwo: ssle height"))?,
    ) as i64;
    let mut ticket = [0u8; 32];
    ticket.copy_from_slice(&pi[16..48]);
    lean_stwo_dummy::verify_ssle(proof, period, height, &ticket).map_err(|_| ZkError::VerifyFailed)
}

/// Same Path A dispatch contracts use: footer-only Stwo VK + `AnyVerifyingKey::verify`.
/// Installs [`STWO_HOST_VERIFY`] so dummy DSTW is rejected on this host.
pub fn path_a_verify_stwo(proof: &[u8], instances: &[u8]) -> ZkResult<()> {
    install();
    let vk = AnyVerifyingKey::Stwo(StwoVerifyingKey::lean_default());
    let i = AnyInstance::Stwo(StwoInstance {
        bytes: instances.to_vec(),
    });
    vk.verify(&Proof::new(proof.to_vec()), std::slice::from_ref(&i))
}

/// ProcessProposal waist: verify FOLD/SSLE without a CosmWasm contract.
/// Wired through Path A `AnyVerifyingKey::verify` (not a second proof format).
#[unsafe(no_mangle)]
pub extern "C" fn verify_stwo_host_proof(
    proof: ByteSliceView,
    instances: ByteSliceView,
    error_msg: Option<&mut UnmanagedVector>,
) {
    let r = catch_unwind(AssertUnwindSafe(|| {
        let p = proof
            .read()
            .ok_or_else(|| Error::unset_arg("proof"))?;
        let inst = instances.read().unwrap_or(&[]);
        path_a_verify_stwo(p, inst).map_err(|e| Error::vm_err(format!("{e:?}")))
    }))
    .unwrap_or_else(|err| {
        handle_vm_panic("verify_stwo_host_proof", err);
        Err(Error::panic())
    });
    handle_c_error_default(r, error_msg);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_rejects_dummy_dstw() {
        let mut p = vec![0u8; 18];
        p[0..4].copy_from_slice(b"DSTW");
        p[4] = 2;
        p[5] = 5;
        assert!(host_verify(&p, &[]).is_err());
    }

    #[test]
    fn host_ssle_roundtrip() {
        let proposer = [0x42u8; 32];
        let (ticket, proof) = lean_stwo_dummy::prove_ssle(1, 7, &proposer).expect("prove");
        host_verify(&proof, &[]).expect("host verify");
        assert_ne!(ticket, proposer);
        let mut bad = proof.clone();
        bad[proof.len() / 2] ^= 1;
        assert!(host_verify(&bad, &[]).is_err());
    }

    #[test]
    fn path_a_same_as_host_verify_ssle() {
        let proposer = [0x11u8; 32];
        let (_ticket, proof) = lean_stwo_dummy::prove_ssle(2, 9, &proposer).expect("prove");
        path_a_verify_stwo(&proof, &[]).expect("path A");
        let mut bad = proof.clone();
        bad[proof.len() / 2] ^= 1;
        assert!(path_a_verify_stwo(&bad, &[]).is_err());
    }
}
