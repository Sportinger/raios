# RaiOS — Scope & Vision

> **Status dieses Dokuments:** Zielbild. Geschrieben, als wäre RaiOS fertig.
> Jede Checkbox ist eine Funktion, die im fertigen System existieren muss.
> Nichts hier ist abgehakt — abgehakt wird nur, was durch Predicates belegt ist.

---

## Was RaiOS ist (fertig gedacht)

RaiOS ist ein agent-natives Betriebssystem. Ein minimaler, capability-basierter
Rust-Kernel (Genesis Layer) multiplext die nackte Hardware sicher. Alles darüber —
Treiber, Netzwerkstack, Grafik, Anwendungen — wird von Agenten gebaut, in isolierten
Domänen mit explizit verliehenen Hardware-Capabilities. Ein fehlgeschlagener
Agenten-Versuch reißt nie das System, sondern nur seine eigene Domäne.

RaiOS ersetzt den fehlenden mathematischen Beweis bewusst durch eine
Vertrauens-Pipeline: signierte Builds, Doppel-Bau, Reports, Rollback und
Ausbruchs-Negativtests von Tag 1. Das ist ein dokumentierter Trade, kein Versehen.

**Referenz-Hardware:** x86_64, Bare Metal auf Surface. **Sprache:** Rust.
**Entwickler:** Agenten (10 Lanes, 1 Orchestrator). **Mensch:** setzt Ziele, verleiht Rechte.

---

## 1. Rust Kernel (serieller Kern)

- [ ] Boot auf Bare Metal (UEFI) bis stabiler Idle-Zustand
- [ ] MMU / Paging: Domänen haben strikt getrennte Adressräume
- [ ] Scheduler: präemptiv, Domänen-fair, Kill-fähig
- [ ] Syscall-Schnittstelle: minimal, stabil, versioniert
- [ ] Interrupt-Routing an Userspace-Domänen (Treiber laufen NICHT im Kernel)
- [ ] IOMMU-Zwang für alles mit DMA-Fähigkeit
- [ ] `unsafe`-Anteil inventarisiert: jede unsafe-Stelle dokumentiert + Predicate-gedeckt
- [ ] Serial-/Debug-Ausgabe maschinenlesbar (RECLOG-Frames)

## 2. Genesis Layer (Capability-Boden)

- [ ] Primitive: `create_domain`, `grant_capability`, `revoke_capability`, `kill_domain`
- [ ] Storage-Primitive (persistenter Block-Zugriff als Capability)
- [ ] Capabilities feingranular: exakt ein PCIe-BAR, eine IRQ-Line, eine DMA-Region
- [ ] Kill + Restart einer Domäne ohne Systemneustart, in < 1 s
- [ ] **Boden-Schnittstelle schmal & kernel-agnostisch** — seL4 bleibt als
      alternativer Boden unterschiebbar (dokumentierter Vertrag, keine Kernel-Interna leaken)

## 3. Sicherheit & Vertrauens-Pipeline

**Tag 1 — Fundament prüfen** (Messgerät beim Bauen, nicht verhandelbar):

- [ ] Ausbruchs-Negativtests als Predicates:
  - [ ] Domäne greift fremden Domänen-Speicher an → verweigert + geloggt
  - [ ] Domäne greift Kernel-Speicher an → verweigert + geloggt
  - [ ] Domäne nutzt fremde DMA-Region → von IOMMU blockiert + geloggt
- [ ] Rollback: jede Domänen-Version zurückrollbar
- [ ] Report-Pipeline: jeder Build/Test erzeugt strukturierten Report (ARTSTOR)

**Phase "Verteilung" — Vertrauen für Fremde** (erst wenn andere das System nutzen):

- [ ] Signierte Builds + Doppel-Bau (Reproduzierbarkeit)
- [ ] Audit-Log: jede Capability-Verleihung ist nachvollziehbar (wer, was, wann, warum)

## 4. Agent Fabric (Orchestrierung)

- [ ] 10 parallele Agent-Lanes + 1 Orchestrator im gleichen Workspace
- [ ] Hardware-Introspektion maschinenlesbar: PCI-Enumeration, Register-Maps,
      Device-Infos als strukturierte Daten (nicht PDFs)
- [ ] Compiler-Diagnostik als JSON → direkter Agent-Feedback-Loop
      (generieren → kompilieren → Fehler lesen → fixen)
- [ ] Test-Harness (QEMU + Bare Metal) von Agenten selbst ansteuerbar (W6-Maschinerie)
- [ ] Lane-Regeln dokumentiert: was parallelisiert (Treiber, Predicates, Pipeline),
      was seriell bleibt (MMU, Scheduler, Syscalls — max. 2 Lanes)

## 5. Treiber & Hardware (agentengebaut, in Domänen)

- [ ] WLAN (Marvell-Port) läuft als isolierte Domäne mit eigener DMA-Region
- [ ] USB-Stack als Domäne
- [ ] Netzwerkstack als Domäne
- [ ] Storage-Treiber als Domäne
- [ ] GPU: Framebuffer-Zugriff als Capability (Fernziel: 3D/Rendering direkt auf Hardware)

## 6. Personal Rust Playground

- [ ] Ein Mensch oder Agent kann eine leere Domäne anfordern und darin bauen,
      ohne das System gefährden zu können
- [ ] Rust-Toolchain im OS (rustc mit Cranelift-Backend) für Selbst-Kompilierung
- [ ] Vorlage-Domänen ("Hello Hardware"): minimaler Start mit Serial-Out + 1 Capability
- [ ] Crash einer Playground-Domäne = Log + Neustart-Angebot, sonst nichts
- [ ] Playground-Ergebnisse können zu "echten" Domänen befördert werden
      (durch die Vertrauens-Pipeline, nicht daran vorbei)

## 7. Dokumentation & Projekt-Hygiene

- [ ] Dieses Scope-Dokument ist die einzige Quelle für "was RaiOS ist"
- [ ] Docs-Struktur: `/docs/vision` (dieses File), `/docs/architecture`,
      `/docs/decisions` (ADRs), `/docs/harness`, `/docs/agents` — nichts anderes
- [ ] Jede Architektur-Entscheidung als ADR (inkl. der seL4-Entscheidung mit Datum)
- [ ] Veraltete Pläne werden archiviert (`/docs/_archive`), nie still gelöscht

---

## Bewusste Nicht-Ziele

- **Kein formaler Beweis.** Ersatz: Predicates + Negativtests (siehe §3). Dokumentierter Trade.
- **Kein POSIX, keine Linux-Kompatibilität.** RaiOS ist kein Unix.
- **Kein Multi-User-Desktop-OS.** Eine Maschine, ein Besitzer, viele Domänen.
- **Keine Legacy-Hardware.** Nur Referenz-Hardware + QEMU.

---

*Definition of Done pro Checkbox: Funktion existiert + mindestens ein Predicate belegt sie + ein Negativtest belegt ihre Grenze.*
