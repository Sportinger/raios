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

    return env.ASSETS.fetch(request);
  },
};
