# Reboot/physical-action driver for the focused Secret Vault profile.
# Dot-sourced only; definitions have no top-level side effects.

function Invoke-SecretVaultForgetBothViaUsb {
    $usbBefore = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"

    # Close the unlock outcome and reopen the still-focused Vault action.
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "ret"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:manage_ready" `
        -Marker "VAULT_MANAGE_READY" `
        -TimeoutSeconds $TimeoutSeconds

    # Manage defaults to Close. Right selects provider; confirmation defaults
    # to Cancel, so a second Right is required before the destructive Enter.
    Send-Rr1HmpKey -KeyName "right"
    Send-Rr1HmpKey -KeyName "ret"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:provider_forget_confirm_ready" `
        -Marker "VAULT_PROVIDER_FORGET_CONFIRM_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1HmpKey -KeyName "right"
    Send-Rr1HmpKey -KeyName "ret"
    foreach ($marker in @(
        @{ Name = "secret-vault:boot2:provider_tombstone_committed"; Text = "C1_VAULT_PROVIDER_TOMBSTONE_COMMITTED version=2 readback=verified" },
        @{ Name = "secret-vault:boot2:provider_forgotten"; Text = "VAULT_PROVIDER_FORGOTTEN version=2 readback=verified" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    # Close the provider outcome, reopen Manage (again defaulting to Close),
    # then Left selects WiFi before the same two-step confirmation.
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "left"
    Send-Rr1HmpKey -KeyName "ret"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:wifi_forget_confirm_ready" `
        -Marker "VAULT_WIFI_FORGET_CONFIRM_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1HmpKey -KeyName "right"
    Send-Rr1HmpKey -KeyName "ret"
    foreach ($marker in @(
        @{ Name = "secret-vault:boot2:wifi_tombstone_committed"; Text = "C1_VAULT_WIFI_TOMBSTONE_COMMITTED version=2 readback=verified" },
        @{ Name = "secret-vault:boot2:wifi_forgotten"; Text = "VAULT_WIFI_FORGOTTEN version=2 readback=verified" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    $usbAfter = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    $physical = $usbAfter -gt $usbBefore
    Add-Predicate `
        -Name "secret-vault:boot2:forget_actions_use_usb_hid" `
        -Expected "both trusted tombstones require physical manage and second-confirmation input" `
        -Passed $physical `
        -Actual "passed=$($physical.ToString().ToLowerInvariant()) before=$usbBefore after=$usbAfter"
    if (-not $physical) {
        throw "Secret Vault forget actions did not traverse USB HID input"
    }
}

function Restart-SecretVaultQemu {
    param([Parameter(Mandatory = $true)][string]$LogName)

    if (-not $script:QemuPid) {
        throw "secret-vault reboot requires a live QEMU process"
    }
    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid

    $nextLog = Join-Path $RunDir $LogName
    $params = $runParams.Clone()
    $params.StopExisting = $false
    $params.SerialLog = $nextLog
    $output = & $RunScript @params

    $script:SerialLog = $nextLog
    $script:SerialLogCachePath = $null
    $script:SerialLogCacheLength = [int64]-1
    $script:SerialLogCacheWriteTicks = [int64]-1
    $script:SerialLogCacheContent = $null
    $script:QemuPid = $null
    foreach ($line in $output) {
        if ($line -match '^qemu pid:\s*(\d+)') {
            $script:QemuPid = [int]$Matches[1]
        }
    }
    if (-not $script:QemuPid) {
        throw "secret-vault reboot did not return a QEMU pid"
    }
    try {
        $script:QemuProcess = Get-Process -Id $script:QemuPid -ErrorAction Stop
    }
    catch {
        $script:QemuProcess = $null
    }
    return $nextLog
}

function Invoke-SecretVaultForgottenRebootProof {
    param([Parameter(Mandatory = $true)][byte[]]$Rr1)

    Invoke-SecretVaultForgetBothViaUsb
    $boot2Log = $script:SerialLog
    $boot3Log = Restart-SecretVaultQemu -LogName "serial-secret-vault-forgotten-reboot.log"
    Assert-Rr1NotInSerial -Name "secret-vault:boot2:rr1_absent_from_serial" -Path $boot2Log

    foreach ($marker in @(
        @{ Name = "secret-vault:boot3:serial_ready"; Text = "SERIAL CONSOLE READY" },
        @{ Name = "secret-vault:boot3:usb_keyboard_ready"; Text = "usb-hid: keyboard ready on slot" },
        @{ Name = "secret-vault:boot3:complete_replay_bound"; Text = "C1_VAULT_COMPLETE_REPLAY_BOUND" },
        @{ Name = "secret-vault:boot3:core_policy_bound"; Text = "C1_VAULT_CORE_POLICY_BOUND" },
        @{ Name = "secret-vault:boot3:wrapper_replayed"; Text = "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation=1" },
        @{ Name = "secret-vault:boot3:provider_tombstone_replayed"; Text = "C1_VAULT_PROVIDER_TOMBSTONE_REPLAYED version=2" },
        @{ Name = "secret-vault:boot3:wifi_tombstone_replayed"; Text = "C1_VAULT_WIFI_TOMBSTONE_REPLAYED version=2" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    $usbBefore = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    Invoke-VaultVisibleAction -SkipFinalEnter
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot3:recovery_unlock_ready" `
        -Marker "VAULT_RECOVERY_UNLOCK_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1ViaUsbKeyboard -Rr1 $Rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:boot3:broker_unlocked" `
        -SuccessMarker "VAULT_RR1_UNLOCKED" `
        -RejectedMarker "VAULT_RR1_UNLOCK_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds
    foreach ($marker in @(
        @{ Name = "secret-vault:boot3:provider_forgotten_use_denied"; Text = "VAULT_PROVIDER_FORGOTTEN_USE_DENIED reason=secret_forgotten test_infrastructure=true" },
        @{ Name = "secret-vault:boot3:wifi_forgotten_use_denied"; Text = "VAULT_WIFI_FORGOTTEN_USE_DENIED reason=secret_forgotten test_infrastructure=true" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    $usbAfter = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    $physicalUnlock = $usbAfter -gt $usbBefore
    Add-Predicate `
        -Name "secret-vault:boot3:rr1_unlock_uses_usb_hid" `
        -Expected "forgotten-state recovery unlock still requires physical RR1 input" `
        -Passed $physicalUnlock `
        -Actual "passed=$($physicalUnlock.ToString().ToLowerInvariant()) before=$usbBefore after=$usbAfter"
    if (-not $physicalUnlock) {
        throw "Forgotten-state RR1 unlock did not traverse USB HID input"
    }

    $forbidden = @(
        "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED",
        "VAULT_PROVIDER_CONTAINED_CONSUMED",
        "VAULT_WIFI_PREUSE_AUDIT_COMMITTED",
        "VAULT_WIFI_CONTAINED_CONSUMED"
    ) | Where-Object { (Get-SerialMarkerCount -Path $script:SerialLog -Marker $_) -ne 0 }
    $deniedBeforeAudit = @($forbidden).Count -eq 0
    Add-Predicate `
        -Name "secret-vault:boot3:forgotten_slots_deny_before_audit_or_consumer" `
        -Expected "tombstoned provider and WiFi slots emit no pre-use audit or consumer success" `
        -Passed $deniedBeforeAudit `
        -Actual $(if ($deniedBeforeAudit) { "audit=false consumer=false" } else { "forbidden=$($forbidden -join ',')" })
    if (-not $deniedBeforeAudit) {
        throw "Forgotten Vault slot reached an audit or consumer success"
    }

    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:boot3:rr1_absent_from_serial" -Path $boot3Log
    return $boot3Log
}

function Join-SecretVaultSerialLogs {
    param(
        [Parameter(Mandatory = $true)][string[]]$Paths,
        [Parameter(Mandatory = $true)][string]$Destination
    )

    $destinationStream = [System.IO.File]::Open(
        $Destination,
        [System.IO.FileMode]::Create,
        [System.IO.FileAccess]::Write,
        [System.IO.FileShare]::Read
    )
    try {
        foreach ($path in $Paths) {
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                throw "secret-vault combined log input missing: $path"
            }
            $source = [System.IO.File]::OpenRead($path)
            try {
                $source.CopyTo($destinationStream)
            }
            finally {
                $source.Dispose()
            }
            $separator = [byte[]](13, 10)
            $destinationStream.Write($separator, 0, $separator.Length)
        }
        $destinationStream.Flush()
    }
    finally {
        $destinationStream.Dispose()
    }
}
