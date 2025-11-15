#!/usr/bin/env python3
import argparse, base64, hmac, hashlib, json, time
ap=argparse.ArgumentParser()
ap.add_argument("--sub", required=True)
ap.add_argument("--aud", required=True)
ap.add_argument("--scopes", nargs="+", required=True)
ap.add_argument("--secret", required=True)
ap.add_argument("--out", required=True)
a=ap.parse_args()
hdr={"alg":"HS256","typ":"FLEXCAP"}
now=int(time.time())
pay={"sub":a.sub,"aud":a.aud,"scopes":a.scopes,"iat":now,"exp":now+3600}
b64=lambda b: base64.urlsafe_b64encode(json.dumps(b,separators=(",",":")).encode()).rstrip(b"=").decode()
msg=f"{b64(hdr)}.{b64(pay)}".encode()
sig=base64.urlsafe_b64encode(hmac.new(a.secret.encode(), msg, hashlib.sha256).digest()).rstrip(b"=").decode()
open(a.out,"w").write(f"{msg.decode()}.{sig}\n")
print("[cap_token] wrote", a.out)
