#!/usr/bin/env python3
import os, sys
try:
    import capnp  # pycapnp
except Exception as e:
    print("[py-client] pycapnp not installed. `pip install pycapnp` to use this client.")
    sys.exit(0)
IDL_PATH = os.path.join(os.path.dirname(__file__), "..", "contracts", "inference.capnp")
capnp.remove_import_hook()
inference = capnp.load(IDL_PATH)
def main():
    sock = "unix:/tmp/flex_infer.sock"
    client = capnp.TwoPartyClient(sock)
    srv = client.bootstrap().cast_as(inference.Inference)
    req = srv.predict_request()
    req.req = {"input": b"hello pycapnp", "traceId": "py-smoke-1"}
    rep = req.send().wait().rep
    print("[py-client] modelHash=", rep.modelHash, " bytes=", len(rep.output))
if __name__ == "__main__":
    main()
