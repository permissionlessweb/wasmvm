//go:build cgo && !nolink_libwasmvm

package cosmwasm

import "github.com/CosmWasm/wasmvm/v3/internal/api"

// VerifyStwoHost runs named Stwo/M31 verify (FOLD/SSLE) in libwasmvm.
// Dummy DSTW is rejected. This is the ABCI waist without a contract.
//
// Contracts must use env.proof_instance_verify (Path A) with a stored
// curve_id=5 circuit — the same AnyVerifyingKey::verify this C ABI now
// calls. Do not add a second guest import for Stwo.
func VerifyStwoHost(proof, instances []byte) error {
	return api.VerifyStwoHost(proof, instances)
}
