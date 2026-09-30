param(
    [Parameter(Mandatory = $true)]
    [ValidateSet('Install', 'Launch', 'Verify')]
    [string]$Step,
    [Parameter(Mandatory = $true)]
    [string]$DataDirectory,
    [string]$PackagePath,
    [string]$InstallDirectory,
    [string]$CompletedId,
    [string]$RejectedId
)

$ErrorActionPreference = 'Stop'
$projectDirectory = Split-Path -Parent $PSScriptRoot
$fixtureRoot = [IO.Path]::GetFullPath((Join-Path $projectDirectory 'target/desktop-smoke'))

function Assert-FixturePath([string]$Candidate) {
    if (-not [IO.Path]::IsPathRooted($Candidate)) { throw 'Use an explicit absolute smoke path.' }
    $resolved = [IO.Path]::GetFullPath($Candidate)
    if (-not $resolved.StartsWith($fixtureRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Smoke paths must be below this project target/desktop-smoke directory.'
    }
    return $resolved
}

$dataPath = Assert-FixturePath $DataDirectory
if (-not $InstallDirectory) { $InstallDirectory = Join-Path $fixtureRoot 'installed' }
$installPath = Assert-FixturePath $InstallDirectory
$executable = Join-Path $installPath 'local-stack-proof-desktop.exe'

switch ($Step) {
    'Install' {
        if (-not $PackagePath) { throw 'Provide the exact project-built NSIS package path.' }
        $package = (Resolve-Path -LiteralPath $PackagePath).Path
        $bundleRoot = [IO.Path]::GetFullPath((Join-Path $projectDirectory 'target/release/bundle/nsis'))
        if (-not $package.StartsWith($bundleRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase) -or [IO.Path]::GetExtension($package) -ne '.exe') {
            throw 'Only this project locally built NSIS package is allowed.'
        }
        if (Test-Path -LiteralPath $installPath) { throw 'Choose a fresh isolated install directory; no overwrites are performed by this script.' }
        # NSIS requires /D last. No user profile, data, container, or old installation is deleted.
        $installer = Start-Process -FilePath $package -ArgumentList @('/S', ('/D=' + $installPath)) -WindowStyle Hidden -Wait -PassThru
        if ($installer.ExitCode -ne 0 -or -not (Test-Path -LiteralPath $executable)) { throw 'Local NSIS installation failed.' }
        Write-Output ('Installed project package: ' + $executable)
    }
    'Launch' {
        if (-not (Test-Path -LiteralPath $executable)) { throw 'Install the exact package first.' }
        New-Item -ItemType Directory -Path $dataPath -Force | Out-Null
        $previousOverride = $env:LOCAL_STACK_PROOF_DATA_DIR
        try {
            $env:LOCAL_STACK_PROOF_DATA_DIR = $dataPath
            # Visible intentionally: this is the interactive app under test, not a background helper.
            $application = Start-Process -FilePath $executable -WindowStyle Normal -PassThru
            Write-Output ('Native app PID: ' + $application.Id)
            Write-Output 'Use the UI to create/execute default seed 42/north/0 success and forced-rejection runs. Close and Launch again with the same data path.'
        } finally {
            if ($null -eq $previousOverride) { Remove-Item Env:LOCAL_STACK_PROOF_DATA_DIR -ErrorAction SilentlyContinue }
            else { $env:LOCAL_STACK_PROOF_DATA_DIR = $previousOverride }
        }
    }
    'Verify' {
        if (-not $CompletedId -or -not $RejectedId) { throw 'Supply the exact IDs observed in the UI.' }
        Push-Location $projectDirectory
        try {
            & cargo run -p local-stack-proof-persistence-sqlite --bin sqlite-proof -- (Join-Path $dataPath 'runs.sqlite3') $CompletedId $RejectedId
            if ($LASTEXITCODE -ne 0) { throw 'Read-only SQLite evidence check failed.' }
        } finally { Pop-Location }
    }
}
