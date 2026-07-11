$ErrorActionPreference = "Stop"
$repoRoot = Split-Path $PSScriptRoot -Parent
Set-Location $repoRoot

$owner = "tanvoid0"
$repo = "portal-desktop"
$projectTitle = "Portal Desktop Roadmap"

# Verify project scope
$status = gh auth status 2>&1 | Out-String
if ($status -notmatch "project") {
    Write-Host "Missing 'project' scope. Run:" -ForegroundColor Yellow
    Write-Host "  gh auth refresh -h github.com -s read:project,project" -ForegroundColor Cyan
    exit 1
}

# Reuse existing project if present
$existing = gh project list --owner $owner --format json | ConvertFrom-Json
$project = $existing.projects | Where-Object { $_.title -eq $projectTitle } | Select-Object -First 1

if ($project) {
    $projectNumber = $project.number
    Write-Host "Using existing project #$projectNumber"
} else {
    gh project create --owner $owner --title $projectTitle | Out-Null
    $existing = gh project list --owner $owner --format json | ConvertFrom-Json
    $project = $existing.projects | Where-Object { $_.title -eq $projectTitle } | Select-Object -First 1
    $projectNumber = $project.number
    Write-Host "Created project #$projectNumber"
}

gh project link $projectNumber --owner $owner --repo $repo | Out-Null
Write-Host "Linked $repo repository"

$onProject = gh project item-list $projectNumber --owner $owner --format json | ConvertFrom-Json
$existingNumbers = @($onProject.items | ForEach-Object { $_.content.number })

$issues = gh issue list --repo "$owner/$repo" --state open --limit 100 --json number | ConvertFrom-Json
foreach ($issue in $issues) {
    if ($existingNumbers -contains $issue.number) {
        Write-Host "  Skipped #$($issue.number) (already on board)"
        continue
    }
    $url = "https://github.com/$owner/$repo/issues/$($issue.number)"
    gh project item-add $projectNumber --owner $owner --url $url | Out-Null
    Write-Host "  Added #$($issue.number)"
}

$view = gh project view $projectNumber --owner $owner --format json | ConvertFrom-Json
Write-Host ""
Write-Host "Done! Project board:" -ForegroundColor Green
Write-Host "  $($view.url)" -ForegroundColor Cyan
Write-Host ""
Write-Host "Tip: Switch to Board view and group by Milestone or Priority" -ForegroundColor DarkGray
