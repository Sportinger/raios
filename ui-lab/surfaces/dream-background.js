let bgCache = null;
function buildDreamBg() {
  const c = document.createElement("canvas");
  c.width = cv.width; c.height = cv.height;
  const g = c.getContext("2d");
  const shades = ["#08090B", "#0B0D10", "#0E1014"];
  const bayer = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
  const logicalWidth = Math.floor(c.width / SCALE);
  const logicalHeight = Math.floor(c.height / SCALE);
  const verticalCenter = Math.floor(logicalHeight / 2) + Math.floor(logicalHeight / 40);
  const verticalSpan = Math.max(1, Math.floor(logicalHeight * 3 / 5));
  const horizontalCenter = Math.floor(logicalWidth / 2);
  const horizontalSpan = Math.max(1, Math.floor(logicalWidth * 21 / 32));
  for (let y = 0; y < logicalHeight; y++) {
    const t = 1 - Math.abs(y - verticalCenter) / verticalSpan;
    for (let x = 0; x < logicalWidth; x++) {
      const cx = 1 - Math.abs(x - horizontalCenter) / horizontalSpan;
      const v = Math.max(0, Math.min(1, t * 0.6 + cx * 0.55)) * (shades.length - 1);
      let i = Math.floor(v);
      if (bayer[y % 4][x % 4] / 16 < v - i) i++;
      g.fillStyle = shades[Math.min(i, shades.length - 1)];
      g.fillRect(x * 2, y * 2, 2, 2);
    }
  }
  let seed = 1337;
  const rnd = () => { seed = (seed * 1103515245 + 12345) & 0x7fffffff; return seed / 0x7fffffff; };
  const starCount = Math.round(c.width * c.height * 70 / (1280 * 800));
  for (let s = 0; s < starCount; s++) {
    const x = Math.floor(rnd() * logicalWidth), y = Math.floor(rnd() * logicalHeight);
    const bright = rnd() > 0.82;
    g.fillStyle = bright ? "#232833" : "#161A20";
    g.fillRect(x * 2, y * 2, 2, 2);
  }
  drawCandleInto(g);
  return c;
}

// Prozedurale Kerze als leises Hintergrund-Motiv — reine Geometrie, kein
// Bild-Asset. Scherenschnitt-Stil: EINE Farbe, Detail lebt in der Kontur.
// In physischer Aufloesung gezeichnet (1px, wie die feine Schrift) und
// einmal in den Hintergrund-Puffer gebacken (Laufzeit: 0).
function drawCandleInto(g) {
  const TONE = "#222933";
  g.fillStyle = TONE;
  const span = (x0, x1, y) => {
    if (x1 > x0) g.fillRect(Math.round(x0), Math.round(y), Math.max(1, Math.round(x1 - x0)), 1);
  };
  // Wie im Rust-Renderer bleibt X physisch fix; der Fuss folgt der realen
  // Framebuffer-Unterkante. Referenzkonturen bleiben an 1280x800 gebunden.
  const CX = 220, REFERENCE_BASE = 786, BASE = cv.height - 14;
  const verticalOffset = BASE - REFERENCE_BASE;
  const PLATE_TOP = BASE - 24, STICK_TOP = PLATE_TOP - 58, BODY_TOP = STICK_TOP - 108;

  // Teller mit Wachs-Pfuetze: Ellipse + weiche Randbuckel
  for (let y = PLATE_TOP; y <= BASE; y++) {
    const t = (y - PLATE_TOP) / (BASE - PLATE_TOP);
    const hw = 66 * Math.sin(Math.PI * Math.min(1, t * 1.04));
    span(CX - hw, CX + hw, y);
  }
  for (const [dx, w, h] of [[-34, 16, 5], [-12, 12, 4], [22, 18, 6], [44, 10, 4]]) {
    for (let i = 0; i < h; i++) {
      const hw = (w / 2) * Math.sqrt(1 - (i / h) * (i / h));
      span(CX + dx - hw, CX + dx + hw, PLATE_TOP - i);
    }
  }

  // Gedrechselter Staender: Fussflare -> Nodus-Kugel -> Kelchflare mit Rand
  for (let y = STICK_TOP; y <= PLATE_TOP; y++) {
    const t = (y - STICK_TOP) / (PLATE_TOP - STICK_TOP); // 0 oben .. 1 unten
    let hw = 9
      + 20 * Math.pow(Math.max(0, t - 0.72) / 0.28, 1.6)          // Fussflare
      + 7 * Math.exp(-Math.pow((t - 0.45) / 0.13, 2))             // Nodus
      + 11 * Math.pow(Math.max(0, 0.16 - t) / 0.16, 1.3);         // Kelchflare
    if (t < 0.06) hw = 21;                                         // Kelchrand
    span(CX - hw, CX + hw, y);
  }

  // Kerzenkoerper: organisch geschmolzene Kontur — Detail lebt an der KANTE
  // (seitliche Wachs-Wuelste als lange weiche Ausbuchtungen)
  const edge = (y) => {
    const referenceY = y - verticalOffset;
    return 14
      + 2.6 * Math.sin(referenceY * 0.05) + 1.6 * Math.sin(referenceY * 0.12 + 1.7)
      + 3.5 * Math.exp(-Math.pow((referenceY - (REFERENCE_BASE - 190 + 46)) / 18, 2));
  };
  for (let y = BODY_TOP; y <= STICK_TOP; y++) {
    let hw = edge(y);
    const top = y - BODY_TOP;
    if (top < 12) hw += 7 * Math.sin(Math.PI * top / 12) * (1 + 0.35 * Math.sin(y * 0.8)); // Schmelzrand-Ueberhang
    span(CX - hw, CX + hw, y);
  }

  // Tropfnasen AUSSEN an der Kontur: haengen sichtbar unter dem Schmelzrand,
  // mit runden Tropfen-Spitzen
  const drips = [
    [-20, 5, 30], [-24, 4, 16], [19, 6, 72], [23, 5, 38],
  ];
  for (const [dx, w, len] of drips) {
    for (let i = 0; i < len; i++) {
      const t = i / len;
      let hw = (w / 2) * (0.75 + 0.25 * Math.sin(i * 0.25));
      if (t > 0.82) hw = (w / 2) * Math.sqrt(Math.max(0, 1 - Math.pow((t - 0.82) / 0.18, 2))) * 1.15;
      span(CX + dx - hw, CX + dx + hw, BODY_TOP + 6 + i);
    }
  }
  // eine lange Nase ueber den Kelchrand bis fast zum Teller
  for (let i = 0; i < 48; i++) {
    const t = i / 48;
    let hw = 3.4 * (0.8 + 0.2 * Math.sin(i * 0.3));
    if (t > 0.8) hw = 4.2 * Math.sqrt(Math.max(0, 1 - Math.pow((t - 0.8) / 0.2, 2)));
    span(CX - 19 - hw, CX - 19 + hw, STICK_TOP + 2 + i);
  }

  // Docht: kurz sichtbar zwischen Kerze und Flamme
  for (let i = 0; i < 9; i++) span(CX + Math.round(i * 0.22) - 1, CX + Math.round(i * 0.22) + 1, BODY_TOP - i);

  // Flamme: schlanke, spitze Traene mit leichtem S-Schwung, Luecke zum Docht
  const FTOP = BODY_TOP - 68;
  for (let y = FTOP; y <= BODY_TOP - 12; y++) {
    const t = (y - FTOP) / (BODY_TOP - 12 - FTOP);   // 0 Spitze .. 1 Basis
    const hw = 7.2 * Math.sin(Math.PI * Math.pow(t, 0.55)) * (1 - 0.1 * Math.sin(3 * t));
    const lean = 3.5 * Math.sin(Math.PI * t) * (t - 0.45);
    span(CX + lean - hw, CX + lean + hw, y);
  }

  // Halo: duenner glatter Ring um die Flamme
  const HCX = CX, HCY = FTOP + 34, HR = 44;
  for (let a = 0; a < 360; a += 0.6) {
    const r = a * Math.PI / 180;
    g.fillRect(Math.round(HCX + HR * Math.cos(r)), Math.round(HCY + HR * Math.sin(r)), 1, 1);
    g.fillRect(Math.round(HCX + (HR - 1) * Math.cos(r)), Math.round(HCY + (HR - 1) * Math.sin(r)), 1, 1);
  }

  // Henkel: glatter dicker Ring rechts + S-Verbindung zum Staender
  const GX = CX + 74, GY = PLATE_TOP - 28, GR = 26;
  for (let a = 0; a < 360; a += 0.5) {
    const r = a * Math.PI / 180;
    for (let rr = GR - 4; rr <= GR; rr++) {
      g.fillRect(Math.round(GX + rr * Math.cos(r)), Math.round(GY + rr * Math.sin(r)), 1, 1);
    }
  }
  for (let x = CX + 18; x <= GX - GR + 4; x++) {
    const t = (x - CX - 18) / (GX - GR + 4 - CX - 18);
    const y = PLATE_TOP - 38 + 10 * Math.sin(Math.PI * t - Math.PI / 2);
    span(x, x + 1, y); span(x, x + 1, y + 1); span(x, x + 1, y + 2); span(x, x + 1, y + 3);
  }
}
