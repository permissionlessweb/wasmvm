package api

import "testing"

func TestVerifyStwoHostRejectsDummyDSTW(t *testing.T) {
	proof := make([]byte, 18)
	copy(proof[0:4], []byte("DSTW"))
	proof[4] = 2
	proof[5] = 5
	if err := VerifyStwoHost(proof, nil); err == nil {
		t.Fatal("expected dummy DSTW reject")
	}
}

func TestVerifyStwoHostRejectsShort(t *testing.T) {
	if err := VerifyStwoHost([]byte("nope"), nil); err == nil {
		t.Fatal("expected reject")
	}
}
