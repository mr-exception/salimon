# syntax=docker/dockerfile:1

FROM node:22-alpine AS build
WORKDIR /app

# Install exactly the dependencies recorded by the client lockfile.
COPY client/package.json client/package-lock.json ./
RUN npm ci --no-audit --no-fund

COPY client/ ./
RUN npm run build

# Only the compiled site is copied into the final image.
FROM nginx:stable-alpine
COPY docker/nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=build /app/dist/ /usr/share/nginx/html/
EXPOSE 80
