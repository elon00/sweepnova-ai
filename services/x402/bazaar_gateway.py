"""x402 preflight challenge only. Payments stay denied until real facilitator
verification, settlement and replay journal are implemented and tested.

This is NOT a complete x402 v2 or Bazaar integration.
"""
import time
import uuid


class X402BazaarGateway:
    def __init__(self, resource_id: str, price_base_units: str, recipient: str,
                 network: str = "base"):
        self.resource_id = resource_id
        self.price = price_base_units
        self.recipient = recipient
        self.network = network
        self.catalog = {}

    def create_402_challenge(self):
        return {
            "status": 402,
            "title": "Payment Required (integration unavailable)",
            "resource": self.resource_id,
            "priceBaseUnits": self.price,
            "currency": "USDC",
            "recipient": self.recipient,
            "network": self.network,
            "nonce": str(uuid.uuid4()),
            "expiresAt": int(time.time()) + 300,
        }

    def verify_proof(self, proof_str: str, nonce: str):
        # A client-controlled Base64 JSON object / tx-hash string is never
        # proof of payment. No API request may be unlocked through this path.
        return False, ("BLOCKED: x402 facilitator verification, settlement, "
                       "replay journal and network finality are not configured")
