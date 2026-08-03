# raiOS: Bericht zum Prinzip des selbsterklärenden Systems

**Stand:** 2026-07-24<br>
**Typ:** Architekturanalyse, keine neue Architekturentscheidung<br>
**Gegenstand:** Codebasis, Zielbild, Implementierungsstand, Vorläufer und
belastbare Neuheitsgrenze

**Snapshot-Grenze:** Diese Analyse ist historisch und nicht normativ. Aussagen
zum implementierten Stand beziehen sich ausschließlich auf den am 2026-07-24
untersuchten Repository-Snapshot. Der Bericht liefert keine aktuelle
Implementierungs- oder Hardwareevidenz, begründet keinen ADR-Status, akzeptiert
keine Änderung und schließt keine Checkbox.

## Kurzurteil

In raiOS steckt ein echtes und ungewöhnliches Architekturprinzip. Die
belastbare Idee ist jedoch nicht einfach:

> Das Betriebssystem erklärt sich selbst.

Diese breite Idee existiert bereits in Forschung und anderen Systemen. Die
präzisere raiOS-Idee lautet:

> raiOS besitzt ein typisiertes, evidenzgebundenes Selbstmodell. Es legt offen,
> was es weiß, woher es das weiß, was es nicht weiß und warum eine Aktion
> erlaubt oder verweigert wird. Eine Beobachtung, Zusammenfassung oder
> KI-Erklärung kann dabei niemals selbst Autorität erzeugen.

Der möglicherweise unterscheidbare Beitrag ist die durchgehende Verbindung
dieses Selbstmodells mit:

- begrenztem und nachvollziehbarem Agentenkontext,
- isoliert erzeugten Artefakten,
- exakten Artefakt-Hashes,
- Test- und Negativtest-Evidenz,
- angeforderten und lokal berechneten Capabilities,
- artefaktbezogener Zustimmung des Besitzers,
- widerrufbaren Laufzeit-Grants,
- Audit und Rollback.

Nicht jeder Baustein ist neu. Ungewöhnlich ist der Versuch, sie als ein
OS-natives Autoritätsmodell zu behandeln.

## 1. Was „selbsterklärend“ in raiOS bedeutet

„Selbsterklärend“ bedeutet nicht, dass ein LLM Logdateien in überzeugende Prosa
verwandelt. Das wäre lediglich eine weitere potenziell falsche Interpretation.

Gemeint ist eine maschinenlesbare Kette:

```text
Hardware und Dienste
        |
        v
typisierte Fakten mit Quelle, Klassifikation und Gültigkeitsbereich
        |
        v
begrenzter Agentenkontext samt expliziten Auslassungen
        |
        v
Agent schlägt Code oder eine Aktion vor
        |
        v
isolierter Build und Tests
        |
        v
exakter Artefakt-Hash + Evidenz + angeforderte Capabilities
        |
        v
artefaktbezogene Freigabe durch den Besitzer
        |
        v
lokal berechnete, widerrufbare Laufzeit-Grants
        |
        v
Audit, Revoke und Rollback aktualisieren dasselbe Selbstmodell
```

Das System soll dabei fünf verschiedene Dinge beschreiben können.

### 1.1 Aktueller Zustand

Das System beantwortet strukturierte Fragen wie:

```text
system.describe
system.snapshot
system.capabilities
system.boot_log
device.graph
service.inventory
problem.list
```

Der Agent soll nicht aus beliebigen Dateien und Logzeilen erraten müssen, ob
ein Gerät erkannt wurde, ein Dienst läuft oder eine Voraussetzung fehlt.

Die grundlegende Entscheidung steht in
`docs/architecture/decisions/0002-agent-self-description-and-live-built-modules.md:23-84`.

### 1.2 Herkunft und Stärke des Wissens

Nicht jede Aussage besitzt dieselbe Autorität. ADR 0004 unterscheidet unter
anderem:

- Core-Ledger und Grants,
- Testevidenz und Attestierungen,
- aktuelle Snapshots,
- explizite Entscheidungen,
- Dienstzustände,
- Ereignisse,
- abgeleitete Zusammenfassungen,
- semantische Suchtreffer,
- Chatverlauf.

Eine Zusammenfassung darf eine Quelle auffindbar machen, sie aber nicht
ersetzen. Ein semantischer Treffer darf auf einen Record zeigen, ist selbst
jedoch keine Autorität.

Siehe
`docs/architecture/decisions/0004-system-memory-and-agent-context.md:80-98`.

### 1.3 Begrenzter Agentenkontext

Der Agent soll nicht automatisch den gesamten Speicher, alle Logs oder alle
Geheimnisse erhalten. Stattdessen erzeugt ein lokaler Broker ein
aufgabenspezifisches Kontextpaket mit:

- Profil und Zweck,
- Tokenbudget,
- Authority-Reihenfolge,
- enthaltenen Record-IDs,
- ausgelassenen Informationen,
- Gründen für jede Auslassung,
- Verweisen zurück zur Quelle.

Der Agent kann anschließend mit `memory.trace` fragen, woher eine Information
stammt. Er soll außerdem erkennen können, dass Kontext fehlt, statt die Lücke
mit einer plausiblen Vermutung zu schließen.

Siehe
`docs/architecture/decisions/0004-system-memory-and-agent-context.md:155-287`
und `docs/architecture/device-protocol/memory-context-v0.md`.

### 1.4 Begründung einer Entscheidung

Eine Verweigerung soll kein unstrukturiertes „Permission denied“ sein.
Idealerweise nennt sie die tatsächlich fehlende Voraussetzung:

```text
artifact_hash_mismatch
vm_test_report_missing
owner_approval_missing
capability_not_granted
provider_trust_unverified
rollback_binding_missing
```

Damit wird ein Policy-Entscheidungspfad erklärbar. Das ist keine vollständige
kausale Erklärung beliebigen Systemverhaltens. Es ist eine deterministische
Antwort auf die engere Frage:

> Welche geprüfte Voraussetzung dieses Gates war nicht erfüllt?

### 1.5 Übernahme neuer Software

Vom Agenten erzeugter Quellcode beginnt als nicht autorisierende Eingabe. Erst
eine lokal überprüfte Kette soll aus einem Kandidaten ein laufendes Programm
machen:

```text
Manifest
+ exakter Artefakt-Hash
+ Basis- oder Image-Identität
+ Test- und Negativtest-Bericht
+ angeforderte Capabilities
+ lokale Attestierung
+ Zustimmung des Besitzers
= lokal berechneter Grant
```

Das Manifest darf Rechte anfordern, aber keine Rechte vergeben. Auch ein
erfolgreicher Test soll allein kein Startrecht erzeugen.

Siehe
`docs/architecture/decisions/0002-agent-self-description-and-live-built-modules.md:41-122`.

## 2. Was im Repository bereits implementiert ist

Das Prinzip ist nicht nur Website-Prosa. Mehrere zentrale Teile sind bereits
Code.

### 2.1 Native Read-only-Selbstbeschreibung

Der Kernel routet unter anderem:

- `system.describe`,
- `system.snapshot`,
- `system.capabilities`,
- `device.graph`,
- `problem.list`,
- `service.inventory`,
- `memory.profile`,
- `memory.context`,
- `memory.query`,
- `memory.trace`,
- `memory.recent_events`.

Relevante Einstiegspunkte:

- `seed-kernel/src/agent_protocol.rs:358-414`
- `seed-kernel/src/agent_protocol_system.rs:715-1211`
- `seed-kernel/src/agent_protocol_memory.rs:42-303`

### 2.2 Gemeinsames Evidenzformat

`raios.evidence_response.v1` trennt:

- Fakten,
- Evidenz,
- Entscheidung,
- Klassifikation,
- Quelle und Ereignisbezug.

Siehe `crates/raios-core/src/evidence_response.rs`.

### 2.3 Fakten dürfen keine Autorität vortäuschen

Die Projektion von Systemzustand behandelt Fakten und Entscheidungen getrennt.
Reservierte Felder wie:

```text
outcome
grants
effects
blocked_by
authorizes_*
```

dürfen nicht in einem Faktenobjekt erscheinen.

Eine Beobachtung erzeugt keine Grants und keine Effekte. Eine unbekannte
Klassifikation wird zurückgewiesen, statt stillschweigend als öffentlich oder
vertrauenswürdig behandelt zu werden.

Siehe `crates/raios-core/src/system_status_projection.rs:46-118`.

Das ist eine der stärksten bereits vorhandenen Invarianten:

> Eine Beobachtung kann strukturell keine Autorisierung behaupten.

### 2.4 Kontext mit Auslassungsgründen

`memory.context` baut bereits ein lokales Paket aus Snapshot, Capability-Katalog,
Diensten, Problemen, Boot-Zusammenfassung und dauerhaften Records. Es enthält:

- Authority-Reihenfolge,
- Ziel- und Schätzbudget,
- ausgewählte Records,
- statische und dynamische Auslassungen,
- Quellen für spätere Trace-Abfragen.

`memory.query` kennzeichnet die semantische Suche ehrlich als
`not_implemented_locator_only`.

Siehe `seed-kernel/src/agent_protocol_memory.rs:175-286`.

### 2.5 Strukturierte Build- und Testevidenz

Die VM-Harness-Berichte enthalten unter anderem Identitäten, Hashes,
ausgeführte Prädikate, Resultate und Sidecar-Bindungen. Ein rotes Prädikat
erzwingt ein rotes Gesamtergebnis.

Build-Receipts und Modulpfade binden bereits Teile von:

- Snapshot,
- Toolchain,
- Kandidatenhash,
- Manifest,
- Testbericht,
- Attestierung.

Diese Records dürfen dennoch nicht automatisch Autorität erzeugen.

### 2.6 Owner- und Recovery-Projektionen

Genesis- und Recovery-Oberflächen zeigen bereits Teile des typisierten Zustands,
Prüfergebnisse und retained feedback. Eine UI-Auswahl wird dabei nicht
automatisch als erfolgreich ausgeführte Aktion behandelt.

Relevante Pfade:

- `seed-kernel/src/shell_host/genesis.rs`
- `seed-kernel/src/shell_host/recovery.rs`

## 3. Was noch nicht erreicht ist

Das vollständige Zielbild ist noch nicht geschlossen.

### 3.1 Keine vollständige Identität des laufenden Systems

`system.describe` liefert derzeit im Wesentlichen Produkt, Stage,
Protokollmodus und Methodennamen. Es fehlen in dieser zentralen Antwort unter
anderem:

- Kernel-Build-ID,
- Image-Hash,
- Boot-ID,
- Schema-Hashes,
- Capture-Epoch,
- genaue Semantik jeder Methode.

Ohne diese Bindung kann eine Erklärung noch nicht zweifelsfrei dem exakten
laufenden Artefakt zugeordnet werden.

### 3.2 Zentrale Beobachtungen sind noch nicht atomar evidenzgebunden

Mehrere Snapshot-Felder werden nacheinander gelesen. Die Antwort besitzt keinen
sichtbaren gemeinsamen Aufnahmezeitpunkt. Ein Konsument kann daher nicht
beweisen, dass alle Felder denselben Systemmoment beschreiben.

Außerdem werden zentrale Beobachtungsantworten teilweise mit leerer
Evidenzliste und ohne Event-Bindung ausgegeben.

### 3.3 Die Problemabdeckung ist endlich und teilweise fest codiert

`problem.list` erkennt eine definierte Menge bekannter Zustände. Eine leere
Treffermenge darf deshalb nur bedeuten:

> Keine Funde durch Detector-Set X in Version Y.

Sie darf nicht als „Das System ist gesund“ dargestellt werden.

Die Website-Aussage „Jeder Zustand, jedes Gerät, jedes Problem“ in
`raios-ui-lab.html:2110-2116` beschreibt daher das Zielbild, nicht den aktuellen
Stand.

### 3.4 Capability-Katalog und tatsächliche Grants sind noch leicht verwechselbar

Das Feld `capabilities` im Snapshot enthält derzeit alle bekannten
Capability-IDs, auch verweigerte. Nur die gesonderte Capability-Antwort erhält
den Status `granted` oder `denied`.

Eine UI oder ein Agent könnte das Vorhandensein einer ID fälschlich als
Verfügbarkeit interpretieren.

### 3.5 Nicht jede strukturierte Verweigerung ist bereits die echte Ursache

Einige Methoden verwenden noch generische Listen von Voraussetzungen. Eine
strukturierte Standardantwort ist besser als freie Prosa, aber noch keine
präzise Erklärung, wenn die genannten Voraussetzungen nicht zum konkreten Gate
gehören.

### 3.6 Die Provider-Grenze bleibt absichtlich geschlossen

Der lokale `provider_minimal`-Kontext existiert, wird aber nicht automatisch an
einen Provider übertragen. Provider-Kontextinjektion bleibt deaktiviert, bis
Trust, Klassifikation, Redaction und Audit-Bindung positiv sind.

Siehe
`docs/architecture/device-protocol/provider-context-export-v0.md`.

### 3.7 Der generische positive Promotionspfad ist nicht vollständig live

Read-only-Inspektion, Reports und verschiedene begrenzte Pfade existieren. Der
vollständig generische Ablauf:

```text
Kandidat
-> Build
-> Test
-> Attestierung
-> Owner-Freigabe
-> Grant
-> Start
-> Revoke
-> gemeinsamer Artefakt-, Grant- und Zustands-Rollback
```

ist noch nicht als ein universeller Pfad geschlossen.

Der Scope enthält deshalb weiterhin offene Punkte, insbesondere beim
durchgängigen Grant-Audit, signierten und reproduzierbaren Builds sowie
vollständigen Maschinenmanifesten.

## 4. Was das System beweisen kann und was nicht

raiOS kann für modellierte Eigenschaften deterministische Aussagen liefern.

Beispielsweise:

- Genau dieser Artefakt-Hash wurde geprüft.
- Der Bericht gehört zu dieser Basis- oder Image-Identität.
- Diese Tests und Negativtests wurden ausgeführt.
- Das Wasm-Modul besitzt nur diese sichtbaren Imports.
- Für diese Capability existiert kein Grant.
- Diese Person hat genau dieses Artefakt freigegeben.
- Ein Revoke hat die zugehörigen Handles entzogen.
- Ein Rollback hat eine bestimmte Vorgängerversion aktiviert.

Es kann daraus nicht allgemein folgern:

- Der Code ist vollständig sicher.
- Der Code besitzt keine bösartige Logik.
- Ein bestandener Test deckt jedes zukünftige Verhalten ab.
- Ein laufendes, kompromittiertes TCB berichtet noch wahrheitsgemäß über sich.
- Keine unbekannte Fehlerklasse existiert.

Der Scope formuliert das korrekt: Reports, Rollback und Escape-Negativtests sind
ein dokumentierter Ersatz für fehlende mathematische Vollbeweise, nicht deren
Äquivalent.

Siehe `docs/SCOPE.md:27-29`.

## 5. Warum die Idee trotzdem außergewöhnlich ist

In klassischen Systemen liegen die relevanten Funktionen häufig in getrennten
Schichten:

- Observability beschreibt den Zustand.
- Ein Paketmanager oder Deployment-System prüft Artefakte.
- Sandbox, Container oder Kernel erzwingen Rechte.
- Eine andere UI fragt nach Zustimmung.
- Ein Audit-System speichert Ereignisse.
- Ein Rollback-System verwaltet Versionen.
- Ein LLM interpretiert anschließend Logs und Dokumentation.

Diese Schichten können unterschiedliche Identitäten, Schemata und
Wahrheitsbegriffe verwenden. Genau an diesen Übergängen entstehen
Missverständnisse und Umgehungsmöglichkeiten.

raiOS versucht stattdessen, denselben typisierten Record durch den gesamten
Lebenszyklus zu tragen:

```text
Beobachtung
-> Kontext
-> Vorschlag
-> Prüfung
-> Entscheidung
-> Laufzeitautorität
-> Erklärung
-> Audit
-> Revoke oder Rollback
```

Der ungewöhnliche Teil ist deshalb nicht, dass ein System Statusdaten besitzt.
Der ungewöhnliche Teil wäre:

> Derselbe evidenzgebundene Datensatz informiert den Agenten, erklärt dem
> Besitzer die Entscheidung, bestimmt die tatsächlich laufenden Rechte und
> bleibt anschließend Grundlage für Audit, Revoke und Rollback.

Man könnte dies als **agentenlesbare, evidenzgebundene Autoritätsebene** oder
als **evidence-bound OS control plane** bezeichnen.

## 6. Engste Vorläufer

Die breite Behauptung „So etwas hat noch niemand gemacht“ ist nicht
verteidigbar.

### 6.1 Microsoft Singularity

Singularity ist der unangenehm nahe historische Vorläufer. Bereits 2004 wurde
ein gesamtes Singularity-System als „self-describing artifact“ beschrieben.
Spätere Arbeiten verbanden:

- manifestbasierte Programme,
- deklarierte Ressourcen und gewünschte Capabilities,
- Verifikation vor Installation und Start,
- isolierte Prozesse,
- vertraglich definierte Kommunikation,
- einen Gatekeeper, der Ausführung verweigern konnte.

Quellen:

- https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/tr-2004-105.pdf
- https://www.microsoft.com/en-us/research/publication/singularity-rethinking-the-software-stack/
- https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/eurosys2006_solvingthestartingproblem.pdf

Die raiOS-Differenz müsste daher enger sein: agentenspezifischer Kontext,
unabhängige Test- und Negativtest-Evidenz, artefaktbezogene
Owner-Promotion sowie die Kontinuität derselben Bindung bis zu Runtime-Grants
und Rollback.

### 6.2 Models@run.time und Self-Explanation

Die Forschung zu `models@run.time` beschreibt seit vielen Jahren
kausal verbundene Selbstrepräsentationen laufender Systeme. Auch
„self-explanation“ auf Grundlage von Runtime-Modellen und Evidenz wird
ausdrücklich behandelt.

Quellen:

- https://publications.aston.ac.uk/id/eprint/31845/1/The_Role_of_models_run.time_in_Autonomic_Systems.pdf
- https://publications.aston.ac.uk/id/eprint/37117/

raiOS erfindet daher weder das laufende Selbstmodell noch den Begriff
Selbsterklärung.

### 6.3 Genode/Sculpt

Sculpt besitzt bereits:

- capability-basierte Isolation,
- einen lebenden Komponenten- und Beziehungsgraphen,
- strukturierte Systemreports,
- dynamische Deployment-Konfiguration,
- Updates und Rollback.

Quelle:

- https://genode.org/download/sculpt

Sculpt ist vermutlich der engste existierende Vergleich für eine
nutzerverständliche, capability-basierte Systemstruktur.

### 6.4 Fuchsia

Fuchsia verbindet:

- Komponentenmanifeste,
- explizites Capability-Routing,
- Komponenten-Topologie,
- strukturierte Inspect-Daten,
- Lifecycle-Management.

Quellen:

- https://fuchsia.dev/fuchsia-src/concepts/components/v2
- https://fuchsia.dev/docs/concepts/components/v2/capabilities/README
- https://fuchsia.dev/fuchsia-src/reference/diagnostics/inspect/tree

### 6.5 WebAssembly Component Model

Wasm Components sind bereits selbstbeschreibend bezüglich ihrer typisierten
Imports und Exports. Ohne einen Import existiert kein entsprechender Weg nach
außen.

Quellen:

- https://component-model.bytecodealliance.org/design/components.html
- https://component-model.bytecodealliance.org/design/worlds.html

Das Component Model definiert aber bewusst keine vollständige Semantik für
Provenienz, Owner-Freigabe, Grant-Lifecycle oder Rollback.

### 6.6 Nix, Guix und OSTree

Diese Systeme liefern unveränderliche oder adressierte Artefakte, Generationen,
atomare Aktivierung und Rollback. Sie lösen wichtige Teile der
Artefaktidentität und Reproduzierbarkeit.

Sie verbinden diese Artefakte jedoch nicht automatisch mit einem
feingranularen Runtime-Capability-Grant.

### 6.7 in-toto, SLSA, Sigstore und Binary Authorization

Diese Systeme verbinden:

- Artefakt-Digests,
- Build-Provenienz,
- autorisierte Funktionäre,
- signierte Attestierungen,
- Admission-Entscheidungen vor einem Deployment.

Quellen:

- https://in-toto.io/docs/getting-started/
- https://slsa.dev/spec/v1.2/build-provenance
- https://docs.cloud.google.com/docs/security/binary-authorization-for-borg

Die Pipeline endet typischerweise an der Deployment-Grenze. Ein gemeinsames
Modell für objektbezogene Runtime-Grants, Widerruf und Zustands-Rollback ist
nicht ihr Hauptgegenstand.

### 6.8 Qubes, seL4 und Proof-Carrying Code

Qubes bietet starke Domain-Isolation, Policy-Regeln, Ask/Allow/Deny und
Disposable VMs. seL4 und capDL liefern formal analysierbare
Capability-Strukturen. Proof-Carrying Code bindet Code an maschinenprüfbare
Beweise enger Sicherheitseigenschaften.

Diese Systeme zeigen, dass Isolation, Capabilities, Erklärbarkeit von Policy
und sichere Aufnahme fremden Codes keine neuen Einzelideen sind.

## 7. Belastbare Neuheitsgrenze

Nicht beansprucht werden sollte:

- das erste selbstbeschreibende Betriebssystem,
- die ersten selbstbeschreibenden Artefakte,
- die erste capability-basierte Isolation,
- die erste erklärbare Policy-Entscheidung,
- die erste hashgebundene Build-Provenienz,
- die erste menschliche Freigabe vor Ausführung,
- das erste Rollback-System,
- der erste sichere Agent,
- die erste Kombination dieser Ideen überhaupt.

„Erstmals kombiniert“ wäre ohne vollständige systematische Literatur-, Patent-
und Marktprüfung zu stark. Der folgende Vergleich ist auf die ausdrücklich
genannten Systeme und Quellen begrenzt.

Die maximal belastbare Formulierung lautet:

> raiOS untersucht ein OS-natives, lokales Promotions- und Autoritätsprotokoll
> für agentengenerierte Komponenten. Code bleibt nicht-autorisierende Eingabe,
> bis der lokale Referenzmonitor den exakten Artefakt-Digest mit Basiszustand,
> Test- und Negativtest-Evidenz, angeforderten Capabilities und einer
> artefaktgebundenen Zustimmung des Besitzers verbindet. Daraus werden
> widerrufbare Laufzeit-Grants abgeleitet. Dieselbe Bindung bleibt als
> Erklärung, Audit-, Revoke- und Rollback-Struktur erhalten.

Noch vorsichtiger:

> Unter den untersuchten Systemen wurde kein einzelnes gefunden, das genau
> diese lokale, artefakt-, test-, owner-, grant- und rollbackgebundene
> Transaktion vollständig als agentenlesbares OS-Protokoll integriert.

Das ist ein innerhalb dieser begrenzten Analyse belastbarer Recherchebefund,
aber kein Beweis eines weltweiten „First“.

## 8. Empfohlene öffentliche Formulierungen

### Sehr kurz

> raiOS macht nicht die KI zur Wahrheitsquelle. Es macht das Betriebssystem zu
> einer überprüfbaren Quelle, aus der die KI lesen und gegen die sie nur
> Vorschläge einreichen darf.

### Zum selbsterklärenden System

> Das System legt offen, was es weiß, woher es das weiß, was es nicht weiß und
> warum ein Policy-Gate eine Aktion erlaubt oder verweigert.

### Technischer

> raiOS explores an agent-facing, evidence-bound OS control plane in which
> observations cannot authorize actions, generated code begins as inert data,
> and runtime authority is derived locally from an exact artifact, test
> evidence, requested capabilities and owner approval.

### Ehrliche Neuheitsformulierung

> The individual mechanisms are not new. The experiment is whether
> self-description, evidence-bound promotion, capability derivation and
> rollback can become one native authority model instead of several optional
> tools layered on top of a conventional OS.

## 9. Voraussetzungen für einen starken späteren Claim

Bevor raiOS überzeugend von einem selbsterklärenden Autoritätssystem sprechen
kann, sollten mindestens folgende Invarianten geschlossen sein:

1. Jede relevante Antwort bindet sich an Boot-ID, Build-ID und Image-Hash.
2. Jeder Fakt besitzt Quelle, Klassifikation, Capture-Epoch, Scope und
   Freshness.
3. Die Abdeckung jedes Detektors ist sichtbar und versioniert.
4. Fakten, Zusammenfassungen und LLM-Prosa können niemals Grants erzeugen.
5. Mutationen prüfen einen authentifizierten Subject-Grant und nicht nur einen
   Capability-Namen.
6. Jeder Denial-Grund stammt aus dem tatsächlich ausgeführten Evaluator.
7. Provider-Ausgaben zitieren die verwendeten Record- und Evidence-IDs.
8. Unbekannte, widersprüchliche oder veraltete Evidenz führt zu
   `unknown` oder `denied`.
9. Der generische Promotionspfad ist end-to-end geschlossen.
10. Rollback stellt Artefakt, Capability-Satz und kompatiblen Zustand gemeinsam
    wieder her.
11. Der Auditpfad beantwortet dauerhaft „wer, was, wann und warum“.
12. Kritische Selbstaussagen besitzen eine unabhängige Vertrauenswurzel, etwa
    gemessenen Boot, signierte Image-Identität oder einen externen Verifier.

## Schlussfolgerung

Die Codebasis enthält bereits mehr als eine visuelle Idee. Sie besitzt die
Anfänge eines echten, typisierten Selbstmodells und eine wichtige
Sicherheitsinvariante: Beobachtung ist keine Autorität.

Die Formulierung „Das System erklärt sich selbst“ ist als Leitbild verständlich,
aber technisch zu breit. Präziser wäre:

> raiOS exposes what it currently knows, how it knows it, what it omitted, and
> why a policy gate denied, within explicitly declared coverage.

Wenn zusätzlich die vollständige Promotions-, Grant-, Audit- und
Rollback-Kette geschlossen wird, entsteht daraus tatsächlich eine
bemerkenswerte Architektur. Nicht weil kein einzelner Baustein vorher
existierte, sondern weil Zustandswissen, Agentenkontext, Softwareaufnahme und
Laufzeitautorität dieselbe überprüfbare Sprache sprechen.
