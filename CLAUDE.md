# UI-Lab

Für eine interaktive QEMU-Vorschau immer
`scripts/run-stage0-baremetal-vm.ps1` mit einer temporären Kopie von
`release/raios-stage0.img` starten. Der Wrapper aktiviert `qemu-xhci`,
`usb-kbd` und `usb-tablet`; ohne ihn fehlt die Maus im Overlay.

Vor dem Start prüfen, dass keine andere QEMU-Instanz läuft.
