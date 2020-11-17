# Pathnames
Set-Item -Path Env:TRACKER_D_BASE -Value $(Join-Path "C:" "$Env:HOMEPATH\AppData\Local\tracker")
Set-Item -Path Env:TRACKER_LOG -Value "$Env:TRACKER_D_BASE\tracker.log"
Set-Item -Path Env:TRACKER_HISTLOGS -Value "$Env:TRACKER_D_BASE\histlogs"

# init files/dirs
If (-Not (Test-Path -Path "$Env:TRACKER_HISTLOGS")){
    New-Item "$Env:TRACKER_HISTLOGS" -ItemType Directory -Force | Out-Null
}
Set-Item -Path Env:TRACKER_HISTLOG -Value "$Env:TRACKER_HISTLOGS\history.log"

function TrckrExit() {
    Stop-Transcript
    Remove-Item -Path "$Env:TRACKER_HISTLOG"
    Write-Host "[* TRACKER] Exit"
    exit
}
# $null = Register-EngineEvent -SourceIdentifier PowerShell.Exiting -Action { TrckrExit }

Start-Transcript -Path "$Env:TRACKER_HISTLOG" -IncludeInvocationHeader
Write-Output "[* TRACKER] Please run 'TrckrExit' at the end of the session."
