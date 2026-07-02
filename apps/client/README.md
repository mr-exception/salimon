# Client

React and Phaser client for Salimon.

The client loads world data from:

```text
https://hjp81v6wyh.execute-api.us-east-1.amazonaws.com/world
```

Set `VITE_API_BASE_URL` when building to override the default API Gateway base
URL:

```sh
VITE_API_BASE_URL=https://example.execute-api.eu-west-1.amazonaws.com \
  npm run build --workspace client
```

The value must not include `/world`; the client adds that path.
