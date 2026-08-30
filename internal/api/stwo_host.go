package api

// #include <stdlib.h>
// #include "bindings.h"
import "C"

import (
	"runtime"
)

// VerifyStwoHost verifies a named Stwo/M31 proof (FOLD or SSLE) in-process.
// Dummy DSTW is rejected. No CosmWasm contract is required.
// libwasmvm routes this through Path A AnyVerifyingKey.Verify (same as
// env.proof_instance_verify). Guest contracts must not grow a second Stwo import.
func VerifyStwoHost(proof, instances []byte) error {
	p := makeView(proof)
	defer runtime.KeepAlive(proof)
	i := makeView(instances)
	defer runtime.KeepAlive(instances)
	errmsg := uninitializedUnmanagedVector()
	_, err := C.verify_stwo_host_proof(p, i, &errmsg)
	if err != nil {
		return errorWithMessage(err, errmsg)
	}
	return nil
}
