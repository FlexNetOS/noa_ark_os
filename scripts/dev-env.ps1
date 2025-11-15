param(
    [Parameter(Position = 0)]
    [string]$Command = "help",
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Remaining
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot ".."))
$ManifestPath = Join-Path $RepoRoot "tools/devshell/dev-env.manifest.toml"
$DefaultImageTag = "noa-ark-os/dev-env:latest"
$script:Runtime = $env:CONTAINER_RUNTIME

function Show-Usage {
    @"
Usage: scripts/dev-env.ps1 [command] [args]

Commands:
  build        Build or rebuild the developer container image from the manifest.
  run [cmd]    Start an interactive shell (default) or run [cmd] inside the container.
  smoke        Run a smoke test that validates core tooling inside the container.
  manifest     Print the manifest as formatted JSON.
  help         Show this message.

Environment variables:
  CONTAINER_RUNTIME  Override container runtime (docker or podman). Defaults to podman if available.
  DEV_ENV_IMAGE_TAG  Override the image tag used for build/run commands.
"@
}

function Invoke-Python {
    param(
        [Parameter(Mandatory = $true)][string]$Script,
        [string[]]$Arguments
    )

    $python = Get-Command python -ErrorAction Stop
    $tempFile = New-TemporaryFile
    try {
        Set-Content -Path $tempFile -Value $Script -Encoding UTF8
        & $python.Source $tempFile @Arguments
        if ($LASTEXITCODE -ne 0) {
            throw "Python script failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Remove-Item -Force $tempFile -ErrorAction SilentlyContinue
    }
}

function Get-ManifestJson {
    Invoke-Python -Script @"
import json
import tomllib
import sys
from pathlib import Path
path = Path(sys.argv[1])
if not path.exists():
    raise SystemExit(f"Manifest not found: {path}")
with path.open('rb') as fh:
    data = tomllib.load(fh)
json.dump(data, sys.stdout, indent=2)
"@ -Arguments @($ManifestPath)
}

function Get-Dockerfile {
    Invoke-Python -Script @"
import tomllib
import textwrap
import sys
from pathlib import Path
manifest = tomllib.loads(Path(sys.argv[1]).read_text())
image = manifest['image']['name']
user = manifest['user']
node = manifest['node']
rust = manifest['rust']
packages = manifest.get('packages', {}).get('apt', [])
post_create = manifest.get('post_create', {}).get('script')
apt_packages = ' '.join(packages)
node_setup_script = 'setup_lts.x'
node_desc = node.get('version', 'lts')
if node_desc.lower() != 'lts':
    node_setup_script = f"setup_{node_desc}.x"
enable_pnpm = node.get('enable_pnpm', False)
rust_profile = rust.get('profile', 'minimal')
rust_toolchain = rust.get('toolchain', 'stable')
lines = [
    f"FROM {image}",
    "",
    f"ARG USERNAME={user['name']}",
    f"ARG USER_UID={user['uid']}",
    f"ARG USER_GID={user['gid']}",
    "",
    "RUN groupadd --gid ${USER_GID} ${USERNAME} \\",
    "    && useradd --uid ${USER_UID} --gid ${USER_GID} -m ${USERNAME}",
    "",
    "RUN apt-get update \\",
    "    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \\",
    "       ca-certificates curl gnupg" + (f" {apt_packages}" if apt_packages else "") + " \\",
    "    && rm -rf /var/lib/apt/lists/*",
    "",
    f"RUN curl -fsSL https://deb.nodesource.com/{node_setup_script} | bash - \\",
    "    && apt-get update \\",
    "    && DEBIAN_FRONTEND=noninteractive apt-get install -y nodejs \\",
    "    && rm -rf /var/lib/apt/lists/*",
]
if enable_pnpm:
    lines.append("RUN corepack enable pnpm")
else:
    lines.append("RUN corepack enable")
lines.extend([
    textwrap.dedent(f"""
    RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \\
        sh -s -- -y --default-toolchain {rust_toolchain} --profile {rust_profile}
    ENV PATH="/home/${{USERNAME}}/.cargo/bin:$PATH"
    RUN /bin/bash -lc "rustup component add rustfmt clippy"
    """).strip(),
])
lines.append("RUN mkdir -p /opt/devcontainer")
if post_create:
    lines.extend([
        f"COPY {post_create} /opt/devcontainer/post-create.sh",
        "RUN chmod +x /opt/devcontainer/post-create.sh",
    ])
lines.extend([
    "COPY tools/devshell/entrypoint.sh /opt/devcontainer/entrypoint.sh",
    "RUN chmod +x /opt/devcontainer/entrypoint.sh",
    "USER ${USERNAME}",
    "WORKDIR /workspace",
    "ENTRYPOINT [\"/opt/devcontainer/entrypoint.sh\"]",
    "CMD [\"bash\"]",
])
print('\n'.join(lines))
"@ -Arguments @($ManifestPath)
}

function Get-Runtime {
    if ($script:Runtime) { return $script:Runtime }
    if (Get-Command podman -ErrorAction SilentlyContinue) { $script:Runtime = "podman" }
    elseif (Get-Command docker -ErrorAction SilentlyContinue) { $script:Runtime = "docker" }
    else { throw "Neither podman nor docker is available in PATH." }
    return $script:Runtime
}

function Get-ImageTag {
    if ($env:DEV_ENV_IMAGE_TAG) { return $env:DEV_ENV_IMAGE_TAG }
    return $DefaultImageTag
}

function Invoke-Build {
    $runtime = Get-Runtime
    $dockerfile = New-TemporaryFile
    try {
        Get-Dockerfile | Set-Content -Path $dockerfile -Encoding UTF8
        $tag = Get-ImageTag
        Write-Host "[dev-env] Building image $tag with $runtime"
        & $runtime build -f $dockerfile -t $tag $RepoRoot
    }
    finally {
        Remove-Item -Force $dockerfile -ErrorAction SilentlyContinue
    }
}

function Ensure-Image {
    $runtime = Get-Runtime
    $tag = Get-ImageTag
    $null = & $runtime image inspect $tag 2>$null
    if (-not $?) {
        Write-Host "[dev-env] Image $tag not found, building first..."
        Invoke-Build
    }
}

function Invoke-Run {
    Ensure-Image
    $runtime = Get-Runtime
    $tag = Get-ImageTag
    $cmd = if ($Remaining -and $Remaining.Count -gt 0) { $Remaining } else { @("bash") }
    $uid = [System.Environment]::GetEnvironmentVariable("UID")
    if (-not $uid) { $uid = "1000" }
    $gid = [System.Environment]::GetEnvironmentVariable("GID")
    if (-not $gid) { $gid = "1000" }
    Write-Host "[dev-env] Starting container from $tag"
    & $runtime run --rm -it `
        -v "$RepoRoot:/workspace:Z" `
        -w /workspace `
        -e HOST_UID=$uid `
        -e HOST_GID=$gid `
        $tag @cmd
}

function Invoke-Smoke {
    Ensure-Image
    $runtime = Get-Runtime
    $tag = Get-ImageTag
    Write-Host "[dev-env] Running smoke test"
    & $runtime run --rm `
        -v "$RepoRoot:/workspace:Z" `
        -w /workspace `
        -e DEV_ENV_SKIP_POST_CREATE=1 `
        $tag bash -lc 'node --version && pnpm --version && rustc --version && cargo --version'
}

switch ($Command.ToLowerInvariant()) {
    "build" { Invoke-Build }
    "run" { Invoke-Run }
    "smoke" { Invoke-Smoke }
    "manifest" { Get-ManifestJson }
    "help" { Show-Usage }
    default {
        Show-Usage
        throw "Unknown command: $Command"
    }
}
