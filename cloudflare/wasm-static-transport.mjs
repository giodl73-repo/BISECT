// Route only package-generated exact WASM paths to precompressed static assets.
export function hostedWasmTransport(fallback, routes) {
  return {
    async fetch(request, env, context) {
      const url = new URL(request.url);
      const asset = Object.hasOwn(routes, url.pathname) ? routes[url.pathname] : null;
      if (!asset) return fallback.fetch(request, env, context);
      if (request.method !== 'GET' && request.method !== 'HEAD') {
        return new Response(null, { status: 405, headers: { Allow: 'GET, HEAD' } });
      }
      // Fetch a fixed internal path. Client Range, conditional headers, and
      // query parameters cannot select or truncate the stored gzip member.
      const response = await env.ASSETS.fetch(new Request(new URL(asset, url), {
        method: request.method, headers: { 'Accept-Encoding': 'identity' },
      }));
      const encoding = response.headers.get('Content-Encoding');
      if (response.status !== 200 || (encoding && encoding !== 'identity') ||
          (request.method === 'GET' && !response.body)) {
        await response.body?.cancel();
        return new Response('WASM asset unavailable.', {
          status: 502, headers: { 'Cache-Control': 'no-store' },
        });
      }
      return new Response(request.method === 'HEAD' ? null : response.body, {
        encodeBody: 'manual',
        headers: {
          'Content-Type': 'application/wasm',
          'Content-Encoding': 'gzip',
          'Cache-Control': 'no-cache',
          'X-Content-Type-Options': 'nosniff',
          'Vary': 'Accept-Encoding',
        },
      });
    },
  };
}
