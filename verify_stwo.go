//go:build cgo && !nolink_libwasmvm

package cosmwasm

import "github.com/CosmWasm/wasmvm/v3/internal/api"

// VerifyStwoHost runs named Stwo/M31 verify (FOLD/SSLE) in libwasmvm.
// Dummy DSTW is rejected. This is the ABCI waist without a contract.
func VerifyStwoHost(proof, instances []byte) error {
	return api.VerifyStwoHost(proof, instances)
}
