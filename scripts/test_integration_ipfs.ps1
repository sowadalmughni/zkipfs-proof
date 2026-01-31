# test_integration_ipfs.ps1
$ErrorActionPreference = "Stop"

function Test-Integration {
    Write-Host "Starting Integration Test..." -ForegroundColor Cyan

    # 1. Create a dummy file
    $TestContent = "Hello, IPFS and ZK!"
    $TestFile = "test_data.txt"
    $TestContent | Out-File -FilePath $TestFile -Encoding utf8
    
    Write-Host "Created test file: $TestFile"

    # 2. Check if server is running (Assume running on 3000)
    try {
        $health = Invoke-RestMethod -Uri "http://localhost:3000/health" -Method Get
        if ($health -ne "OK") {
            throw "Server is not healthy"
        }
        Write-Host "Server is up and healthy." -ForegroundColor Green
    } catch {
        Write-Error "Server is not reachable on http://localhost:3000. Please start 'cargo run -p zkipfs-proof-server'"
        return
    }

    # 3. Submit Job
    Write-Host "Submitting proof generation job..."
    $uri = "http://localhost:3000/generate"
    
    $form = @{
        file = Get-Item -Path $TestFile
        content_selection = "pattern:Hello"
        security_level = "128"
    }
    
    # PowerShell 7+ supports -Form. For older versions, this is harder, assuming PS7 or using curl.
    # Using 'curl' (aliased to Invoke-WebRequest usually, but real curl if installed) is safer cross-platform if available.
    # Let's try native PowerShell Invoke-RestMethod with Form (PS 7+).
    # Fallback to curl if fails.

    try {
         $response = Invoke-RestMethod -Uri $uri -Method Post -Form $form
    } catch {
         Write-Warning "PowerShell Invoke-RestMethod -Form failed (requires Core 7+). Trying curl..."
         $responseJson = curl -F "file=@$TestFile" -F "content_selection=pattern:Hello" -F "security_level=128" $uri
         $response = $responseJson | ConvertFrom-Json
    }

    $jobId = $response.job_id
    Write-Host "Job Submitted. ID: $jobId" -ForegroundColor Yellow

    # 4. Poll Status
    $status = "Pending"
    while ($status -eq "Pending" -or $status -eq "Processing") {
        Start-Sleep -Seconds 1
        $jobState = Invoke-RestMethod -Uri "http://localhost:3000/status/$jobId" -Method Get
        $status = $jobState.status
        Write-Host "Job Status: $status"
        
        # Determine status from enum struct {"status": "...", "result/message": "..."}
        # In rust: #[serde(tag = "status", content = "result")]
        # JSON: { "status": "Processing" } or { "status": "Completed", "result": {...} }
        
        $statusKey = $jobState.status
        if ($statusKey -eq "Completed") {
             Write-Host "Job Completed Successfully!" -ForegroundColor Green
             Write-Host "Proof ID: $($jobState.result.id)"
             break
        } elseif ($statusKey -eq "Failed") {
             Write-Error "Job Failed: $($jobState.result)"
        }
    }

    # Cleanup
    Remove-Item $TestFile
    Write-Host "Integration Test Passed!" -ForegroundColor Green
}

Test-Integration
