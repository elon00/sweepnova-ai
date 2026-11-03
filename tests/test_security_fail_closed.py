"""Negative security regressions. PASS does not mean live integrations work."""
import base64
import json
import os
import unittest
from unittest.mock import patch

from services.quicknode.provider import QuickNodeProvider
from services.x402.bazaar_gateway import X402BazaarGateway
from services.pqc.hybrid_pqc_verifier import HybridPQCVerifier
from services.rsi.raven_rsi_engine import RavenRSIEngine


class FailClosedRegressions(unittest.TestCase):
    def test_fake_x402_proof_never_unlocks_payment(self):
        gateway = X402BazaarGateway("r1", "1000000", "recipient")
        challenge = gateway.create_402_challenge()
        forged = {"nonce": challenge["nonce"], "expiresAt": challenge["expiresAt"],
                  "txHash": "0x" + "a" * 64}
        proof = base64.b64encode(json.dumps(forged).encode()).decode()
        accepted, message = gateway.verify_proof(proof, challenge["nonce"])
        self.assertFalse(accepted)
        self.assertIn("BLOCKED", message)
        self.assertFalse(gateway.verify_proof(proof, challenge["nonce"])[0])

    def test_pqc_does_not_accept_length_only(self):
        verifier = HybridPQCVerifier()
        self.assertFalse(verifier.verify_hybrid(b"c" * 64, b"p" * 3309,
                                                 b"message", b"k" * 1952))
        self.assertFalse(verifier.verify_hybrid(b"", b"", b"", b""))

    def test_unconfigured_rpc_never_reports_healthy(self):
        with patch.dict(os.environ, {}, clear=True):
            provider = QuickNodeProvider()
            self.assertFalse(provider.verify_health())
            self.assertEqual(provider.health_status, "UNVERIFIED")

    def test_fallback_requires_chain_identity(self):
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaises(ValueError):
                QuickNodeProvider(primary_url="https://example.com/rpc",
                                  fallback_url="https://backup.example.com/rpc")

    def test_invalid_rpc_url_fails_closed(self):
        with patch.dict(os.environ, {}, clear=True):
            provider = QuickNodeProvider(primary_url="http://example.com/rpc")
            self.assertFalse(provider.verify_health())

    def test_raven_cannot_fabricate_test_runs(self):
        record = RavenRSIEngine("sweepnova-ai").execute_governed_cycle("latency", 8, 5)
        self.assertEqual(record["test"], "NOT_RUN")
        self.assertEqual(record["review"], "NOT_APPROVED")
        self.assertEqual(record["status"], "BLOCKED")


if __name__ == "__main__":
    unittest.main()
