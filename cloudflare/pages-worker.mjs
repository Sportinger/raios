const CANONICAL_HOST = "raios.tech";
const REDIRECT_HOSTS = new Set([
  "www.raios.tech",
  "raios.info",
  "www.raios.info",
  "raios.site",
  "www.raios.site",
  "raios.me",
  "www.raios.me",
]);

export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    if (REDIRECT_HOSTS.has(url.hostname.toLowerCase())) {
      url.protocol = "https:";
      url.hostname = CANONICAL_HOST;
      url.port = "";
      return Response.redirect(url.toString(), 308);
    }

    const assetResponse = await env.ASSETS.fetch(request);
    const headers = new Headers(assetResponse.headers);
    const contentType = (headers.get("Content-Type") || "").toLowerCase();

    if (contentType.startsWith("text/html") || url.pathname === "/version.json") {
      headers.set("Cache-Control", "no-store");
      headers.set("Pragma", "no-cache");
      headers.set("Expires", "0");
    } else {
      headers.set("Cache-Control", "public, max-age=0, must-revalidate");
    }

    return new Response(assetResponse.body, {
      status: assetResponse.status,
      statusText: assetResponse.statusText,
      headers,
    });
  },
};
