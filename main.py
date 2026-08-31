import asyncio
import os
import subprocess
from datetime import datetime

from fastapi import FastAPI, HTTPException, Request
from pydantic import BaseModel
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address


# -------------------------------
# Models
# -------------------------------
class ClientKey(BaseModel):
    apiKey: str
    clientPubKey: str
    clientVersion: float


class ServerConf(BaseModel):
    serverKey: str
    address: str
    allowedIps: list[str]
    remoteIp: str
    endpoint: str
    presKeepAlive: int
    dns: str
    wstunnelRemotePort: str


class UnreachableReport(BaseModel):
    process: str
    ips: list[str]


# -------------------------------
# Global state
# -------------------------------
latestAddress = "172.16.0.2"
connected_users_count = 0  # cached connected users
config_version = 1
requiredVersion = 2.4  # minimum client app version allowed to connect
# -------------------------------
# Static config
# -------------------------------
apiKeyTest = "z~WXkukTav2^dodr5#9"
serverKey = "d/TPniwj2smSddmCn+ExUUCjl8aHSitRc3n7ZTt5EBc="
allowedIps = [
    "20.0.0.0/8",
    "63.251.140.0/24",
    "69.25.124.0/24",
    "69.25.0.0/16",
    "70.42.0.0/16",
    "74.201.0.0/16",
    "188.42.0.0/16",
    "216.52.0.0/16",
    "85.0.0.0/8",
    "104.29.147.0/24",
]
remoteIp = "57.131.34.226"
endpoint = "localhost:51820"
presKeepAlive = 25
dns = "8.8.8.8"
wstunnelRemotePort = "443"


# -------------------------------
# Helpers
# -------------------------------
def getNextAddress():
    global latestAddress
    addressInt = list(map(int, latestAddress.split(".")))
    if addressInt[3] == 254:
        if addressInt[2] == 255:
            return None
        addressInt[2] += 1
        addressInt[3] = 1
    else:
        addressInt[3] += 1

    latestAddress = ".".join(map(str, addressInt))
    return latestAddress


def fetch_connected_users():
    """Run the wg command and return the connected user count"""
    try:
        result = subprocess.run(
            [
                "sudo",
                "bash",
                "-c",
                'wg show wg0 latest-handshakes | awk -v now="$(date +%s)" '
                "'$2 > 0 && (now - $2 < 120) { count++ } END { print count+0 }'",
            ],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            check=True,
        )
        return int(result.stdout.strip())
    except Exception:
        return 0


async def update_connected_users():
    """Background task to update connected user count every 2 minutes"""
    global connected_users_count
    while True:
        connected_users_count = fetch_connected_users()
        await asyncio.sleep(120)


# -------------------------------
# App setup
# -------------------------------
app = FastAPI(redirect_slashes=False)
limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)


@app.on_event("startup")
async def startup_event():
    # Start background task
    asyncio.create_task(update_connected_users())


# -------------------------------
# Routes
# -------------------------------
@app.post("/exchange_keys/")
@limiter.limit("5/hour")
async def exchange(clientData: ClientKey, request: Request):
    if clientData.apiKey != apiKeyTest:
        raise HTTPException(status_code=401, detail="Invalid API Key")
    if clientData.clientVersion < requiredVersion:
        raise HTTPException(
            status_code=409,
            detail=f"Outdated client version. Please update the app (required version {requiredVersion})",
        )
    adr = getNextAddress()
    if adr is None:
        raise HTTPException(status_code=500, detail="No more IP addresses available")
    adr += "/32"

    if (
        len(clientData.clientPubKey) != 44
        or not clientData.clientPubKey.endswith("=")
        or " " in clientData.clientPubKey
    ):
        raise HTTPException(status_code=400, detail="Invalid public key format")

    serverconf = ServerConf(
        serverKey=serverKey,
        address=adr,
        allowedIps=allowedIps,
        remoteIp=remoteIp,
        endpoint=endpoint,
        presKeepAlive=presKeepAlive,
        dns=dns,
        wstunnelRemotePort=wstunnelRemotePort,
    )
    subprocess.run(
        ["sudo", "tee", "-a", "/etc/wireguard/wg0.conf"],
        input=f"[Peer]\nPublicKey = {clientData.clientPubKey}\nAllowedIPs = {adr}\n\n",
        text=True,
        check=True,
    )
    subprocess.run(["sudo", "systemctl", "reload", "wg-quick@wg0"], check=True)
    return serverconf


@app.get("/connected_users/")
@limiter.limit("4/minute")
async def get_connected_users(request: Request):
    return {"connected_users": connected_users_count}


@app.get("/config_version/")
async def get_config_version():
    return {"config_version": config_version}


@app.put("/unreachable_report/")
async def report_unreachable(report: UnreachableReport):
    if not report.process.strip():
        raise HTTPException(status_code=400, detail="process name is required")
    if any(not ip.strip() for ip in report.ips):
        raise HTTPException(status_code=400, detail="ips must be non-empty strings")

    # Persist a per-process report file (one per process name, appended with a
    # timestamp on each new report so history is kept).
    report_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "reports")
    os.makedirs(report_dir, exist_ok=True)
    safe_name = (
        "".join(c for c in report.process.strip() if c.isalnum() or c in "._- ")
        or "unknown"
    )
    file_path = os.path.join(report_dir, f"unreachable_{safe_name}.txt")

    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
    with open(file_path, "a", encoding="utf-8") as f:
        f.write(f"[{timestamp}] {report.process}\n")
        for ip in report.ips:
            f.write(f"    {ip}\n")

    return {"received": True, "process": report.process, "endpoints": len(report.ips)}
