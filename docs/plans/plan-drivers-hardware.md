# raiOS Treiber-Plan — GPU-Weg und Agent-Installer

Owner-Vormerkung (2026-07-18, Gespräch mit dem Orchestrator). Status:
**Vormerkung** — künftige Richtung für `docs/scope/05-drivers-hardware.md`;
keine SCOPE-Änderung, aktiviert erst nach den Fabrik-Meilensteinen und der
Hardware-Steckdose (`docs/plans/plan-genesis-layer.md`).

## 1. GPU-Treiber durch die Bau-Schicht (BMA), nie durch den Kernel

Sobald die Steckdose steht, ist jeder GPU-Port eine Treiber-Domäne mit
verliehenen Autoritäten — der Kernel bleibt bei jedem Hersteller unberührt.
Ein Port gilt pro **Geräte-Generation** (neue Generation = neue Register +
Firmware = Anpassung in der BMA-Schicht, kein Kernel-Eingriff).

Reihenfolge nach Machbarkeit (Recherche 2026-07-18):

1. **Intel** — öffentliche Doku, steckt im Surface; natürlicher Erstkandidat.
2. **AMD** — offener Treiber + Firmware-Blobs, mittel.
3. **NVIDIA** — seit RTX-2000 real offen geworden: offizielle offene
   Kernel-Module (ab Blackwell sogar nur noch offen), **Nova** (offener
   Rust-Treiber, NVIDIA arbeitet selbst mit — Rust→Rust ist unsere natürliche
   Portierungsquelle), NVK als offener Vulkan-Userspace. Haken: das
   Treiber-„Gehirn" liegt in der signierten, geschlossenen **GSP-Firmware**;
   Vertrauensmodell dafür ist 1:1 das des Marvell-Blobs (owner-sealed,
   IOMMU-gefenced, signiert). Karten vor 2018 bleiben ohne offenen Weg.

Tempo-Ehrlichkeit: Spiel-Maßstab braucht zusätzlich die schnelle
Ausführungsstufe (Fabrik-Treppe Stufe 4–5 in
`docs/plans/plan-personal-rust-playground.md`) — ein Interpreter-Treiber kann
eine GPU nicht füttern. Reihenfolge der Treppe ist technisch zwingend.

## 2. Agent-Installer: Netz am Anfang, danach ist alles erreichbar

Schlüssel-Einsicht: Auf einer frischen Maschine entscheidet allein die erste
Netzverbindung — danach kann der Agent jeden weiteren Treiber auf dem Gerät
selbst porten. Der Installer läuft auf dem **Alt-OS** (Windows/Linux), wo Netz
und Hardware-Kenntnis noch vorhanden sind, und bricht damit das
Henne-Ei-Problem:

- **Hardware-Scan auf dem Alt-OS:** exakte Chip-Identität (Hersteller-,
  Geräte-ID, Revision) statt Raten.
- **Port-Register:** Chip-ID → fertiger, signierter, bewiesener Port. Jeder
  gelungene Port dient ab dann allen Nutzern desselben Chips; WLAN-Chips
  klumpen stark (~5 Familien decken den Löwenanteil).
- **Universal-Fallback USB-Tethering:** ein einziger Standard-USB-Netztreiber
  (Handy als Modem) bringt praktisch jede Maschine ans Netz — der zumutbare
  Boden, wenn kein Port existiert.
- **Live-Port auf dem Gerät** ist der schnellste, ehrlichste Weg (echte
  Register, echte Interrupts; Kill-Switch macht Scheitern billig — Referenz:
  Marvell-Port in ~3 h mit Live-Iteration). Blind-One-Shot vom Installer ist
  Komfort-Bonus, keine tragende Säule.

Leiter im Installer: Register-Treffer → Tethering + Live-Port → Blind-One-Shot.
