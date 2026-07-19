# TEMPLATES — Skelette für den Orchestrator

> Struktur ist Pflicht, Inhalt ist Denkarbeit. Platzhalter `<...>` zwingen dich,
> genau dort nachzudenken. Kein Feld leer lassen — "keins" ist eine gültige Antwort,
> Weglassen nicht. Vorlagen dürfen sich weiterentwickeln: Änderung = normaler
> Commit mit Begründung.

---

## 1. Lane-Auftrag

```markdown
# Auftrag: <kurzer Titel>
Scope-Checkbox: <exakter Wortlaut aus SCOPE.md, §>
Lane: <N> · Modus: <explorativ | konservativ>

## Ziel
<1–3 Sätze: Was existiert, wenn du fertig bist>

## Files
Ändern: <Liste> · Neu: <Liste> · Tabu: <Liste oder "alles andere">

## Definition of Done
- [ ] <Predicate(s), die grün sein müssen>
- [ ] <Negativtest, der die Grenze belegt>
- [ ] <weitere harte Kriterien>

## Tabus
<Was diese Lane explizit NICHT tun darf, über AGENTS.md hinaus — oder "keine">
```

## 2. Lane-System-Prompt

```markdown
Du bist <Rolle: Spezialist für X, denkt in Y>.

## Dein Gegenstand
<Kuratierter Kontext: NUR was diese Aufgabe braucht — Register-Map-Ausschnitt,
ADR-Absatz, Hardware-Manifest-Teil, Code-Stelle. Max ~1 Seite. Du bist der
Bibliothekar: leg das aufgeschlagene Buch hin, nicht die Bibliothek.>

## Deine Umgebung
Domäne: <Name> · Capabilities: <exakte Liste> · Diagnostik: <Kanal, Format>
Crash-Kosten: <billig, Watchdog startet neu | teuer, konservativ arbeiten>

## Bekannte Fallen
<Erkenntnisse aus früheren Versuchen: "X scheitert an Y, unversucht ist Z" —
oder "keine, Neuland">
```

## 3. Blocked-Report (von Lane, bei Stuck nach 3 Versuchen)

```markdown
# Blocked: <Auftragstitel>
## Versucht
1. <Ansatz> → <Beobachtung, mit Log-/Diagnostik-Auszug>
2. <...>
3. <...>
## Hypothese
<Warum es vermutlich scheitert — ehrlich markieren, was Vermutung ist>
## Unversucht
<Ansätze, die noch offen sind, und warum du sie nicht gewählt hast>
```

## 4. ADR

```markdown
# NNNN — <Entscheidung als Aussagesatz>
Date: <YYYY-MM-DD> · Status: active
<!-- The header MUST be English `Date:` and `Status:` — the check-docs-hygiene
     adr-form rule matches `^Date:` and `Status:`; German `Datum:`/`aktiv`
     fail the gate. The prose below the header stays German. -->
## Kontext
<Warum stand die Frage im Raum — 2–4 Sätze>
## Entscheidung
<Was gilt ab jetzt>
## Alternativen & Zweitmeinungen
<Was wurde verworfen, warum. Bei Dissens der Berater: beide Positionen.>
## Folgen
<Was wird dadurch leichter, was schwerer, was ist der bewusste Trade>
```
