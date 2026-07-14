package main

import (
	"bytes"
	"os"
	"testing"

	nativepb "gluerift/native/gen/nativepb"
)

func TestDecisionMapsRoundTrip(t *testing.T) {
	for _, value := range []sourceDecision{decisionDeny, decisionAllow} {
		decoded, err := decodeDecision(encodeDecision(value))
		if err != nil || decoded != value {
			t.Fatalf("decision round trip for %q: decoded=%q err=%v", value, decoded, err)
		}
	}
}

func TestBoundsMapsRoundTripExhaustive(t *testing.T) {
	for minimum := int32(0); minimum <= 2; minimum++ {
		for maximum := int32(0); maximum <= 2; maximum++ {
			value := nestedOutput{Policy: policy{Bounds: bounds{Minimum: minimum, Maximum: maximum}}}
			decoded, err := decodeBounds(encodeBounds(value))
			if err != nil || decoded != value {
				t.Fatalf("bounds round trip for %+v: decoded=%+v err=%v", value, decoded, err)
			}
		}
	}
}

func TestReadFrameRejectsConcatenatedMessages(t *testing.T) {
	one, err := frame(encodeDecision(decisionDeny))
	if err != nil {
		t.Fatal(err)
	}
	old := os.Stdin
	defer func() { os.Stdin = old }()
	read, write, err := os.Pipe()
	if err != nil {
		t.Fatal(err)
	}
	if _, err := write.Write(bytes.Join([][]byte{one, one}, nil)); err != nil {
		t.Fatal(err)
	}
	_ = write.Close()
	os.Stdin = read
	if err := readFrame(new(nativepb.E01Carrier)); err == nil {
		t.Fatal("concatenated messages were accepted")
	}
}
