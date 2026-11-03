"""Fail-closed PQC placeholder until a reviewed signature verifier is wired.

Hashes and signature lengths cannot authenticate ML-DSA or classical signatures.
"""


class HybridPQCVerifier:
    def __init__(self, mode="ML-DSA-65"):
        self.mode = mode

    def verify_hybrid(self, classical_sig: bytes, pqc_sig: bytes,
                      message: bytes, pqc_pubkey: bytes) -> bool:
        # Block all acceptance rather than pretending that the SHA3 digest
        # of unverified signature bytes proves authenticity.
        return False
