param(
    [string] $InstallDir = $(Join-Path $env:LOCALAPPDATA "now\bin")
)

$ErrorActionPreference = "Stop"

$Repo = "doggy8088/now"
$BinaryName = "now"
$Archive = "now-x86_64-pc-windows-msvc.zip"
$BaseUrl = "https://github.com/$Repo/releases/latest/download"
$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("now-install-" + [System.Guid]::NewGuid().ToString("N"))

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ArchivePath = Join-Path $TempDir $Archive
    $ChecksumPath = "$ArchivePath.sha256"

    Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile $ArchivePath
    Invoke-WebRequest -Uri "$BaseUrl/$Archive.sha256" -OutFile $ChecksumPath

    $Expected = ((Get-Content $ChecksumPath -Raw).Trim() -split "\s+")[0].ToLowerInvariant()
    if ($Expected -notmatch "^[a-f0-9]{64}$") {
        throw "Invalid checksum file format"
    }

    $Actual = (Get-FileHash -Algorithm SHA256 -Path $ArchivePath).Hash.ToLowerInvariant()
    if ($Actual -ne $Expected) {
        throw "Checksum mismatch for $Archive"
    }

    Expand-Archive -Force -Path $ArchivePath -DestinationPath $TempDir
    $Binary = Get-ChildItem -Path $TempDir -Filter "$BinaryName.exe" -Recurse -File | Select-Object -First 1
    if (-not $Binary) {
        throw "Archive did not contain $BinaryName.exe"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    Copy-Item -Force -Path $Binary.FullName -Destination (Join-Path $InstallDir "$BinaryName.exe")

    Write-Output "Installed $(Join-Path $InstallDir "$BinaryName.exe")"

    $PathParts = ($env:PATH -split ";") | Where-Object { $_ -ne "" }
    if ($PathParts -notcontains $InstallDir) {
        Write-Output "Add $InstallDir to PATH to run $BinaryName from any directory."
    }
}
finally {
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $TempDir
}
