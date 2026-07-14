# GlueRift 논문 초고 외부검토 요청

검토 대상은 `output/pdf/round-trips-can-lie.pdf`이다. 구현의 유일한 설계
근거는 `ROUND-TRIPS-CAN-LIE-RESEARCH-ARTIFACT-CONTRACT-v0.3.1a.md`이며,
승인된 SHA-256은 다음과 같다.

`1b0ebee64fcb482f87e1d37bece9a5ae2fc44bac7121607f31a531ea9dcf9fc7`

## 논문이 주장하는 것

1. 여섯 adapter round trip은 실제 `TargetNativeExact` comparison의 policy
   soundness를 함의하지 않는다.
2. carrier automorphism과 inverse로 target adapter를 conjugate하면 요청된
   법칙을 모두 보존하면서 unsafe target-native equality를 만들 수 있다.
3. `Safe`와 `Match`를 분리한 soundness/adequacy 검사는 false agreement와
   always-different vacuity를 서로 구별한다.
4. 닫힌 유한 IR 안에서 GlueRift는 지원되는 요청을 exhaustive하게 판정하고,
   지원되지 않는 observer는 `unknown`으로 보고한다.
5. E01과 E02는 shared Protobuf carrier와 별도 Go/Rust process에서 ordinary
   comparator의 false agreement를 재현한다.
6. 완전한 Direct-Relation baseline BL4는 같은 top-level verdict와 첫 witness를
   낸다. GlueRift의 차이는 논리적 검출력이 아니라 twist 생성, nested path,
   derivation, carrier diagnostic, native evidence binding이다.

## 구현에 결속된 핵심 결과

- Lean L1--L9: build 및 axiom audit 통과, `sorry`/`admit` 없음.
- A01/A02/A03/A05: aligned base에서 생성된 candidate이며 여섯 법칙 통과,
  `lawful-harmful`.
- V01: 여섯 법칙 통과 상태에서 carrier/native induced relation divergence.
- T01: 두 후보는 `lawful-safe`, 합성은 법칙을 계속 통과하면서
  `lawful-harmful`.
- T02: policy soundness는 성립하지만 요청 법칙 실패로
  `law-breaking-or-inapplicable`.
- BL4: 모든 paired run의 공통 verdict 및 첫 witness parity 통과.
- E01: `DENY`, `Permitted`, transported `Permitted`, ordinary comparator
  `EQUAL`.
- E02: nested repeated-type field-role swap, witness path
  `output.policy.bounds.minimum`.
- `./artifact/reproduce`: clean regeneration 및 byte comparison 통과.

## 집중 검토 질문

1. 실제 motivating comparator를 `E_A^T`로 모델링한 construct validity가 본문에서
   충분히 명확한가?
2. Theorem 1과 Corollary 2의 total/shared-domain 전제와 구현 범위가 과장 없이
   서술됐는가?
3. lens, CycleGAN automorphism, semantic interoperability와 대비한 novelty
   boundary가 정확한가?
4. BL4 parity를 인정하면서도 남는 attack-paper contribution이 충분한가?
5. four finite attacks와 E01/E02가 SCORED research paper의 evaluation으로
   충분한가?
6. architecture-faithful reconstruction이라는 disclosure 분류가 적절한가?
7. 현재 초고를 승인할 수 없다면, 반드시 고쳐야 할 P0/P1을 문장 또는 절
   단위로 지정해 달라.
