function Send-QemuMonitorCommand {
    param([string]$Command)

    if ($MonitorTcpPort -le 0) {
        throw "usb-hotplug profile requires a QEMU monitor TCP port"
    }

    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $client.Connect("127.0.0.1", $MonitorTcpPort)
        $stream = $client.GetStream()
        $bytes = [System.Text.Encoding]::ASCII.GetBytes("$Command`n")
        $stream.Write($bytes, 0, $bytes.Length)
        $stream.Flush()
        Start-Sleep -Milliseconds 500
    }
    finally {
        $client.Close()
    }
}

Send-QemuMonitorCommand -Command "device_del bootkbd"
Assert-LogContains `
    -Name "usb_hotplug:serial_disconnect_boot_keyboard" `
    -Needle "usb-hotplug: disconnect root-port" `
    -TimeoutSeconds $TimeoutSeconds

Send-QemuMonitorCommand -Command "device_add usb-kbd,bus=xhci.0,id=hotkbd"
Assert-LogContains `
    -Name "usb_hotplug:serial_connect_root_keyboard" `
    -Needle "usb-hotplug: connect root-port" `
    -TimeoutSeconds $TimeoutSeconds
Assert-LogContains `
    -Name "usb_hotplug:serial_connect_keyboard_outcome" `
    -Needle "-> keyboard" `
    -TimeoutSeconds 1
