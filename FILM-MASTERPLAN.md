# FILM-MASTERPLAN — Kino-Loop XXL: „The Factory Moves In"

Stand 2026-07-19 · Plan: Orchestrator (Fable 5, max effort) ·
Zweitmeinungen: Codex A (unabhängiges Storyboard, xhigh) + Codex B
(Lücken-Audit Site vs. Repo, xhigh) · Beweis-Inventar: Explore-Sweep über
STATUS/SCOPE/Reports (445 `shadow-*.json`).

Dieses Dokument ist der komplette Bauplan für die neue Riesen-Animation der
raiOS-Website (`raios-ui-lab.html?site=1`). Es ersetzt nichts automatisch —
es ist die Vorlage, aus der Lanes die Szenen bauen.

---

## 0. Beschlossene Eckdaten (Owner, 2026-07-19)

| Frage | Entscheidung |
| --- | --- |
| Publikum | **Technik-Publikum (Devs)** — Fachbegriffe ok, Architektur + Beweise im Fokus |
| Format | **Kino-Loop XXL** — läuft von selbst, wie der 32s-Opener, nur groß |
| Länge | **~2 Minuten** (120-s-Master-Uhr, nahtloser Loop) |
| Wahrheits-Ebene | **Nur Zielbild** — der Film erzählt das fertige raiOS; Ehrlichkeit wohnt separat (Kapitel + „Ehrliche Bilanz" bleiben darunter) |
| Roter Faden | **„Die Fabrik zieht ein"** — der Selbstbau-Kreislauf ist der Held |
| Screen-Material | **Mix** — isometrische Welt für die Maschine, echte Dream-UI-Screens, wenn die Kamera in Bildschirme hineinzoomt |
| Integration | **Ersetzt den 32s-Opener**; die ~13 Einzel-SVG-Kapitel bleiben als vertiefende Doku darunter |
| Sprache | **Englisch** (Film-Labels, getippte Sätze); die deutsche Site drumherum bleibt |

**Owner-Bildsprache (Ansage vom 2026-07-19, bindend für die Welt):**
Türen sitzen auf den Blöcken und werden mit Schlüsseln aufgeschlossen; die
Türen liegen **immer auf dem Boden, der Genesis-Schicht**; einmal geöffnete
Verbindungen bleiben sichtbar bestehen — **ein pulsierendes Kabel** liegt
zwischen den Partnern durch die offene Tür; es darf **mehr Platz** geschaffen
und die **Blöcke dafür redesignt** werden.

---

## 1. Die Idee in drei Sätzen

Ein Satz fällt in ein Betriebssystem, und wir sehen zu, wie die Maschine sich
die Fabrik dafür selbst hereinholt: Bauplatz, Compiler, Prüfkeller, Tor.
Alles, was Rechte hat, hat sie sichtbar — als Schlüssel, Tür im Boden und
pulsierendes Kabel; alles, was läuft, hat vorher ein Tor mit Beweisen
passiert. Am Ende steht keine App, sondern ein Kreislauf: das frisch gebaute
Werkzeug macht das nächste leichter — und der Loop beginnt von vorn.

**Arbeitstitel / Titelkarte am Loop-Ende:**
`raiOS — the OS that builds its own software.`
Untertitel: `Every gate you just saw is a real test.`

---

## 2. Bildsprache-Gesetzbuch (die Grammatik der Welt)

Diese Regeln gelten in JEDER Szene. Sie sind keine Deko — jede Regel bildet
eine echte Architektur-Eigenschaft ab. Wer eine Szene baut, baut gegen dieses
Gesetzbuch.

### 2.1 Der Boden = Genesis-Schicht
- Der Boden ist eine isometrische Platten-Ebene. Er **wächst**, wenn das
  System wächst (Platten materialisieren einzeln, mit leichtem Aufglühen).
- **Alle Türen gehören dem Boden, nicht den Blöcken.** Ein Block kann seine
  Tür nicht selbst öffnen — der Schlüssel steckt im Boden-Sockel, und der
  Boden öffnet. (Echt: Import-Grants vergibt die Genesis-Schicht; ein Gast
  kann sich keine Fähigkeit selbst geben.)
- Owner-Beobachtung bestätigt: „Türen immer auf dem Boden" ist klug — es
  codiert exakt die echte Regel, dass ein Gast NUR über gewährte Imports
  nach draußen kommt. Es gibt keine Seitenfenster.

### 2.2 Blöcke = Domänen (Redesign, mehr Platz)
Einheitliche Block-Grammatik (neu, ersetzt die bisherigen kleineren Blöcke):
- **Sockel-Ring:** Jeder Block steht auf einem Sockel mit sichtbaren
  **Tür-Sockeln** (Import-Steckplätzen) an der Basis. Nicht gewährte Türen
  **existieren nicht** — kein grauer Platzhalter, einfach Wand. (Echt: ein
  nicht gewährter Import ist keine verschlossene Leitung, er ist gar keine
  Leitung — Link-Failure vor Instanziierung.)
- **Status-Lampe** oben: ruhiges Atmen = läuft; rot-Blitz + Erstarren = Trap.
- **Versions-Kerben** an einer Kante: jede installierte Version eine Kerbe;
  Rollback = oberste Kerbe klappt weg, darunter leuchtet die vorige.
- **Größe = gewährter Speicher.** Der Bauplatz-Block ist der mit Abstand
  größte Block der Welt (sein 1-GiB-Fenster ist real); kleine Programme sind
  sichtbar klein (ein 176-Byte-Editor ist ein Würfelchen).
- Mehr Platz: Der Boden wird ~2× breiter als im 32s-Opener; Blöcke stehen
  mit Luft, damit Kamera-Fahrten zwischen ihnen möglich sind.

### 2.3 Schlüssel & Türen = Capabilities
- **Grant-Zeremonie:** Schlüssel steigt aus dem Boden → dreht sich im
  Boden-Sockel → Tür-Segment im Boden gleitet auf → erst jetzt kann ein
  Kabel hindurch. Immer diese drei Schritte, immer in dieser Reihenfolge.
- **Revoke/F12:** Tür gleitet zu → Kabel wird am Boden-Bündig abgetrennt und
  zieht sich in den Boden zurück → Block-Lampe aus. Kein Fade — ein Schnitt.

### 2.4 Kabel = lebende Verbindungen (Owner-Wunsch, erweitert)
- Eine einmal geöffnete Verbindung **bleibt sichtbar**: ein Kabel liegt durch
  die offene Boden-Tür und **pulsiert** im Takt des Verkehrs (Puls-Welle
  läuft in Übertragungsrichtung; idle = langsames Glimmen).
- **Erweiterung (Empfehlung):** Kabel laufen nie oberirdisch von Block zu
  Block. Sie tauchen durch die Tür in den Boden und laufen als **leuchtende
  Adern IM Boden** (flache Glas-Kanäle in den Platten) zum Ziel. Begründung —
  und das ist der eigentliche Witz: **auch die Verbindungen gehören der
  Genesis-Schicht.** Jeder Verkehr ist im Boden sichtbar = alles ist
  vermittelt, geloggt, entziehbar (echt: Kernel vermittelt jeden Host-Zugriff,
  RECLOG zeichnet auf). Block-zu-Block-Direktkabel über der Oberfläche wären
  eine Architektur-Lüge (Gäste können einander nicht benennen).
- Die **Agent↔Netz-Ader** (Owner-Beispiel) ist die erste und bleibt den
  ganzen Film über sichtbar am Pulsieren — sie etabliert die Regel.

### 2.5 Kernel = Fels mit Hardware-Kristallen
- Unter dem Boden liegt der Kernel als dunkler Fels-Sockel. **In** den Fels
  eingelassen: Hardware-Kristalle (CPU, RAM, NIC, USB, Display, WiFi-Chip).
- **Wichtig (ADR 0005/0015, bindend seit 2026-07-19):** Treiber wohnen IM
  Fels. Es gibt KEINE Treiber-Domänen-Blöcke auf dem Boden. Blöcke bekommen
  Hardware nur als Tür+Ader zum jeweiligen Kristall. Der Film zeigt damit
  die echte Ziel-Architektur — nicht das alte Mikrokernel-Bild, das noch in
  einigen Site-Kapiteln steht (siehe §9).
- Der Fels hat einen **Prüf-Keller** (Kammer im Boden/Fels, siehe Szene 9) —
  das übernimmt die bewährte Opener-Idee „geprüft wird im Boden".

### 2.6 Die Kapsel = der Wunsch und sein Hash
- Der getippte Satz faltet sich zur **Kapsel** — dem Kontinuitäts-Token des
  Films (Codex-A-Idee, übernommen). Die Kapsel trägt ab der ersten
  Verwandlung einen **Hash-Streifen** (Kurz-Hex), der sich bei jeder
  Verwandlung sichtbar ableitet (Antwort → Revision → Kandidat → Programm).
  Devs lesen: Herkunft ist durchgehend, nichts wird heimlich ausgetauscht.
- Die Kamera folgt IMMER der Kapsel oder einem von ihr abgeleiteten Objekt.
  Keine Szene ohne Kapsel-Bezug (Anti-Pattern „27 hübsche Inseln").

### 2.7 Stempel, Tor, Ring
- **Beweis-Ring:** Um einen Kandidaten schließen sich vier Segmente:
  `manifest` · `artifact hash` · `green report` · `owner approval`. Nur der
  geschlossene Ring öffnet das **Tor** (Trust-Pipeline-Bogen). Fehlt ein
  Segment, fällt das Tor hart zu.
- **Grüner Haken existiert nur als Stempel-Abdruck** — nie als Klick-Häkchen
  (das ist die Outro-Pointe der Site, sie gilt auch im Film).

### 2.8 Farb- und Text-Regeln
- Palette der Site/des Openers weiterverwenden (dunkles Blau-Schwarz,
  `#4897F2`-Akzent, warmes Amber für Schlüssel/Stempel, klares Grün NUR für
  bestandene Prüfungen, Rot NUR für Traps/Denials).
- Alle Film-Texte **Englisch**, knapp, monospace für alles Getippte.
  System-Wortlaute sind, wo es sie gibt, die ECHTEN (siehe §4) — ein Dev,
  der das Repo grept, findet jede Zeile wieder.

### 2.9 Was der Film NIE zeigt (harte Taboos)
1. Keine Treiber-Blöcke auf dem Boden (ADR 0005 — Treiber sind im Fels).
2. Keine oberirdischen Block-zu-Block-Kabel.
3. Kein Programm, das ohne Tor/Ring zu laufen beginnt.
4. Keine Tür, die sich ohne Schlüssel-Zeremonie öffnet.
5. Kein grüner Haken ohne Stempel-Abdruck.
6. Der Agent ist nie „Zauberer": Er schreibt und schlägt vor — öffnen tun
   immer Boden (Policy) oder Mensch (Klick).

---

## 3. Das Storyboard — 14 Szenen auf der 120-Sekunden-Uhr

Drei Akte. Zeiten sind Richtwerte für das Animatic (Phase P2), ±2 s je Szene
ist erlaubt, die Akt-Grenzen stehen fest. Der Loop ist nahtlos: Szene 14
endet im Bild von Szene 1.

### AKT I — DER WUNSCH (0–25 s)

**S1 · 0–6 s · „Prompt"**
Kamera: statisch, frontal auf den Website-Hintergrund. Mittig steht nur eine
einzelne dunkle Prompt-Box mit blinkendem Cursor; kein Dashboard, keine
Navigation, keine Statusleisten und kein weiterer In-Scene-HUD.
Es tippt: `> build me a music player` (Anschluss an die bestehende
Site-Ikonografie). Enter. Der Satz löst sich vom Screen.
Screen-Einbau: bewusst minimal (nur die Eingabe).
Label: keins — der Satz IST das Label.

**S2 · 6–14 s · „The world under the glass"**
Kamera: Rückwärtsfahrt durch den Screen hindurch; das UI wird zur
Glas-Oberfläche, darunter materialisiert die isometrische Welt: erst der
anfangs kompakte **Rust-Kernel** mit seinen **Hardware-Kristallen** (CPU · RAM
· NIC · USB · DISPLAY · WIFI), darauf links das **Genesis-Deck**. Der Kernel
ist hier nur so groß, dass Genesis und die später sichtbare NET-Station sicher
auf ihm stehen; eine Builder-Fläche ist noch nicht sichtbar.
Labels: `RUST-KERNEL — owns the hardware` · `GENESIS DECK — grants every door`.

**S3 · 14–25 s · „The agent needs the net"**
Der **Agent-Block** steigt aus dem Boden (mittelgroß, ruhige Lampe; rein
abstrakter Block — keine Gesichts-/Figur-Anmutung, Owner 2026-07-19). Die
Er braucht Wissen: **Schlüssel-Zeremonie** am Boden
(`cap: net.https` am Schlüsselbund-Anhänger), Tür gleitet auf, ein Kabel
taucht ein und läuft als **pulsierende Ader im Boden** zum **NET-Turm** am
Horizont (Internet als ferne Lichter). Die Ader bleibt ab jetzt dauerhaft am
Pulsieren (Owner-Regel). Der Agent holt sich die Antwort: ein Daten-Puls
kommt zurück.
Labels: `key granted: net.https` · `the connection stays — and stays visible`.

### AKT II — DIE FABRIK (25–80 s)

**S4 · 25–33 s · „Unlock the builder deck"**
Die Kamera bleibt zunächst eng auf Genesis. Der Agent stellt seinen
`build.request` durch eine echte Tür; erst danach fährt die Kamera heraus.
Derselbe Rust-Kernel zieht sich nun ein zweites Mal seitlich auf und wird zum
durchgehenden, dickeren Fundament für zwei gleich große Decks. Die versiegelte
Builder-Lage wird aufgeschlossen, danach wächst rechts das blau schraffierte
**Builder-Deck** auf exakt derselben Deckhöhe wie Genesis. Erst darauf steigt
der **Bauplatz-Block** mit seinem 1-GiB-Fenster. Eine einzelne isometrische
Brücke verbindet beide Decks; darauf stehen die drei kleineren Tore in sauberer
Reihe: `read: /src` · `read: /sysroot` · `write: /out`. Eine vierte Stelle
bleibt **Wand — kein Netz-Sockel vorhanden**; kurzes Aufblinken der
Beschriftung: `no net socket. builds are offline by design.`
(Echt: Bauplatz-Gäste haben exakt 30 WASI-Imports und keinen Netz-Import.)

**S5 · 33–41 s · „The answer becomes inert source"**
Die Provider-Antwort zerfällt über dem Boden in Dateien → jede wird gehasht
(kurzer Hex-Stempel) → Blobs setzen sich zu einem **Baum** zusammen →
`revision 1`. Die Revision fährt als Glas-Container durch die `/src`-Tür in
den Bauplatz. Alles hinter Glas, Aufschrift `INERT`.
Labels: `hashed · content-addressed · immutable` · `stored ≠ executable`.

**S6 · 41–52 s · „The compiler moves in"** — das Herzstück.
Durch die Boden-Adern strömen **CAS-Chunks** in den Bauplatz und setzen sich
zu einem massiven Objekt zusammen: **rustc, 91 MB**. Um seinen Sockel rasten
**30 Import-Stecker** einzeln ein (schnelles Klick-Klick-Klick, Zähler läuft
`imports 1..30`). Speicherwände wachsen (`pages 399 → …`). Dann Kamera-Zoom
IN den Bauplatz-Screen (echte Terminal-Optik):
`$ rustc --version` → `rustc 1.83.0-dev` →
`$ rustc hello.rs --target wasm32-wasip1` → Zahnräder: `parse` · `resolve
std` · `typecheck` · `emit` → durch die `/out`-Tür fällt **`hello.wasm`**
als frisch geprägte kleine Kapsel. (Zielbild-Vollendung des realen Wegs.)
Labels: `the compiler is cargo, not magic: 91 MB, 30 imports, no net`.

**S7 · 52–62 s · „The loop that fixes itself"**
Erster Prüf-Durchlauf im Bauplatz: rote Klappe — `check failed:
Cargo.lock missing`. Aus der Prüfzelle verlassen **genau vier kleine
Karten** einen schmalen Schlitz: `check id` · `revision hash` · `tree hash` ·
`reason`. Sie wandern durch die Boden-Ader zurück zum Agenten — sonst nichts
(kurzes Bild: Quelldateien und Log-Rollen prallen am Schlitz ab).
Der Agent formt `revision 2` (sichtbar als **Kind** an `revision 1`
gekettet), gleiche Prüfung: grün.
Labels: `the agent learns from four facts — never from your files`.

**S8 · 62–72 s · „Build twice, believe once"**
Der Bauplatz spiegelt sich: zwei identische Zellen, gleiche Revision, gleiche
Inputs, zwei Builds laufen parallel (Puls-Adern takten synchron). Zwei
Output-Hashes fahren aufeinander zu → **EQUAL** → das Egress-Gate öffnet.
Kurzer Alternativ-Blitz: ein Byte weicht ab → beide Outputs versiegeln rot.
Labels: `same input, twin builds, one truth` · `one differing byte = no exit`.

**S9 · 72–80 s · „The proof cellar"**
Der Kandidat sinkt in den **Prüf-Keller** im Boden (Anschluss an die
Opener-Idee „geprüft wird im Boden"). Drei Angriffe als Blitz-Sequenz:
ein Probe-Arm greift **über die Speicherwand** → Trap-Blitz, Wand steht;
ein Griff nach einer **nicht existierenden Tür** → Wand, `no such door`;
eine **Endlosschleife** → Fuel-Anzeige läuft leer, Zelle parkt.
Jede Abweisung schreibt eine Zeile in den **RECLOG-Fluss** — eine leuchtende
Schriftader im Boden, die sichtbar NUR anwächst (append-only).
Labels: `attacks are part of the test suite` · `every denial is a record`.

### AKT III — TOR, LAUF, ROLLBACK, KREIS (80–120 s)

**S10 · 80–88 s · „The ring closes"**
Vor dem **Tor** (Trust-Pipeline-Bogen) schweben die vier Ring-Segmente an den
Kandidaten: `manifest` · `artifact hash` · `green report` · `owner approval` —
das vierte Segment fehlt noch und blinkt leer. Das Tor bleibt zu.
Label: `claims open nothing`.

**S11 · 88–96 s · „The physical click"**
Kamera fährt aus der Welt heraus ans Glas: echter Genesis-Screen, der
Approve-Dialog (echte Optik): Name, Hash, angeforderte Türen, `Approve +
run program`. Ein Zeiger klickt — **physisch, am Gerät**. Das vierte
Ring-Segment rastet ein, der Ring schließt sich, das Tor öffnet.
(Seitlich prallt ein serieller Fernstart-Versuch ab: `remote start denied`.)
Label: `the last key is a human`.

**S12 · 96–106 s · „Running in its cage"**
Auf dem Boden steigt der **Programm-Block** (klein!) mit exakt seinen Türen:
`fb region` · `input` · (eine) `file door`. Kamera-Zoom in seinen Screen:
der **Music Player läuft in echter Dream-UI-Optik**, Tasten-Events fließen
als Punkte durch die Input-Ader hinein. Über allem, dezent: die rote
**F12-Notleine**. Kurzer Beweis-Moment: ein Nachbar-Block (irgendein
Experiment) erstarrt rot → Tür zu, Kabel gekappt, Block sinkt weg, frischer
Block steigt auf — die Welt zuckt nicht.
Labels: `exactly the doors it earned — nothing else exists for it` ·
`a crash costs one block, never the house`.

**S13 · 106–114 s · „The factory stays"**
Zeitraffer-Totale: Der Boden wächst weiter, neue Blöcke steigen (editor ·
synth · profiler …), Adern pulsieren, Versions-Kerben stapeln sich. Ein
Block-Update schlägt fehl: Hash-Bruch-Blitz → oberste Kerbe klappt weg,
vorige Version leuchtet wieder, eine **Tombstone-Platte** setzt sich in den
Boden. Der Bauplatz brummt durchgehend weiter — die Fabrik wohnt jetzt hier.
Nach getaner Arbeit dunkelt das Builder-Deck nur in einen Ruhezustand ab;
Deck, Brücke und Werkzeuge bleiben bestehen und werden nicht zurückgebaut.
Labels: `rollback is one transaction` · `every tool makes the next one cheaper`.

**S14 · 114–120 s · „The loop"**
Kamera zieht ganz heraus: die Welt als atmender Organismus auf dem Fels; die
Stationen glühen nacheinander als **Kreis** auf (wish → build → prove → gate →
run → keep/roll back → wish). Titelkarte:
`raiOS — the OS that builds its own software.`
`Every gate you just saw is a real test.`
Die Totale blendet in den dunklen Genesis-Screen von S1, der Cursor blinkt,
ein neuer Satz beginnt sich zu tippen → nahtloser Loop.

---

## 4. Echte Screens & echte Wortlaute (Asset-Liste)

Der Film ist Zielbild — aber seine Textur ist wahr. Alle Wortlaute, Zahlen
und UI-Formen kommen aus dem echten System (Dev-Publikum liest mit):

**Echte UI-Screens (Dream-Optik aus dem UI-Lab nachbauen):**
- Genesis-Composer + Conversation (S1, S11): `ui-lab/surfaces/dream.js` /
  `genesis.js` sind die Referenz für Panels, Farben, 8x8-Schrift-Anmutung.
- Approve-Dialog (S11): Name · Hash · angeforderte Capabilities · Button
  `Approve + run program` (Wortlaut existiert real, s. OWNER_DASHBOARD).
- Bauplatz-Terminal (S6): monospace, echte Zeilen (unten).
- Music-Player-Screen (S12): neue Dream-konforme Miniatur (Tokens der Site).

**Echte Wortlaute/Zahlen als Film-Textur:**
- `rustc 1.83.0-dev` (real: `RAIOS_RUSTCSTDOUT len=17 text=rustc 1.83.0-dev.`,
  exit 0 — shadow-20260719-122823, Commit 4716732).
- 91-MB-Compiler / 1457 CAS-Chunks / 71-MB-Sysroot / 1161 Chunks /
  `imports 1..30` / `pages 399→…` (STATUS, shadow-20260719-040901/-030038).
- Vier-Felder-Feedback: `check id · revision hash · tree hash · reason`
  (B2.2a, shadow-20260717-142445, 654/654).
- Trap-Zeile Stil: `RAIOS_ISOLATION … logged=1 host_effect=0`
  (shadow-20260719-084519 / -104006).
- `12+30=42` als Easter-Egg auf einem kleinen Block-Screen in S13
  (Taschenrechner-Beweis, shadow-20260712-025218).
- Versionskette/Tombstone: W6-Dramaturgie (ein Byte kippt → v1 kommt zurück,
  Boot 4 lädt nichts — shadow-20260712-171300, 403/403).

**Merke:** Der Film BEHAUPTET mit diesen Details nichts über den Ist-Stand
(Owner-Entscheidung „Nur Zielbild") — er benutzt sie als wahre Textur. Die
Beweis-Erzählung wohnt in den Kapiteln darunter.

---

## 5. Technik-Spezifikation

- **Ein `<svg>`, eine JS-Master-Uhr (Owner-Entscheid 2026-07-19, ersetzt
  SMIL):** Die Welt bleibt reines SVG (Look und Schärfe der Site), aber die
  120-s-Uhr treibt ein kleines eigenes Timeline-Modul per
  `requestAnimationFrame` (`ui-lab/site/film.js`, null Fremdcode): Szenen
  als deklarative Keyframe-Tabellen in SEKUNDEN (nicht Bruchzahlen),
  Easing-Helfer, ein `t`-Wert für alles. Kein SMIL im Film; die
  Kapitel-SVGs darunter behalten ihr SMIL unverändert.
  Warum: Umtimen = eine Zahl ändern; Spulen/Scrubben beim Entwickeln;
  Kamera als Code — der Hauptschmerz von 120 s SMIL-Choreografie entfällt.
- **Kamera-Rig:** eine äußere `<g id="cam">`, deren `transform`
  (translate + scale) die Timeline pro Frame aus einer Kamera-Keyframe-
  Tabelle setzt; Zoom-Ins in Screens sind Kamera-Fahrten auf
  vorgezeichnete Screen-Gruppen (kein Szenenwechsel).
- **Entwicklungs-Werkzeug:** Query-Param `film=scrub` blendet einen
  Spul-Regler + Szenen-Sprungmarken ein (nur Entwicklung, nicht im
  Site-Modus sichtbar).
- **Komponenten-Bibliothek in `<defs>`:** `block` (Sockel/Lampe/Kerben),
  `door` (Boden-Segment + Sockel), `key` (+Zeremonie-Animation als
  wiederverwendbares Fragment), `vein` (Boden-Ader mit Puls via
  `stroke-dashoffset`), `capsule`, `stamp`, `ring-segment`, `gate`,
  `crystal`. Ziel: Szenen sind Instanzen, nicht Kopien.
- **Puls-Kabel:** `stroke-dasharray` + animiertes `stroke-dashoffset`
  (Richtung = Übertragungsrichtung), idle-Zustand als Opacity-Atmen.
- **Größenbudget:** Ziel ≤ 450 KB für den Film-SVG-Block (der 32s-Opener
  liegt bei ~600 Zeilen; 14 Szenen mit Defs-Wiederverwendung realistisch
  3–5k Zeilen). Harte Grenze: die Seite bleibt eine einzige lokale Datei
  ohne Build-Schritt.
- **Performance-Regeln:** keine SVG-Filter-Orgien (kein `feGaussianBlur`
  flächig animiert), Glühen über vorgezeichnete Gradient-Shapes; max ~40
  gleichzeitig animierte Elemente pro Szenenfenster; Text als `<text>`
  (kein Pfad-Outlining), monospace-Stack der Site.
- **Determinismus/QA-Haken:** `anim=0` friert ein, `animt=<sek>` setzt die
  JS-Uhr exakt auf t und rendert genau einen Frame — noch deterministischer
  als SMIL-`setCurrentTime`; `scroll=N`-Selbsttest-Falle beachten (Frames
  isoliert per Scratch-HTML prüfen, bekannte Regel).
  `prefers-reduced-motion` → Poster-Frame bei t=118 (die Kreis-Totale)
  statt Loop.
- **Barrierefreiheit:** ein ausführliches `aria-label` auf dem Film-SVG
  (englisch), wie beim jetzigen Opener.
- **Einbau-Ort:** ersetzt den Inhalt der Sektion
  `<section class="website-story story-opener">` in `raios-ui-lab.html`;
  Kicker-Text wird `In two minutes` (statt „In 32 Sekunden") — die
  Film-Sektion ist komplett englisch (Owner 2026-07-19), der Rest der Site
  bleibt deutsch; Story-CSS-
  Erweiterungen in `ui-lab/site/story.css` (neue Klassen mit `film-`-Präfix,
  bestehende `aib-*`-Klassen bleiben bis zum Rückbau unangetastet).

---

## 6. Produktionsplan — 7 Phasen (je mit Definition of Done)

Jede Phase ist einzeln committbar und einzeln sehenswert (kein
Alles-oder-nichts). Aufwandsklassen: S/M/L/XL.

**P1 — Drehbuch-Freeze (S) — ✅ DONE 2026-07-19**
Owner hat die 14-Szenen-Liste eingefroren und alle offenen Fragen
entschieden (§11). Das Storyboard §3 ist ab jetzt die bindende Vorlage.

**P2 — Animatic (M)**
Der ganze Film als Grau-Klötzchen: alle 14 Szenen auf der echten 120-s-Uhr,
Kamera-Fahrten drin, keine Ausarbeitung. Zweck: Timing fühlen, Längen
schieben, BEVOR Detailarbeit versenkt wird. Erstes Deliverable von P2 ist
das Timeline-Modul selbst (`film.js`-Gerüst: Uhr, Keyframe-Tabellen,
Kamera, Scrub-Regler) — es wird danach unverändert vom echten Film benutzt.
DoD: `film-animatic`-SVG läuft als Loop; Frame-Shots bei t=5/20/40/60/90/115
sauber; Owner-Sichtung („fühlt sich das Tempo richtig an?").

**P3 — Welt-Bau (L)**
Die Komponenten-Bibliothek (§5) + die statische Welt in Endqualität: Fels,
Kristalle, Boden, Block-Redesign, Türen, Adern, Tor, Prüf-Keller. Noch ohne
Szenen-Feinanimation.
DoD: eine Standbild-Totale der Welt in Endqualität (t=113-Look); alle
Komponenten aus `<defs>` instanziierbar; Größenbudget-Zwischenmessung.

**P4 — Akt-Animation (XL, drei Teil-Lanes)**
P4a Akt I (S1–S3), P4b Akt II (S4–S9), P4c Akt III (S10–S14). Disjunkte
SVG-Gruppen = parallele Lanes möglich.
DoD je Akt: läuft auf der Master-Uhr im Kontext der Nachbar-Akte; alle
Gesetzbuch-Regeln (§2) eingehalten — Review-Checkliste ist §2.9.

**P5 — Screen-Einbauten (M)**
Die vier echten UI-Momente (S1, S6, S11, S12) in Dream-Optik, Wortlaute aus
§4 exakt.
DoD: Screen-Frames als Einzel-Shots pixel-geprüft gegen die Dream-UI-Referenz
(Panel-Farben/Metriken stimmen mit `ui-lab`-Tokens überein).

**P6 — Integration + Feinschliff (M)**
Opener-Austausch in `raios-ui-lab.html`, Kicker/Intro-Texte angepasst,
Loop-Naht poliert (S14→S1), reduced-motion-Poster, aria-label.
DoD: `?site=1` zeigt den Film als Herzstück; alte `aib-*`-Sektion entfernt
oder hinter Kommentar geparkt; Seite lädt lokal per `file://` ohne Fehler.

**P7 — QA-Gate (S/M)**
Frame-Matrix: Shots bei t = 3, 10, 20, 30, 38, 46, 57, 67, 76, 84, 92, 101,
110, 117 (≈ eine pro Szene) via `anim=0&animt=…` + Scratch-HTML-Isolation;
Sichtprüfung gegen §2-Gesetzbuch und §3-Beschreibungen; Chromium + Firefox;
Dateigrößen-Messung; `prefers-reduced-motion`-Test.
DoD: alle Frames entsprechen dem Storyboard; kein Gesetzbuch-Verstoß; Budget
gehalten. Erst dann gilt der Film als „fertig".

---

## 7. Warum dieser Plan so aussieht — die Zweitmeinungen

**Codex A (unabhängiges Storyboard, 27 Szenen, ~7:30 min):** Übernommen:
die **Kapsel als Kontinuitäts-Token**, die Vier-Karten-Feedback-Szene, das
Doppel-Build-Bild „zweimal bauen, einmal glauben", die Anti-Patterns (keine
Insel-Montage; Sandbox nicht als eine Magie-Blase — unsere Antwort darauf:
verschiedene Grenz-TYPEN sichtbar machen: Wand/fehlende Tür/Fuel; nicht mit
der laufenden App enden — unser Akt III). Nicht übernommen: 7:30 min Länge
(Owner: ~2 min) und das durchgängige Vier-Farben-Status-System (Owner: Nur
Zielbild).

**Codex B (Lücken-Audit):** Übernommen: die fünf echten Wow-Momente als
Textur-Quellen (§4), die Architektur-Korrektur **Treiber-in-Kernel (ADR
0005)** als hartes Gesetz (§2.5, §2.9), „nicht existierende Tür" statt
„verschlossene Tür" für ungewährte Imports, RECLOG-Fluss nur als
append-only-Bild.

**DISSENS-BOX (festgehalten, Owner entscheidet — beeinflusst NICHT den
Filmbau):** Beide Zweitmeinungen empfehlen unabhängig voneinander sichtbare
Status-Stempel („heute bewiesen" vs. „Ziel") IM Film; der Owner hat „Nur
Zielbild" gewählt. Brücke, falls gewünscht (billig, jederzeit nachrüstbar):
direkt unter dem Film eine schmale Zeile
`What you just saw, minus the future: 10 proven moments →` als Link auf ein
kleines Kapitel mit den zehn stärksten ECHTEN Momenten samt Report-IDs (das
Inventar existiert bereits). Der Film selbst bleibt rein.
**Owner-Entscheid 2026-07-19: Brücke WEGLASSEN.** Ehrlichkeit wohnt
ausschließlich in den Kapiteln + der „Ehrlichen Bilanz". Die Zeile bleibt
hier als dokumentierte, jederzeit nachrüstbare Option stehen (~30 min
Aufwand), wird aber nicht gebaut.

---

## 8. Folgearbeiten an den bestehenden Kapiteln (unabhängig vom Film)

Die Kapitel bleiben (Owner-Entscheidung) — aber das Lücken-Audit fand
Stellen, wo die Site der bindenden SCOPE inzwischen widerspricht. Eigene
kleine Text-Lane, getrennt vom Film:

1. **Kapitel „Treiber wohnen nicht im Kernel" umbauen** — widerspricht ADR
   0005/0015 (Treiber bleiben bewusst im Kernel; Isolation = Wasm-Dienste
   mit Import-Grants). Umerzählen auf: „Der Käfig ist Wasm, nicht der
   Treiber-Umzug" — oder als „ursprüngliche Idee, bewusst geändert"
   kennzeichnen. (Hinweis: §5/§6-Wording in SCOPE wartet selbst noch auf die
   Owner-Reframe-Freigabe — Kapitel-Umbau danach takten.)
2. IOMMU/DMA-Abwehr in „Schlüsselbund"/„Drei Angriffe" als Zukunft markieren
   (VT-d ist Struktur-Probe, Translation nicht aktiv); die zwei ECHTEN
   Angriffe (OOB, Import-Deny) dafür präzise zeigen.
3. „Kill in 0,8 s" → „sofort" (die <1-s-SLA ist als Prädikat noch offen).
4. Surface-Satz präzisieren: QEMU täglich bewiesen; Surface bootet
   experimentell, WiFi-Chip DETECTED (Association/DHCP offen).
5. „RECLOG wird nie überschrieben"-Bilanz-Karte: auf Host-Report-Realität
   beziehen oder als Ziel markieren.
6. Nachtschleifen-Kapitel: als Zielbild kennzeichnen (Watchdog/Funksteckdose/
   ramoops sind offene Boxen).

---

## 9. Rückmeldungen an das Haupt-Repo (raios2-Loop, nicht Site)

Beim Audit gefunden, gehört in den Orchestrator-Loop:
1. `shadow-20260718-082526-6872.json` (B3A-1c „33/33") trägt
   `result: failed` (Harness konnte die serielle Logdatei nicht lesen —
   gesperrt durch anderen Prozess). STATUS nennt ihn VERIFIED-CLOSED.
   → Profil sauber wiederholen, bevor er je als Beleg zitiert wird.
2. `shadow-20260717-114259-19696.json` (B1.3 RUIP-Persistenz) liegt nicht
   (mehr) in `release/vm-reports/`. → wiederherstellen oder Archiv-Nachweis.

---

## 10. Aufwand & Reihenfolge (Empfehlung)

P1 jetzt (Owner-OK) → P2 als nächste Lane (eine Session) → P3 (eine große
Lane) → P4a/b/c parallel als drei Lanes → P5 → P6 → P7. Die Kapitel-Fixes
(§8) können jederzeit parallel als Text-Lane laufen — disjunkte Datei-Zonen
innerhalb `raios-ui-lab.html` beachten (Film-Sektion vs. Kapitel-Sektionen).

---

## 11. Entschieden (Owner, 2026-07-19 — alle P1-Fragen geschlossen)

1. **Storyboard:** 14-Szenen-Liste (§3) **eingefroren** → P1 done.
2. **Wunsch-Satz S1:** `build me a music player` (Anschluss an die Bildwelt).
3. **Titelkarte S14:** `raiOS — the OS that builds its own software.` /
   `Every gate you just saw is a real test.`
4. **S12-Nachbar-Crash:** bleibt drin (Blast-Radius im Vorbeigehen zeigen).
5. **Dissens-Brücke:** weglassen (siehe §7).
6. **Ton:** stumm.
7. **Rahmen-Text:** Film-Sektion komplett englisch; Rest der Site deutsch.
8. **Agent-Look:** rein abstrakter Block, keine Anthropomorphisierung.
9. **Technik:** SVG-Welt + eigene JS-Timeline (kein SMIL im Film, kein
   three.js, null Fremdcode); Kapitel-SVGs behalten ihr SMIL.

---

## Anhang A — Bauteil-Bibliothek der echten Maschine (für P3/P4)

Quelle: Codebasis-Landkarte (Explore-Sweep 2026-07-19 über `seed-kernel/src`
178 Dateien / ~191k LOC, `crates/*`, `wasm-guests/`, `vm-harness/`). Die Welt
im Film ist Zielbild — aber jedes gezeichnete Bauteil hat hier sein reales
Gegenstück. Wer eine Szene baut, nimmt Namen und Proportionen von dieser
Liste.

### A.1 Fels-Kristalle (Kernel besitzt die Hardware — ADR 0005)
| Kristall im Film | Reales Bauteil | Notiz |
| --- | --- | --- |
| CPU/RAM | `heap.rs`/`memory.rs` (64 MiB–4 GiB Heap), `time.rs` (TSC) | |
| DISPLAY | `framebuffer.rs` + `text.rs` (8×8-Font, Doppelpuffer) | Surface-bewiesen |
| USB | `usb.rs` (4711 Z.: xHCI, HID, Hub/TT/Route-String) | Surface-bewiesen inkl. echter HW-Fixes |
| NIC | `e1000.rs` + `net.rs` (smoltcp: DHCP/DNS/TCP) | QEMU-only |
| DISK | `ahci.rs` (4181 Z., größter Treiber) | QEMU-only |
| WIFI | `marvell_wifi_pcie.rs` (3612 Z.) + `wifi.rs` | nur auf echtem Surface DETECTED — „der schlafende Kristall" |
| RNG | `entropy.rs` (RDRAND) | Surface-bewiesen |
| TPM | `tpm2_transport.rs` + `owner_key.rs` | heute Erkennung, kein Sealing |
| SERIAL | `serial.rs` (COM1 + Ringpuffer) | die autoritative Log-Senke |

### A.2 Boden & Adern (Genesis/Evidenz)
- **RECLOG** — hash-verketteter Append-only-Log (eigene GPT-Region):
  das reale Vorbild des „RECLOG-Flusses" in S9.
- **ARTSTOR** — Content-Store (Blobs, BuildFS-Chunk-Manifeste): die Ader,
  durch die in S6 die CAS-Chunks strömen.
- **BOOTCTL A/B** — Bootslot-Zustand: Vorbild für Versions-Kerben/Rollback.
- Modul-Förderband (Tor-Maschinerie): Kandidat → Shadow-VM-Report →
  Attestation → Approval → A/B-Promotion/Rollback
  (`agent_protocol_module_*`-Familie, ~20 Dateien).

### A.3 Block-Namen für S13 (echte Service-IDs)
`svc.demo.hello` · `svc.demo.echo` · `svc.demo.bufecho` ·
`svc.demo.dnsparse` · `svc.demo.httphead` · `svc.demo.certwindow` ·
`svc.demo.certspki` · `svc.build.assembler` · `svc.net.acquire.w7` ·
`svc.user.shell` · `svc.workspace.installed` — die Zeitraffer-Blöcke in S13
dürfen diese echten Namen tragen (Devs greppen und finden sie).

### A.4 Bauplatz (S4–S8, reale Gegenstücke)
- 30-Import-WASI-Welt: `wasi_build_job.rs` / `wasi_preview1` (6213 Z.).
- Deterministischer Grün-Thread-Scheduler mit Fuel-Runden:
  `raios-core/thread_scheduler.rs` (1292 Z.) — „replay-gleiche Traces".
- 91-MB-rustc + 71-MB-Sysroot aus dem ARTSTOR-Store, in-kernel instanziiert.
- Winziger Eigen-Compiler als Kontrast-Detail: `raios-lang` → `RAIOS_WASM_IR_V1`
  → `svc.build.assembler` (52-Byte-Module).

### A.5 Zahlen-Textur (wahr, für Labels/Feinschliff)
- **346 unsafe-Blöcke im Kernel-Fels — 0 in raios-core, 0 in den übrigen
  Crates**: „alles Gefährliche wohnt im Fels" ist wörtlich wahr (Film darf
  das als Fels-Gravur zeigen).
- 942 Dateien Testgedächtnis (`release/vm-reports`: 445 Reports + 497
  SHA-256-Sidecars), ~40 fokussierte Smoke-Profile, 23 ADRs.
- Kernel ~191k LOC / 178 Dateien; raios-core ~56k LOC / 82 Dateien.

### A.6 QEMU-vs-Surface-Legende (nur für die KAPITEL-Fixes §8, nicht im Film)
Surface-bewiesen: Boot/UI/Framebuffer, USB-xHCI+HID (+Hub-Tastatur), RDRAND,
Marvell DETECTED + Scan. QEMU-only: AHCI/RECLOG-Persistenz, e1000/DHCP,
Provider-TLS-Pfad, Structured-Store „C1", der komplette rustc-On-Device-Track.
