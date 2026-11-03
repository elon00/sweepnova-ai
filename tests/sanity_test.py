# Offline sanity tests. Do NOT claim real settlement or RPC success.
import sys
import os
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))
import base64
import json


def run_sanity_test():
    try:
        from services.x402.bazaar_gateway import X402BazaarGateway
        from services.tokenomics.uncapped_launchpad import UncappedTokenomicsEngine
        from services.wallets.qr_multiconnect import QRMultiwalletEngine
        gw = X402BazaarGateway("res_test", "500000", "0xTestVault")
        ch = gw.create_402_challenge()
        payload = {"nonce": ch["nonce"], "expiresAt": ch["expiresAt"],
                   "txHash": "0x" + "a" * 64}
        forged = base64.b64encode(json.dumps(payload).encode()).decode()
        valid, _ = gw.verify_proof(forged, ch["nonce"])
        assert not valid, "Forged x402 proof was incorrectly accepted"
        tok = UncappedTokenomicsEngine(1_000_000)
        res = tok.execute_mint_and_burn(10_000, 2_000)
        assert res["circulating"] == 1_008_000
        qr = QRMultiwalletEngine("base")
        uri = qr.generate_payment_uri("0xUser", "1000000")
        assert "ethereum:0xUser" in uri
        print("OFFLINE SANITY PASS: fake payment rejected; token math and URI checked")
        print("NOT VERIFIED: real facilitator settlement, wallet signature, RPC, actual mint")
        return True
    except Exception as exc:
        print("SANITY FAIL:", str(exc))
        return False


if __name__ == "__main__":
    if not run_sanity_test():
        sys.exit(1)
