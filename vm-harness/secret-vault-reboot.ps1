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

function Invoke-SecretVaultPersonalTrapContinuityProof {
    $log = $script:SerialLog
    $usbBefore = Get-SerialMarkerCount -Path $log -Marker "usb input batch:"
    $setupClosedBefore = Get-SerialMarkerCount -Path $log -Marker "SETUP CLOSED"
    $manageBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_MANAGE_READY"
    $trapBefore = Get-SerialMarkerCount -Path $log -Marker "PERSONAL SHELL FALLBACK trap"
    $rr1UnlockBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_RR1_UNLOCKED"
    $brokerUnlockBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_BROKER_UNLOCKED source=recovery slot=current"
    $unlockReadyBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_RECOVERY_UNLOCK_READY"
    $providerAuditBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED"
    $providerConsumerBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_PROVIDER_CONTAINED_CONSUMED"
    $wifiAuditBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_WIFI_PREUSE_AUDIT_COMMITTED"
    $wifiConsumerBefore = Get-SerialMarkerCount -Path $log -Marker "VAULT_WIFI_CONTAINED_CONSUMED"

    # Close the unlock outcome, then leave Settings physically so the existing
    # typed agent command runs only through Command mode.
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "esc"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:personal_trap_command_mode" `
        -Marker "SETUP CLOSED" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfterClose = Get-SerialMarkerCount -Path $log -Marker "usb input batch:"
    $setupClosedAfter = Get-SerialMarkerCount -Path $log -Marker "SETUP CLOSED"
    if ($setupClosedAfter -ne ($setupClosedBefore + 1)) {
        throw "Personal trap command route did not leave Settings exactly once"
    }

    Send-AgentCommand `
        -Command "ui.personal_shell_proof trap" `
        -ExpectedMarker "RAIOS_AGENT_END ui.personal_shell_proof" `
        -Name "secret-vault:boot2:personal_trap_request"
    $trapResponse = Get-LastAgentResponseJson -Method "ui.personal_shell_proof"
    $trap = $trapResponse.body.result
    $trapAccepted = $trap.activation_mode -eq "trap" -and
        $trap.activation_requested -eq $true -and
        $trap.activation_request_reason -eq "queued_for_core_owned_shell_host"
    Add-Predicate `
        -Name "secret-vault:boot2:personal_trap_request_bounded" `
        -Expected "existing signed current-boot personal-shell trap proof is queued without a new crash command" `
        -Passed $trapAccepted `
        -Actual $(if ($trapAccepted) { "trap=queued existing_proof=true" } else { "trap=not_queued" })
    if (-not $trapAccepted) {
        throw "Secret Vault continuity proof could not queue the existing personal trap"
    }

    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:personal_trap_fallback" `
        -Marker "PERSONAL SHELL FALLBACK trap" `
        -TimeoutSeconds $TimeoutSeconds
    $trapAfter = Get-SerialMarkerCount -Path $log -Marker "PERSONAL SHELL FALLBACK trap"
    $newTrapFallback = $trapAfter -eq ($trapBefore + 1)
    Add-Predicate `
        -Name "secret-vault:boot2:personal_trap_fallback_new" `
        -Expected "the requested existing trap produces exactly one new fallback to core Genesis" `
        -Passed $newTrapFallback `
        -Actual "new_fallback=$($newTrapFallback.ToString().ToLowerInvariant())"
    if (-not $newTrapFallback) {
        throw "Personal trap did not produce exactly one new Genesis fallback"
    }

    Send-AgentCommand `
        -Command "agent recovery.snapshot" `
        -ExpectedMarker "RAIOS_AGENT_END recovery.snapshot" `
        -Name "secret-vault:boot2:recovery_after_personal_trap"
    $recoveryResponse = Get-LastAgentResponseJson -Method "recovery.snapshot"
    $recovery = $recoveryResponse.body.result
    $recoveryResponsive = $recovery.schema -eq "raios.recovery_snapshot.v0" -and
        $recovery.scope -eq "current_boot" -and
        $recovery.classification -eq "local_only" -and
        $recovery.lifeline_available -eq $true -and
        $recovery.mutates_state -eq $false -and
        $recovery.routes_through_wasm -eq $false -and
        $recovery.routes_through_provider -eq $false -and
        $recovery.redacted -eq $true
    Add-Predicate `
        -Name "secret-vault:boot2:genesis_recovery_responsive_after_personal_trap" `
        -Expected "core Genesis fallback still serves the redacted non-mutating recovery snapshot" `
        -Passed $recoveryResponsive `
        -Actual $(if ($recoveryResponsive) { "genesis=true recovery=responsive" } else { "genesis_or_recovery=invalid" })
    if (-not $recoveryResponsive) {
        throw "Genesis recovery was not responsive after the personal trap"
    }

    # Navigate from core Genesis back to Vault physically; an already unlocked
    # Broker must enter Manage without RR1 input.
    Invoke-VaultVisibleAction -SkipFinalEnter
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:boot2:manage_after_personal_trap" `
        -Marker "VAULT_MANAGE_READY" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfterReopen = Get-SerialMarkerCount -Path $log -Marker "usb input batch:"
    $manageAfter = Get-SerialMarkerCount -Path $log -Marker "VAULT_MANAGE_READY"
    $physical = $usbAfterClose -gt $usbBefore -and $usbAfterReopen -gt $usbAfterClose
    $newManage = $manageAfter -eq ($manageBefore + 1)
    Add-Predicate `
        -Name "secret-vault:boot2:personal_trap_vault_actions_physical" `
        -Expected "physical Enter/Escape leave Settings and later physical navigation opens one new Vault Manage view" `
        -Passed ($physical -and $newManage) `
        -Actual "physical=$($physical.ToString().ToLowerInvariant()) new_manage=$($newManage.ToString().ToLowerInvariant())"
    if (-not ($physical -and $newManage)) {
        throw "Vault close/reopen around the personal trap was not exactly physical"
    }

    $unlockUnchanged = $rr1UnlockBefore -gt 0 -and $brokerUnlockBefore -gt 0 -and
        $unlockReadyBefore -gt 0 -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_RR1_UNLOCKED") -eq $rr1UnlockBefore -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_BROKER_UNLOCKED source=recovery slot=current") -eq $brokerUnlockBefore -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_RECOVERY_UNLOCK_READY") -eq $unlockReadyBefore
    Add-Predicate `
        -Name "secret-vault:boot2:personal_trap_preserves_unlocked_vault" `
        -Expected "personal trap and Genesis fallback preserve the unlocked core Vault handle without RR1 or another unlock" `
        -Passed $unlockUnchanged `
        -Actual "unlock_unchanged=$($unlockUnchanged.ToString().ToLowerInvariant())"
    if (-not $unlockUnchanged) {
        throw "Personal trap changed the Vault unlock count"
    }

    $secretUseUnchanged =
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED") -eq $providerAuditBefore -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_PROVIDER_CONTAINED_CONSUMED") -eq $providerConsumerBefore -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_WIFI_PREUSE_AUDIT_COMMITTED") -eq $wifiAuditBefore -and
        (Get-SerialMarkerCount -Path $log -Marker "VAULT_WIFI_CONTAINED_CONSUMED") -eq $wifiConsumerBefore
    Add-Predicate `
        -Name "secret-vault:boot2:personal_trap_triggers_no_secret_use" `
        -Expected "personal trap, recovery read and Vault reopen add no provider/WiFi audit or consumer use" `
        -Passed $secretUseUnchanged `
        -Actual "secret_use_unchanged=$($secretUseUnchanged.ToString().ToLowerInvariant())"
    if (-not $secretUseUnchanged) {
        throw "Personal trap unexpectedly triggered a secret audit or consumer"
    }
}

function Start-SecretVaultQemu {
    param(
        [Parameter(Mandatory = $true)][string]$LogName,
        [string]$StructuredStorePath = $StructuredStoreDiskImage,
        [string]$PersistPath = $PersistDiskImage
    )
    $nextLog = Join-Path $RunDir $LogName
    $params = $runParams.Clone()
    $params.StopExisting = $false
    $params.SerialLog = $nextLog
    $params.StructuredStoreDiskPath = $StructuredStorePath
    $params.PersistDiskPath = $PersistPath
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

function Restart-SecretVaultQemu {
    param([Parameter(Mandatory = $true)][string]$LogName)

    if (-not $script:QemuPid) {
        throw "secret-vault reboot requires a live QEMU process"
    }
    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    return Start-SecretVaultQemu -LogName $LogName
}

function Invoke-SecretVaultStoreDenialProof {
    if ($script:QemuPid -and (Get-Process -Id $script:QemuPid -ErrorAction SilentlyContinue)) {
        Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    }

    $mutator = Join-Path $RepoRoot "scripts\make-structured-store-image.py"
    $persistBuilder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
    $persist = Join-Path $RunDir "raios-persist-store-denial.img"
    $foreignStore = Join-Path $RunDir "raios-structured-store-c1-foreign-guid.img"
    $corruptStore = Join-Path $RunDir "raios-structured-store-c1-corrupt-frame.img"
    foreach ($path in @($persist, $foreignStore, $corruptStore)) {
        if (Test-Path -LiteralPath $path) {
            throw "secret-vault store-denial output already exists: $path"
        }
    }

    $null = @(& python $persistBuilder --self-check --seed-bootctl valid-a $persist 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "secret-vault store-denial valid-a persist fixture build failed"
    }

    $foreignMutationOutput = @(
        & python $mutator mutate-partition-guid $StructuredStoreDiskImage $foreignStore 2>&1
    )
    if ($LASTEXITCODE -ne 0) {
        throw "secret-vault foreign-partition copy mutation failed: $($foreignMutationOutput -join ' ')"
    }
    $foreignMutation = ($foreignMutationOutput -join [Environment]::NewLine) | ConvertFrom-Json
    if ($foreignMutation.operation -ne "change_partition_unique_guid" -or
        $foreignMutation.source_exact_identity -ne $true -or
        $foreignMutation.mutated_exact_identity -ne $false -or
        $foreignMutation.gpt_crcs_recomputed -ne $true -or
        $foreignMutation.initialized_content_preserved -ne $true) {
        throw "secret-vault foreign-partition copy mutation evidence is invalid"
    }

    $corruptMutationOutput = @(
        & python $mutator corrupt-last-log-frame $StructuredStoreDiskImage $corruptStore 2>&1
    )
    if ($LASTEXITCODE -ne 0) {
        throw "secret-vault corrupt-frame copy mutation failed: $($corruptMutationOutput -join ' ')"
    }
    $corruptMutation = ($corruptMutationOutput -join [Environment]::NewLine) | ConvertFrom-Json
    if ($corruptMutation.operation -ne "corrupt_last_log_frame" -or
        $corruptMutation.exact_test_image_identity -ne $true -or
        $corruptMutation.gpt_unchanged -ne $true -or
        $corruptMutation.initialized_content_present -ne $true) {
        throw "secret-vault corrupt-frame copy mutation evidence is invalid"
    }

    $logs = @()
    $cases = @(
        [pscustomobject]@{
            Name = "foreign-guid"
            Store = $foreignStore
            LogName = "serial-secret-vault-foreign-guid-denial.log"
            Denial = "C1_STRUCTURED_STORE_DENIED: Gpt(PartitionIdentityMismatch)"
            FixtureAccepted = $false
        },
        [pscustomobject]@{
            Name = "corrupt-frame"
            Store = $corruptStore
            LogName = "serial-secret-vault-corrupt-frame-denial.log"
            Denial = "C1_STRUCTURED_STORE_DENIED: Core(StoreChainLocked)"
            FixtureAccepted = $true
        }
    )
    $forbiddenSuccess = @(
        "C1_VAULT_COMPLETE_REPLAY_BOUND",
        "C1_VAULT_RECOVERY_WRAPPER_",
        "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED",
        "VAULT_WIFI_PREUSE_AUDIT_COMMITTED",
        "VAULT_PROVIDER_CONTAINED_CONSUMED",
        "VAULT_WIFI_CONTAINED_CONSUMED",
        "VAULT_BROKER_UNLOCKED",
        "VAULT_RR1_UNLOCKED"
    )

    foreach ($case in $cases) {
        $log = Start-SecretVaultQemu `
            -LogName $case.LogName `
            -StructuredStorePath $case.Store `
            -PersistPath $persist
        $logs += $log
        try {
            Assert-VaultFixedLogMarker `
                -Name "secret-vault:store-denial:$($case.Name):serial_ready" `
                -Marker "SERIAL CONSOLE READY" `
                -TimeoutSeconds $TimeoutSeconds
            Assert-VaultFixedLogMarker `
                -Name "secret-vault:store-denial:$($case.Name):usb_keyboard_ready" `
                -Marker "usb-hid: keyboard ready on slot" `
                -TimeoutSeconds $TimeoutSeconds
            Assert-VaultFixedLogMarker `
                -Name "secret-vault:store-denial:$($case.Name):c1_denied" `
                -Marker $case.Denial `
                -TimeoutSeconds $TimeoutSeconds

            $acceptedCount = Get-SerialMarkerCount `
                -Path $log `
                -Marker "C1_STRUCTURED_STORE_FIXTURE_ACCEPTED"
            $acceptedAsExpected = if ($case.FixtureAccepted) {
                $acceptedCount -eq 1
            }
            else {
                $acceptedCount -eq 0
            }
            if ($case.FixtureAccepted) {
                $acceptedOffset = Get-FileByteSequenceOffset `
                    -Path $log `
                    -Needle ([System.Text.Encoding]::ASCII.GetBytes("C1_STRUCTURED_STORE_FIXTURE_ACCEPTED"))
                $denialOffset = Get-FileByteSequenceOffset `
                    -Path $log `
                    -Needle ([System.Text.Encoding]::ASCII.GetBytes($case.Denial))
                $acceptedAsExpected = $acceptedAsExpected -and
                    $acceptedOffset -ge 0 -and $denialOffset -gt $acceptedOffset
            }
            Add-Predicate `
                -Name "secret-vault:store-denial:$($case.Name):fixture_boundary" `
                -Expected $(if ($case.FixtureAccepted) { "exact fixture accepted once before corrupt log denial" } else { "foreign partition identity denied before fixture acceptance" }) `
                -Passed $acceptedAsExpected `
                -Actual "fixture_accepted_count=$acceptedCount ordered=$($acceptedAsExpected.ToString().ToLowerInvariant())"
            if (-not $acceptedAsExpected) {
                throw "secret-vault $($case.Name) crossed the wrong fixture-acceptance boundary"
            }

            $forbidden = @(
                $forbiddenSuccess | Where-Object {
                    (Get-SerialMarkerCount -Path $log -Marker $_) -ne 0
                }
            )
            $failedClosed = $forbidden.Count -eq 0
            Add-Predicate `
                -Name "secret-vault:store-denial:$($case.Name):no_vault_authority" `
                -Expected "store denial emits no complete replay, wrapper, audit, consumer, or unlock success" `
                -Passed $failedClosed `
                -Actual $(if ($failedClosed) { "vault_authority=false" } else { "forbidden=$($forbidden -join ',')" })
            if (-not $failedClosed) {
                throw "secret-vault $($case.Name) emitted Vault authority after store denial"
            }

            $usbBefore = Get-SerialMarkerCount -Path $log -Marker "usb input batch:"
            Invoke-VaultVisibleAction
            Assert-VaultFixedLogMarker `
                -Name "secret-vault:store-denial:$($case.Name):visible_ram_only_denial" `
                -Marker "VAULT_UNAVAILABLE state=ram_only_denial current_boot=true" `
                -TimeoutSeconds $TimeoutSeconds
            $usbAfter = Get-SerialMarkerCount -Path $log -Marker "usb input batch:"
            $physical = $usbAfter -gt $usbBefore
            Add-Predicate `
                -Name "secret-vault:store-denial:$($case.Name):physical_vault_action" `
                -Expected "physical Genesis Vault action reaches the visible RAM-only denial" `
                -Passed $physical `
                -Actual "physical=$($physical.ToString().ToLowerInvariant()) before=$usbBefore after=$usbAfter"
            if (-not $physical) {
                throw "secret-vault $($case.Name) denial action did not traverse USB HID"
            }

            $lateForbidden = @(
                $forbiddenSuccess | Where-Object {
                    (Get-SerialMarkerCount -Path $log -Marker $_) -ne 0
                }
            )
            $stillFailedClosed = $lateForbidden.Count -eq 0
            Add-Predicate `
                -Name "secret-vault:store-denial:$($case.Name):physical_denial_grants_nothing" `
                -Expected "physical visible-denial action still emits no replay, wrapper, audit, consumer, or unlock success" `
                -Passed $stillFailedClosed `
                -Actual $(if ($stillFailedClosed) { "vault_authority=false" } else { "forbidden=$($lateForbidden -join ',')" })
            if (-not $stillFailedClosed) {
                throw "secret-vault $($case.Name) physical denial emitted forbidden success: $($lateForbidden -join ',')"
            }
        }
        finally {
            if ($script:QemuPid) {
                Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
            }
        }
    }

    return [pscustomobject]@{
        Logs = [string[]]$logs
        StructuredStores = [string[]]@($foreignStore, $corruptStore)
        PersistImages = [string[]]@($persist)
    }
}

function Invoke-SecretVaultSafeReconnectProof {
    param(
        [Parameter(Mandatory = $true)][byte[]]$Rr1,
        [Parameter(Mandatory = $true)][int]$InitialFrameCount
    )

    $safeStore = Join-Path $RunDir "raios-structured-store-c1-safe.img"
    $safePersist = Join-Path $RunDir "raios-persist-safe.img"
    Copy-Item -LiteralPath $StructuredStoreDiskImage -Destination $safeStore -Force

    $builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
    if ($InitialFrameCount -ne 0) {
        throw "secret-vault SAFE fixture requires the observed empty pre-audit RECLOG"
    }
    $null = @(& python $builder --self-check --seed-bootctl pending-safe $safePersist 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "secret-vault SAFE BOOTCTL fixture build failed"
    }
    $seed = Get-PersistInspectionForVault -Path $safePersist
    $seeded = $seed.bootctl_read.decision.posture -eq "Safe" -and
        [int]$seed.reclog_scan.count -eq 0
    Add-Predicate `
        -Name "secret-vault:safe:fixture_seeded" `
        -Expected "separate sparse fixture has authoritative pending-safe BOOTCTL and the observed empty pre-audit RECLOG" `
        -Passed $seeded `
        -Actual $(if ($seeded) { "posture=Safe reclog_count=0" } else { "posture_or_reclog=invalid" })
    if (-not $seeded) {
        throw "secret-vault SAFE BOOTCTL fixture did not inspect as Safe"
    }

    $safeLog = Start-SecretVaultQemu `
        -LogName "serial-secret-vault-safe-reconnect.log" `
        -StructuredStorePath $safeStore `
        -PersistPath $safePersist
    foreach ($marker in @(
        @{ Name = "secret-vault:safe:serial_ready"; Text = "SERIAL CONSOLE READY" },
        @{ Name = "secret-vault:safe:last_good_core_policy"; Text = "CORE_POLICY_SAFE_LAST_GOOD_OWNER_VERIFIED slot=A generation=1" },
        @{ Name = "secret-vault:safe:complete_replay_bound"; Text = "C1_VAULT_COMPLETE_REPLAY_BOUND" },
        @{ Name = "secret-vault:safe:core_policy_bound"; Text = "C1_VAULT_CORE_POLICY_BOUND" },
        @{ Name = "secret-vault:safe:wrapper_replayed"; Text = "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation=1" },
        @{ Name = "secret-vault:safe:wifi_replayed"; Text = "C1_VAULT_WIFI_REPLAYED version=1" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    $usbBefore = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    Invoke-VaultVisibleAction -SkipFinalEnter
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:safe:recovery_unlock_ready" `
        -Marker "VAULT_RECOVERY_UNLOCK_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1ViaUsbKeyboard -Rr1 $Rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:safe:broker_unlocked" `
        -SuccessMarker "VAULT_RR1_UNLOCKED" `
        -RejectedMarker "VAULT_RR1_UNLOCK_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:safe:auto_connect_denied" `
        -Marker "VAULT_SAFE_UNLOCKED_AUTO_CONNECT_DENIED explicit_genesis_action_required=true" `
        -TimeoutSeconds $TimeoutSeconds

    $preActionForbidden = @(
        "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED",
        "VAULT_PROVIDER_CONTAINED_CONSUMED",
        "VAULT_WIFI_PREUSE_AUDIT_COMMITTED",
        "VAULT_WIFI_CONTAINED_CONSUMED"
    ) | Where-Object { (Get-SerialMarkerCount -Path $script:SerialLog -Marker $_) -ne 0 }
    $noAutoUse = @($preActionForbidden).Count -eq 0
    Add-Predicate `
        -Name "secret-vault:safe:no_use_before_physical_action" `
        -Expected "SAFE unlock emits no provider/WiFi audit or consumer success" `
        -Passed $noAutoUse `
        -Actual $(if ($noAutoUse) { "audit=false consumer=false" } else { "forbidden=$($preActionForbidden -join ',')" })
    if (-not $noAutoUse) {
        throw "SAFE unlock reached a secret audit or consumer before physical action"
    }

    # Close the unlock outcome, move from Vault to the contained Set-WiFi
    # action, then activate it physically. No serial command can reach it.
    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    for ($index = 0; $index -lt 3; $index++) {
        Send-Rr1HmpKey -KeyName "tab"
    }
    Send-Rr1HmpKey -KeyName "ret"
    foreach ($marker in @(
        @{ Name = "secret-vault:safe:wifi_preuse_audit"; Text = "VAULT_WIFI_PREUSE_AUDIT_COMMITTED consumer=native_wifi_supplicant operation=associate_bound_bss readback=verified reparse=verified rescan=verified" },
        @{ Name = "secret-vault:safe:wifi_consumer"; Text = "VAULT_WIFI_CONTAINED_CONSUMED target=bound_bss accepted=true test_infrastructure=true" },
        @{ Name = "secret-vault:safe:explicit_reconnect_completed"; Text = "VAULT_WIFI_SAFE_EXPLICIT_RECONNECT_COMPLETED test_infrastructure=true" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    $exactOneWifiUse = (Get-SerialMarkerCount -Path $script:SerialLog -Marker "VAULT_WIFI_PREUSE_AUDIT_COMMITTED") -eq 1 -and
        (Get-SerialMarkerCount -Path $script:SerialLog -Marker "VAULT_WIFI_CONTAINED_CONSUMED") -eq 1 -and
        (Get-SerialMarkerCount -Path $script:SerialLog -Marker "VAULT_PROVIDER_PREUSE_AUDIT_COMMITTED") -eq 0 -and
        (Get-SerialMarkerCount -Path $script:SerialLog -Marker "VAULT_PROVIDER_CONTAINED_CONSUMED") -eq 0
    $usbAfter = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    $explicitPhysical = $usbAfter -gt $usbBefore
    Add-Predicate `
        -Name "secret-vault:safe:exactly_one_physical_wifi_use" `
        -Expected "one physical Genesis action yields exactly one SAFE WiFi audit+consumer and no provider use" `
        -Passed ($exactOneWifiUse -and $explicitPhysical) `
        -Actual "wifi_use_once=$($exactOneWifiUse.ToString().ToLowerInvariant()) physical=$($explicitPhysical.ToString().ToLowerInvariant())"
    if (-not ($exactOneWifiUse -and $explicitPhysical)) {
        throw "SAFE WiFi reconnect was not exactly-once and physical"
    }

    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:safe:rr1_absent_from_serial" -Path $safeLog

    $inspection = Get-PersistInspectionForVault -Path $safePersist
    $frames = @($inspection.reclog_frames)
    $newFrames = @($frames | Select-Object -Skip $InitialFrameCount)
    $record = if ($newFrames.Count -eq 1) { $newFrames[0].payload_json } else { $null }
    $value = if ($record) { $record.value } else { $null }
    $fields = if ($value) { @($value.PSObject.Properties.Name) } else { @() }
    $exactSafeAudit = $inspection.reclog_scan.status -eq "valid" -and
        [int]$inspection.reclog_scan.count -eq ($InitialFrameCount + 1) -and
        $record.entity -eq "native_wifi_supplicant" -and
        $record.predicate -eq "associate_bound_bss" -and
        $record.classification -eq "local_only" -and
        $value.boot_scope -eq "safe_recovery" -and
        $value.explicit_recovery_action -eq $true -and
        $fields -notcontains "ssid" -and
        $fields -notcontains "bssid" -and
        $fields -notcontains "target"
    Add-Predicate `
        -Name "secret-vault:safe:offline_exact_audit" `
        -Expected "exactly one new local_only WiFi record binds safe_recovery+explicit action and no raw BSS" `
        -Passed $exactSafeAudit `
        -Actual $(if ($exactSafeAudit) { "delta=one scope=safe_recovery explicit=true" } else { "delta_or_binding=invalid" })
    if (-not $exactSafeAudit) {
        throw "offline SAFE WiFi audit binding is invalid"
    }
    return [pscustomobject]@{
        Log = $safeLog
        StructuredStore = $safeStore
        Persist = $safePersist
    }
}

function Invoke-SecretVaultPowerCutProof {
    param([Parameter(Mandatory = $true)][byte[]]$Rr1)

    $powerCutStore = Join-Path $RunDir "raios-structured-store-c1-power-cut.img"
    $powerCutPersist = Join-Path $RunDir "raios-persist-power-cut.img"
    Copy-Item -LiteralPath $StructuredStoreDiskImage -Destination $powerCutStore -Force

    $builder = Join-Path $RepoRoot "scripts\make-gpt-persist-image.py"
    $null = @(& python $builder --self-check --seed-bootctl valid-a $powerCutPersist 2>&1)
    if ($LASTEXITCODE -ne 0) {
        throw "secret-vault power-cut BOOTCTL fixture build failed"
    }

    $beforeLog = Start-SecretVaultQemu `
        -LogName "serial-secret-vault-power-cut-before.log" `
        -StructuredStorePath $powerCutStore `
        -PersistPath $powerCutPersist
    foreach ($marker in @(
        @{ Name = "secret-vault:power-cut:before:serial_ready"; Text = "SERIAL CONSOLE READY" },
        @{ Name = "secret-vault:power-cut:before:complete_replay_bound"; Text = "C1_VAULT_COMPLETE_REPLAY_BOUND" },
        @{ Name = "secret-vault:power-cut:before:wrapper_replayed"; Text = "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation=1" },
        @{ Name = "secret-vault:power-cut:before:wifi_replayed"; Text = "C1_VAULT_WIFI_REPLAYED version=1" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    Invoke-VaultVisibleAction -SkipFinalEnter
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:power-cut:before:recovery_unlock_ready" `
        -Marker "VAULT_RECOVERY_UNLOCK_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1ViaUsbKeyboard -Rr1 $Rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:power-cut:before:broker_unlocked" `
        -SuccessMarker "VAULT_RR1_UNLOCKED" `
        -RejectedMarker "VAULT_RR1_UNLOCK_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds

    $usbBefore = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    Send-Rr1HmpKey -KeyName "f9"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:power-cut:precommit_ready" `
        -Marker "C1_VAULT_POWER_CUT_PRECOMMIT_READY key=wifi previous_version=1 proposed_version=2 commit_written=false test_infrastructure=true" `
        -TimeoutSeconds $TimeoutSeconds
    $usbAfter = Get-SerialMarkerCount -Path $script:SerialLog -Marker "usb input batch:"
    $physical = $usbAfter -gt $usbBefore
    Add-Predicate `
        -Name "secret-vault:power-cut:physical_f9_action" `
        -Expected "physical F9 on the exact contained C1 fixture arms the pre-COMMIT boundary" `
        -Passed $physical `
        -Actual "physical=$($physical.ToString().ToLowerInvariant())"
    if (-not $physical) {
        throw "secret-vault power-cut action did not traverse USB HID"
    }
    $commitAbsent = (Get-SerialMarkerCount -Path $script:SerialLog -Marker "C1_VAULT_WIFI_TOMBSTONE_COMMITTED version=2") -eq 0
    Add-Predicate `
        -Name "secret-vault:power-cut:no_commit_before_kill" `
        -Expected "no v2 WiFi tombstone COMMIT success before hard QEMU stop" `
        -Passed $commitAbsent `
        -Actual "commit_written=$(((-not $commitAbsent)).ToString().ToLowerInvariant())"
    if (-not $commitAbsent) {
        throw "power-cut fixture unexpectedly committed WiFi v2"
    }

    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:power-cut:before:rr1_absent_from_serial" -Path $beforeLog

    $afterLog = Start-SecretVaultQemu `
        -LogName "serial-secret-vault-power-cut-after.log" `
        -StructuredStorePath $powerCutStore `
        -PersistPath $powerCutPersist
    foreach ($marker in @(
        @{ Name = "secret-vault:power-cut:after:serial_ready"; Text = "SERIAL CONSOLE READY" },
        @{ Name = "secret-vault:power-cut:after:replay_preserved"; Text = "C1_VAULT_POWER_CUT_REPLAY_PRESERVED key=wifi version=1 tail=incomplete_transaction test_infrastructure=true" },
        @{ Name = "secret-vault:power-cut:after:complete_replay_bound"; Text = "C1_VAULT_COMPLETE_REPLAY_BOUND" },
        @{ Name = "secret-vault:power-cut:after:wrapper_replayed"; Text = "C1_VAULT_RECOVERY_WRAPPER_REPLAYED generation=1" },
        @{ Name = "secret-vault:power-cut:after:wifi_v1_replayed"; Text = "C1_VAULT_WIFI_REPLAYED version=1" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    Invoke-VaultVisibleAction -SkipFinalEnter
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:power-cut:after:recovery_unlock_ready" `
        -Marker "VAULT_RECOVERY_UNLOCK_READY" `
        -TimeoutSeconds $TimeoutSeconds
    Send-Rr1ViaUsbKeyboard -Rr1 $Rr1
    Assert-VaultSerialOutcome `
        -Name "secret-vault:power-cut:after:broker_unlocked" `
        -SuccessMarker "VAULT_RR1_UNLOCKED" `
        -RejectedMarker "VAULT_RR1_UNLOCK_REJECTED" `
        -TimeoutSeconds $TimeoutSeconds
    foreach ($marker in @(
        @{ Name = "secret-vault:power-cut:after:provider_consumer"; Text = "VAULT_PROVIDER_CONTAINED_CONSUMED target=api.openai.com accepted=true test_infrastructure=true" },
        @{ Name = "secret-vault:power-cut:after:wifi_consumer"; Text = "VAULT_WIFI_CONTAINED_CONSUMED target=bound_bss accepted=true test_infrastructure=true" }
    )) {
        Assert-VaultFixedLogMarker -Name $marker.Name -Marker $marker.Text -TimeoutSeconds $TimeoutSeconds
    }

    Send-Rr1HmpKey -KeyName "ret"
    Start-Sleep -Milliseconds 150
    Send-Rr1HmpKey -KeyName "ret"
    Assert-VaultFixedLogMarker `
        -Name "secret-vault:power-cut:after:vault_handle_preserved" `
        -Marker "VAULT_MANAGE_READY" `
        -TimeoutSeconds $TimeoutSeconds

    Stop-Rr1VmForLogInspection -QemuProcessId $script:QemuPid
    Assert-Rr1NotInSerial -Name "secret-vault:power-cut:after:rr1_absent_from_serial" -Path $afterLog
    return [pscustomobject]@{
        BeforeLog = $beforeLog
        AfterLog = $afterLog
        StructuredStore = $powerCutStore
        Persist = $powerCutPersist
    }
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
