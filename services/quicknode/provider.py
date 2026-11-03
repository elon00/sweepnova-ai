"""Real Solana JSON-RPC provider. Fail closed unless an endpoint is configured.

A successful health check is never treated as evidence of transaction finality.
"""
import json
import os
import secrets
import urllib.parse
import urllib.request


class QuickNodeProvider:
    def __init__(self, network="solana-mainnet", primary_url=None, fallback_url=None,
                 expected_genesis_hash=None, timeout_seconds=10):
        self.network = network
        self.primary_url = primary_url or os.environ.get("QUICKNODE_RPC_URL")
        self.fallback_url = fallback_url or os.environ.get("QUICKNODE_FALLBACK_RPC_URL")
        self.expected_genesis_hash = expected_genesis_hash or os.environ.get("SOLANA_GENESIS_HASH")
        self.timeout_seconds = timeout_seconds
        self.current_endpoint = self.primary_url
        self.health_status = "UNVERIFIED"
        if self.fallback_url and not self.expected_genesis_hash:
            raise ValueError("Fallback requires SOLANA_GENESIS_HASH to prevent cross-chain failover")

    @staticmethod
    def _valid_url(url):
        if not url:
            raise RuntimeError("QuickNode RPC not configured: set QUICKNODE_RPC_URL")
        parsed = urllib.parse.urlparse(url)
        if parsed.scheme != "https" or not parsed.hostname or parsed.username or parsed.password:
            raise ValueError("QuickNode RPC endpoint must be a valid HTTPS URL")

    def _rpc(self, url, method, params):
        self._valid_url(url)
        request_id = secrets.token_hex(12)
        payload = json.dumps({"jsonrpc": "2.0", "id": request_id, "method": method,
                              "params": params}).encode("utf-8")
        req = urllib.request.Request(url, data=payload,
                    headers={"Content-Type": "application/json"}, method="POST")
        with urllib.request.urlopen(req, timeout=self.timeout_seconds) as res:
            if res.status != 200:
                raise RuntimeError("JSON-RPC non-success HTTP status")
            response = json.loads(res.read(1048576).decode("utf-8"))
        if (not isinstance(response, dict) or response.get("jsonrpc") != "2.0"
                or response.get("id") != request_id or "error" in response
                or "result" not in response):
            raise RuntimeError("Invalid or unsuccessful JSON-RPC response")
        return response

    def request(self, method: str, params=None):
        if not isinstance(method, str) or not method or method.startswith("_"):
            raise ValueError("Invalid RPC method")
        if params is None:
            params = []
        if not isinstance(params, list):
            raise TypeError("JSON-RPC params must be a list")
        endpoints = [self.primary_url]
        if self.fallback_url:
            endpoints.append(self.fallback_url)
        errors = []
        for endpoint in endpoints:
            try:
                self._valid_url(endpoint)
                if self.expected_genesis_hash:
                    genesis = self._rpc(endpoint, "getGenesisHash", [])["result"]
                    if genesis != self.expected_genesis_hash:
                        raise RuntimeError("Solana genesis hash mismatch")
                response = self._rpc(endpoint, method, params)
                self.current_endpoint = endpoint
                return response
            except (OSError, ValueError, RuntimeError, json.JSONDecodeError) as exc:
                # Deliberately redact URLs and tokens from exception messages.
                errors.append(type(exc).__name__)
        raise RuntimeError("Real JSON-RPC request failed: " + ", ".join(errors))

    def verify_health(self):
        try:
            response = self.request("getHealth")
            self.health_status = "HEALTHY" if response.get("result") == "ok" else "UNHEALTHY"
        except (ValueError, RuntimeError, OSError):
            self.health_status = "UNVERIFIED"
        return self.health_status == "HEALTHY"
