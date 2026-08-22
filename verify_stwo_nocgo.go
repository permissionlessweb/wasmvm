//go:build !cgo || nolink_libwasmvm

package cosmwasm

import "fmt"

func VerifyStwoHost(proof, instances []byte) error {
	_ = proof
	_ = instances
	return fmt.Errorf("libwasmvm not linked")
}
