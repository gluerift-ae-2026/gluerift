package main

import (
	"bytes"
	"encoding/binary"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"os"

	nativepb "gluerift/native/gen/nativepb"
	"google.golang.org/protobuf/proto"
)

const (
	protocolSchema = "gluerift.native-protocol/v1"
	maxFrame       = 4096
)

type sourceDecision string

const (
	decisionDeny  sourceDecision = "DENY"
	decisionAllow sourceDecision = "ALLOW"
)

type bounds struct {
	Minimum int32 `json:"minimum"`
	Maximum int32 `json:"maximum"`
}

type policy struct {
	Bounds bounds `json:"bounds"`
}

type nestedOutput struct {
	Policy policy `json:"policy"`
}

func envelope(fixture, operation string, payload any) map[string]any {
	return map[string]any{
		"fixture_id":  fixture,
		"operation_id": operation,
		"payload":       payload,
		"schema":        protocolSchema,
	}
}

func writeJSON(writer io.Writer, value any) error {
	// Normalize through interface-valued maps so encoding/json's deterministic
	// map-key ordering applies recursively rather than preserving Go struct
	// declaration order. The protocol excludes floats, matching the native
	// integer-only RFC-8785 control vocabulary.
	raw, err := json.Marshal(value)
	if err != nil {
		return err
	}
	var normalized any
	decoder := json.NewDecoder(bytes.NewReader(raw))
	decoder.UseNumber()
	if err := decoder.Decode(&normalized); err != nil {
		return err
	}
	encoded, err := json.Marshal(normalized)
	if err != nil {
		return err
	}
	_, err = writer.Write(encoded)
	return err
}

func frame(message proto.Message) ([]byte, error) {
	payload, err := proto.MarshalOptions{Deterministic: true}.Marshal(message)
	if err != nil {
		return nil, err
	}
	if len(payload) > maxFrame {
		return nil, fmt.Errorf("protobuf payload exceeds %d bytes", maxFrame)
	}
	prefix := make([]byte, binary.MaxVarintLen64)
	n := binary.PutUvarint(prefix, uint64(len(payload)))
	return append(prefix[:n], payload...), nil
}

func readFrame(message proto.Message) error {
	input, err := io.ReadAll(io.LimitReader(os.Stdin, maxFrame+1))
	if err != nil {
		return err
	}
	if len(input) == 0 {
		return errors.New("empty protobuf input")
	}
	if len(input) > maxFrame {
		return fmt.Errorf("stdin exceeds %d bytes", maxFrame)
	}
	length, n := binary.Uvarint(input)
	if n <= 0 {
		return errors.New("invalid protobuf length prefix")
	}
	if length > maxFrame || uint64(len(input)-n) != length {
		return errors.New("invalid protobuf frame length or trailing bytes")
	}
	return proto.UnmarshalOptions{DiscardUnknown: false}.Unmarshal(input[n:], message)
}

func parseDecision(value string) (sourceDecision, error) {
	var result sourceDecision
	if err := json.Unmarshal([]byte(value), &result); err != nil {
		return "", err
	}
	if result != decisionDeny && result != decisionAllow {
		return "", fmt.Errorf("invalid source decision %q", result)
	}
	return result, nil
}

func encodeDecision(value sourceDecision) *nativepb.E01Carrier {
	decision := nativepb.DecisionCarrier_DECISION_CARRIER_DENY
	if value == decisionAllow {
		decision = nativepb.DecisionCarrier_DECISION_CARRIER_ALLOW
	}
	return &nativepb.E01Carrier{Decision: decision}
}

func decodeDecision(value *nativepb.E01Carrier) (sourceDecision, error) {
	switch value.Decision {
	case nativepb.DecisionCarrier_DECISION_CARRIER_DENY:
		return decisionDeny, nil
	case nativepb.DecisionCarrier_DECISION_CARRIER_ALLOW:
		return decisionAllow, nil
	default:
		return "", errors.New("malformed E01 carrier: unspecified or unknown decision")
	}
}

func validateBound(value int32) error {
	if value < 0 || value > 2 {
		return fmt.Errorf("E02 bounded integer %d is outside 0..=2", value)
	}
	return nil
}

func validateNested(value nestedOutput) error {
	if err := validateBound(value.Policy.Bounds.Minimum); err != nil {
		return err
	}
	return validateBound(value.Policy.Bounds.Maximum)
}

func encodeBounds(value nestedOutput) *nativepb.E02Carrier {
	return &nativepb.E02Carrier{Policy: &nativepb.PolicyCarrier{Bounds: &nativepb.BoundsCarrier{
		MinimumSlot: value.Policy.Bounds.Minimum,
		MaximumSlot: value.Policy.Bounds.Maximum,
	}}}
}

func decodeBounds(value *nativepb.E02Carrier) (nestedOutput, error) {
	if value.Policy == nil || value.Policy.Bounds == nil {
		return nestedOutput{}, errors.New("malformed E02 carrier: missing nested policy/bounds")
	}
	result := nestedOutput{Policy: policy{Bounds: bounds{
		Minimum: value.Policy.Bounds.MinimumSlot,
		Maximum: value.Policy.Bounds.MaximumSlot,
	}}}
	if err := validateNested(result); err != nil {
		return nestedOutput{}, err
	}
	return result, nil
}

func writeCarrier(fixture, operation string, carrier proto.Message, metadata map[string]any) error {
	encoded, err := frame(carrier)
	if err != nil {
		return err
	}
	if err := writeJSON(os.Stderr, envelope(fixture, operation, metadata)); err != nil {
		return err
	}
	_, err = os.Stdout.Write(encoded)
	return err
}

func run(fixture, operation, value string) error {
	switch fixture + "/" + operation {
	case "E01/encode":
		native, err := parseDecision(value)
		if err != nil {
			return err
		}
		carrier := encodeDecision(native)
		return writeCarrier(fixture, operation, carrier, map[string]any{"carrier": carrier.Decision.String(), "native": native})
	case "E01/decode":
		carrier := new(nativepb.E01Carrier)
		if err := readFrame(carrier); err != nil {
			return err
		}
		native, err := decodeDecision(carrier)
		if err != nil {
			return err
		}
		return writeJSON(os.Stdout, envelope(fixture, operation, map[string]any{"carrier": carrier.Decision.String(), "native": native}))
	case "E01/program-output":
		carrier := encodeDecision(decisionDeny)
		return writeCarrier(fixture, operation, carrier, map[string]any{"carrier": carrier.Decision.String(), "native": decisionDeny})
	case "E02/encode":
		var native nestedOutput
		if err := json.Unmarshal([]byte(value), &native); err != nil {
			return err
		}
		if err := validateNested(native); err != nil {
			return err
		}
		carrier := encodeBounds(native)
		return writeCarrier(fixture, operation, carrier, map[string]any{
			"carrier": map[string]any{"minimum_slot": carrier.Policy.Bounds.MinimumSlot, "maximum_slot": carrier.Policy.Bounds.MaximumSlot},
			"native": native,
		})
	case "E02/decode":
		carrier := new(nativepb.E02Carrier)
		if err := readFrame(carrier); err != nil {
			return err
		}
		native, err := decodeBounds(carrier)
		if err != nil {
			return err
		}
		return writeJSON(os.Stdout, envelope(fixture, operation, map[string]any{
			"carrier": map[string]any{"minimum_slot": carrier.Policy.Bounds.MinimumSlot, "maximum_slot": carrier.Policy.Bounds.MaximumSlot},
			"native": native,
		}))
	case "E02/program-output":
		native := nestedOutput{Policy: policy{Bounds: bounds{Minimum: 0, Maximum: 2}}}
		carrier := encodeBounds(native)
		return writeCarrier(fixture, operation, carrier, map[string]any{
			"carrier": map[string]any{"minimum_slot": int32(0), "maximum_slot": int32(2)},
			"native": native,
		})
	default:
		return fmt.Errorf("unsupported fixture/operation %s/%s", fixture, operation)
	}
}

func main() {
	fixture := flag.String("fixture", "", "E01 or E02")
	operation := flag.String("operation", "", "encode, decode, or program-output")
	value := flag.String("value", "", "canonical native JSON for encode")
	flag.Parse()
	if flag.NArg() != 0 {
		fmt.Fprint(os.Stderr, `{"schema":"gluerift.native-protocol/v1","status":"malformed-message","error":"unexpected positional arguments"}`)
		os.Exit(2)
	}
	if err := run(*fixture, *operation, *value); err != nil {
		_ = writeJSON(os.Stderr, map[string]any{"error": err.Error(), "schema": protocolSchema, "status": "malformed-message"})
		os.Exit(2)
	}
}
