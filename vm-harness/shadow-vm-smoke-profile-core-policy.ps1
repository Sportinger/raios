Assert-LogContains `
    -Name "core-policy:owner_signature_kernel_bootctl_bound" `
    -Needle "CORE_POLICY_OWNER_VERIFIED slot=A generation=1" `
    -TimeoutSeconds $TimeoutSeconds

$denialObserved = Select-String -LiteralPath $SerialLog -SimpleMatch "CORE_POLICY_DENIED" -Quiet
Add-Predicate `
    -Name "core-policy:no_positive_denial" `
    -Expected "the owner-signed A/1 policy produces no Core-Policy denial" `
    -Passed (-not $denialObserved) `
    -Actual $(if ($denialObserved) { "CORE_POLICY_DENIED observed" } else { "no denial observed" })

if ($denialObserved) {
    throw "Owner-signed Core Policy was denied"
}
