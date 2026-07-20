// Website-only scroll story below the Surface composite. Reveals sections on
// scroll and keeps screenshots deterministic. Never touches renderer or lab
// state.
//
// Query params (website mode only):
//   anim=0    - apply all reveal end states instantly and freeze SVG loops
//   scroll=N  - jump to N pixels from the top (for headless section shots)

(function () {
  const storyParams = new URLSearchParams(location.search);
  const reduceMotion =
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  const animationsOff = storyParams.get("anim") === "0" || reduceMotion;
  const sections = Array.from(document.querySelectorAll("[data-story-reveal]"));

  const scrollTarget = Number.parseInt(storyParams.get("scroll") || "", 10);
  if (Number.isFinite(scrollTarget) && scrollTarget > 0) {
    const jumpToTarget = () => window.scrollTo(0, scrollTarget);
    jumpToTarget();
    // Re-assert after every load handler: the lab selftest toggles shell modes
    // on load and its canvas focus scrolls the page back to the composite.
    window.addEventListener("load", () => {
      jumpToTarget();
      setTimeout(jumpToTarget, 0);
      setTimeout(jumpToTarget, 250);
    });
  } else {
    if ("scrollRestoration" in history) history.scrollRestoration = "manual";
    const resetToTop = () => window.scrollTo({ top: 0, left: 0, behavior: "auto" });
    resetToTop();
    window.addEventListener("load", () => {
      resetToTop();
      requestAnimationFrame(resetToTop);
      setTimeout(resetToTop, 250);
    }, { once: true });
  }

  if (storyParams.get("storydebug") === "1") {
    const reportStoryMetrics = () => {
      const probe = document.querySelector(".story-what .story-copy");
      const probeStyle = probe ? getComputedStyle(probe) : null;
      document.body.dataset.storyMetrics = JSON.stringify({
        scrollY: Math.round(window.scrollY),
        docHeight: Math.round(document.documentElement.scrollHeight),
        probe: probeStyle
          ? {
              opacity: probeStyle.opacity,
              display: probeStyle.display,
              visibility: probeStyle.visibility,
              color: probeStyle.color,
              transform: probeStyle.transform,
            }
          : "missing",
        centerElement: (document.elementFromPoint(
          Math.round(window.innerWidth / 2),
          Math.round(window.innerHeight / 2),
        ) || { tagName: "none" }).tagName,
        sheets: Array.from(document.styleSheets).map((sheet) => {
          let rules = "ERR";
          try { rules = sheet.cssRules.length; } catch (error) { /* cross-origin */ }
          return String(sheet.href || "inline").split("/").pop() + ":" + rules;
        }),
        sections: sections.map((section) => ({
          cls: section.className.split(" ", 2).join(" "),
          top: Math.round(section.getBoundingClientRect().top + window.scrollY),
          height: Math.round(section.getBoundingClientRect().height),
        })),
      });
    };
    reportStoryMetrics();
    window.addEventListener("load", () => setTimeout(reportStoryMetrics, 300));
  }

  if (animationsOff) {
    document.body.classList.add("story-anim-off");
    const freezeTime = Number.parseFloat(storyParams.get("animt") || "2.6");
    document.querySelectorAll(".website-story svg").forEach((svg) => {
      if (typeof svg.setCurrentTime === "function") svg.setCurrentTime(freezeTime);
      if (typeof svg.pauseAnimations === "function") svg.pauseAnimations();
    });
  }

  if (animationsOff || typeof IntersectionObserver !== "function") {
    sections.forEach((section) => section.classList.add("in-view"));
    return;
  }

  const revealObserver = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (!entry.isIntersecting) return;
        entry.target.classList.add("in-view");
        revealObserver.unobserve(entry.target);
      });
    },
    { threshold: 0.18, rootMargin: "0px 0px -8% 0px" },
  );
  sections.forEach((section) => revealObserver.observe(section));
})();
