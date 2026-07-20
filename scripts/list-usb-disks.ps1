param(
    [switch]$IncludeAll
)

$ErrorActionPreference = "Stop"

$disks = Get-Disk | Sort-Object Number
if (-not $IncludeAll) {
    $disks = $disks | Where-Object { $_.BusType -eq "USB" }
}

$disks |
    Select-Object `
        Number,
        FriendlyName,
        SerialNumber,
        BusType,
        PartitionStyle,
        OperationalStatus,
        IsBoot,
        IsSystem,
        @{Name = "SizeGB"; Expression = { [math]::Round($_.Size / 1GB, 2) } } |
    Format-Table -AutoSize
